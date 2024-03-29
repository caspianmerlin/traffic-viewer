use std::{io::{BufRead, BufReader, ErrorKind, LineWriter, Write}, net::{TcpListener, TcpStream}, sync::{atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, Sender, TryRecvError}, Arc}, thread::{self, JoinHandle}, time::Duration};

use fsd_interface::{messages::{ClientQueryMessage, ClientQueryResponseMessage, FlightPlanMessage, MetarResponseMessage, TextMessage}, ClientQueryType, FsdMessageType};

use crate::ui::{Message, Ui};

use super::{metar::MetarProvider, vatsim::VatsimDataProvider, worker::FLIGHT_PLAN_RECIPIENT, Preferences};

const SERVER_CALLSIGN: &str = "SERVER";
const WELCOME_MESSAGE: &str = "Connected to Traffic Viewer. Welcome!";

pub struct Server {
    thread: Option<JoinHandle<()>>,
    should_terminate: Arc<AtomicBool>,
    sender: Sender<String>,
}

impl Server {
    pub fn new<U: Ui + 'static>(preferences: Preferences, vatsim_data_provider: VatsimDataProvider, metar_provider: MetarProvider, ui: U, should_terminate: Arc<AtomicBool>) -> Server {
        let u = ui.clone();
        let (tx, rx) = mpsc::channel();
        let thread = Some(server_thread(Arc::clone(&should_terminate), vatsim_data_provider, metar_provider, preferences, u, rx));
        Server {
            thread,
            should_terminate,
            sender: tx,
        }
    }

    pub fn send_packet(&mut self, message: String) {
        self.sender.send(message);
    }

    pub fn sender(&self) -> Sender<String> {
        self.sender.clone()
    }

}
impl Drop for Server {
    fn drop(&mut self) {
        self.should_terminate.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            thread.join().ok();
        }
    }
}

fn server_thread<U: Ui + 'static>(should_terminate: Arc<AtomicBool>, vatsim_data_provider: VatsimDataProvider, metar_provider: MetarProvider, preferences: Preferences, ui: U, receiver: Receiver<String>) -> JoinHandle<()> {
    thread::Builder::new().name("TrafficViewerFSDThread".into()).spawn(move|| {
        let tcp_listener = match TcpListener::bind("127.0.0.1:6809") {
            Ok(tcp_listener) => tcp_listener,
            Err(_) => {
                ui.dispatch_message(Message::FatalError(String::from("Unable to bind to localhost port 6809. Exiting.")));
                should_terminate.store(true, Ordering::Relaxed);
                return;
            },
        };
            
        
        while !should_terminate.load(Ordering::Relaxed) {
            tcp_listener.set_nonblocking(true).ok();
            match tcp_listener.accept() {
                Ok((stream, _)) => {
                    stream.set_nonblocking(false).ok();
                    let mut writer = LineWriter::new(stream.try_clone().unwrap());
                    while let Ok(_) = receiver.try_recv() {}
                    let this_connection_ended = Arc::new(AtomicBool::new(false));
                    // Spawn recv thread
                    let recv_thread = recv_thread(Arc::clone(&should_terminate), Arc::clone(&this_connection_ended), stream, vatsim_data_provider.clone(), metar_provider.clone(), preferences.clone(), ui.clone());
                    while !should_terminate.load(Ordering::Relaxed) && !this_connection_ended.load(Ordering::Relaxed) {
                        match receiver.try_recv() {
                            Ok(msg) => _ = {
                                writer.write(&string_to_byte_slice(&format!("{}\r\n", msg))).ok();
                            },
                            Err(TryRecvError::Disconnected) => {
                                break;
                            },
                            Err(TryRecvError::Empty) => {
                                thread::sleep(Duration::from_millis(100));
                                continue;
                            }
                        }

                    }
                    writer.get_ref().set_nonblocking(true).ok();
                    writer.get_ref().shutdown(std::net::Shutdown::Both).ok();
                    recv_thread.join().unwrap();
                },
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
                // TODO: Come back and handle this gracefully
                Err(_) => break,
            }
        }
    }).unwrap()
}


fn recv_thread<U: Ui + 'static>(should_terminate: Arc<AtomicBool>, this_connection_closed: Arc<AtomicBool>, tcp_stream: TcpStream, vatsim_data_provider: VatsimDataProvider, metar_provider: MetarProvider, mut preferences: Preferences, ui: U) -> JoinHandle<()> {
    preferences.set_es_callsign(String::new());
    thread::Builder::new().name(String::from("TrafficViewerFSDRecvThread")).spawn(move|| {
        let mut writer = LineWriter::new(tcp_stream.try_clone().unwrap());
        let mut reader = BufReader::new(tcp_stream);
        
        while !should_terminate.load(Ordering::Relaxed) {
            let mut buffer = Vec::with_capacity(512);
            match reader.read_until(b'\n', &mut buffer) {
                Ok(0) => {
                    ui.dispatch_message(Message::EuroscopeDisconnected);
                    this_connection_closed.store(true, Ordering::Relaxed);
                    break;
                },
                Ok(_) => {
                    let message = byte_slice_to_string(&buffer);
                    if let Ok(fsd_message) = fsd_interface::parse_message(message.trim()) {
                        match fsd_message {
                            FsdMessageType::AtcRegisterMessage(msg) => {
                                preferences.set_es_callsign(msg.from.clone());
                                ui.dispatch_message(Message::EuroscopeConnected(msg.from.clone()));
                                writer.write(&string_to_byte_slice(&format!("{}\r\n", TextMessage::new(SERVER_CALLSIGN, msg.from, WELCOME_MESSAGE)))).ok();
                            },
                            FsdMessageType::MetarRequestMessage(msg) => {
                                println!("METAR requested for {}", msg.station);
                                if let Some(metar) = metar_provider.lookup_metar(&msg.station) {
                                    let response = format!("{}\r\n", MetarResponseMessage::new(SERVER_CALLSIGN, msg.from, metar));
                                    println!("Sending {}", response);
                                    writer.write(&string_to_byte_slice(&response)).ok();
                                }
                            },
                            FsdMessageType::ClientQueryMessage(cqm) => match cqm.query_type {
                                ClientQueryType::RealName => {
                                    if let Some(details) = vatsim_data_provider.get_aircraft_details(&cqm.to) {
                                        let real_name = details.name;
                                        let message = ClientQueryResponseMessage::real_name(cqm.to, cqm.from, real_name, String::new(), 1);
                                        let response = format!("{}\r\n", message);
                                        writer.write(&string_to_byte_slice(&response)).ok();
                                    }
                                },
                                ClientQueryType::FlightPlan(subject) => {
                                    if let Some(flight_plan) = vatsim_data_provider.get_aircraft_details(&subject).and_then(|details| details.flight_plan).map(fsd_interface::FlightPlan::from) {
                                        let message = FlightPlanMessage::new(cqm.from, subject, flight_plan);
                                        let response = format!("{}\r\n", message);
                                        writer.write(&string_to_byte_slice(&response)).ok();
                                    }
                                },
                                _ => {},

                            }, 
                            
                            _ => {},
                        }
                    }
                },
                Err(_) => {
                    ui.dispatch_message(Message::EuroscopeDisconnected);
                    this_connection_closed.store(true, Ordering::Relaxed);
                    break;
                },
            }
        }
    }).unwrap()
}



#[inline]
fn byte_slice_to_string(slice: &[u8]) -> String {
    slice.iter().map(|c| *c as char).collect()
}

#[inline]
fn string_to_byte_slice(string: &str) -> Vec<u8> {
    string.chars().map(|c| c as u8).collect()
}