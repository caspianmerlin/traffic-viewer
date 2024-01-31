use std::{io::{BufRead, BufReader, ErrorKind, LineWriter, Write}, net::{TcpListener, TcpStream}, sync::{atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, Sender, TryRecvError}, Arc}, thread::{self, JoinHandle}, time::Duration};

use fsd_interface::{messages::{MetarResponseMessage, TextMessage}, FsdMessageType};

use crate::ui::Ui;

use super::{metar::MetarProvider, vatsim::VatsimDataProvider, Preferences};

const SERVER_CALLSIGN: &str = "SERVER";
const WELCOME_MESSAGE: &str = "Connected to Traffic Viewer. Welcome!";

pub struct Server {
    thread: Option<JoinHandle<()>>,
    should_terminate: Arc<AtomicBool>,
    sender: Sender<String>,
}

impl Server {
    pub fn new<U: Ui>(preferences: Preferences, vatsim_data_provider: VatsimDataProvider, metar_provider: MetarProvider, ui: U, should_terminate: Arc<AtomicBool>) -> Server {
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

}
impl Drop for Server {
    fn drop(&mut self) {
        self.should_terminate.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            thread.join().ok();
        }
    }
}

fn server_thread<U: Ui>(should_terminate: Arc<AtomicBool>, vatsim_data_provider: VatsimDataProvider, metar_provider: MetarProvider, preferences: Preferences, ui: U, receiver: Receiver<String>) -> JoinHandle<()> {
    thread::Builder::new().name("TrafficViewerFSDThread".into()).spawn(move|| {

        let tcp_listener = TcpListener::bind("127.0.0.1:6809").unwrap();
        tcp_listener.set_nonblocking(true);
        while !should_terminate.load(Ordering::Relaxed) { 
            match tcp_listener.accept() {
                Ok((stream, _)) => {
                    let mut writer = LineWriter::new(stream.try_clone().unwrap());

                    // Spawn recv thread
                    let recv_thread = recv_thread(Arc::clone(&should_terminate), stream, vatsim_data_provider.clone(), metar_provider.clone(), preferences.clone(), ui);
                    while !should_terminate.load(Ordering::Relaxed) {
                        match receiver.try_recv() {
                            Ok(msg) => _ = writer.write(&string_to_byte_slice(&msg)).ok(),
                            Err(TryRecvError::Disconnected) => break,
                            Err(TryRecvError::Empty) => {
                                thread::sleep(Duration::from_millis(100));
                                continue;
                            }
                        }
                    }
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


fn recv_thread<U: Ui>(should_terminate: Arc<AtomicBool>, tcp_stream: TcpStream, vatsim_data_provider: VatsimDataProvider, metar_provider: MetarProvider, preferences: Preferences, ui: U) -> JoinHandle<()> {
    thread::Builder::new().name(String::from("TrafficViewerFSDRecvThread")).spawn(move|| {
        let mut writer = LineWriter::new(tcp_stream.try_clone().unwrap());
        let mut reader = BufReader::new(tcp_stream);
        
        while !should_terminate.load(Ordering::Relaxed) {
            let mut buffer = Vec::with_capacity(512);
            match reader.read_until(b'\n', &mut buffer) {
                Ok(0) => {
                    println!("Connection to controller client ended");
                    break;
                },
                Ok(_) => {
                    let message = byte_slice_to_string(&buffer);
                    println!("RECV: {}", message.trim());
                    if let Ok(fsd_message) = fsd_interface::parse_message(message.trim()) {
                        match fsd_message {
                            FsdMessageType::AtcRegisterMessage(msg) => {
                                writer.write(&string_to_byte_slice(&TextMessage::new(SERVER_CALLSIGN, msg.from, WELCOME_MESSAGE).to_string())).ok();
                            },
                            FsdMessageType::MetarRequestMessage(msg) => {
                                if let Some(metar) = metar_provider.lookup_metar(&msg.station) {
                                    writer.write(&string_to_byte_slice(&MetarResponseMessage::new(SERVER_CALLSIGN, msg.from, metar).to_string())).ok();
                                }
                            }
                            _ => {},
                        }
                    }
                },
                Err(e) => {
                    println!("{:?}", e);
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