use std::{ffi::CStr, ops::{Div, Mul}, sync::{atomic::{AtomicBool, Ordering}, mpsc::{Receiver, Sender, TryRecvError}, Arc}, thread::{self, JoinHandle}, time::Duration};

use fsd_interface::{messages::{FlightPlanMessage, PilotPositionUpdateMessage}, PilotRating, TransponderCode, TransponderMode};

use crate::ui::{Message, Ui};

use super::{fsuipc, metar::MetarProvider, vatsim::VatsimDataProvider, Preferences};

const FLIGHT_PLAN_RECIPIENT: &str = "A*";
const HDG_FACTOR: f32 = 182.044444444;

pub fn worker_thread<U: Ui + 'static>(should_terminate: Arc<AtomicBool>, preferences: Preferences, ui_link: U, mut metar_provider: MetarProvider, mut vatsim_data_provider: VatsimDataProvider, msg_sender: Sender<String>) -> JoinHandle<()> {
    thread::Builder::new().name("TrafficViewerWorkerThread".into()).spawn(move || {
        for i in 0..usize::MAX {
            if should_terminate.load(Ordering::Relaxed) { break };

            let metar_refresh_due = preferences.fetch_metars() && if metar_provider.last_update_successful() { i % 120 == 0 } else { true };
            if metar_refresh_due {
                let message = if metar_provider.update() {
                    Message::MetarsRetrieved
                } else {
                    Message::MetarsDisconnected
                };
                ui_link.dispatch_message(message);
            }

            let vatsim_data_refresh_due = preferences.fetch_flight_plans() && if vatsim_data_provider.last_update_successful() { i % 15 == 0 } else { true };
            if vatsim_data_refresh_due {
                let message = if vatsim_data_provider.update() {
                    Message::VatsimDataRetrieved
                } else {
                    Message::VatsimDataDisconnected
                };
                ui_link.dispatch_message(message);
            }

            let aircraft_refresh_due = i % 4 == 0;
            if aircraft_refresh_due {

                // Aircraft
   
                if let Ok(aircraft_list) = fsuipc::get_aircraft(true).and_then(|ground_aircraft| fsuipc::get_aircraft(true).map(|airborne_aircraft| ground_aircraft.into_iter().chain(airborne_aircraft.into_iter()))) {
                    ui_link.dispatch_message(Message::MsfsConnected);
                    for tcas_data in aircraft_list {
                        let callsign = CStr::from_bytes_until_nul(&tcas_data.atc_id).unwrap();
                        let callsign = match callsign.to_str() {
                            Ok(cs) => cs,
                            Err(_) => continue,
                        };

                        let vatsim_details = if preferences.fetch_flight_plans() {
                            vatsim_data_provider.get_details_and_flight_plan_to_send(callsign)
                        } else {
                            None
                        };
                        let (pos_rep, fp_update) = match vatsim_details {
                            None => {
                                if preferences.only_show_vatsim() { continue }
                                let pos_rep = PilotPositionUpdateMessage::new(callsign, TransponderMode::ModeC, TransponderCode::try_from(2000).unwrap(), PilotRating::Student, tcas_data.lat as f64, tcas_data.lon as f64, tcas_data.alt as f64, tcas_data.alt as f64, tcas_data.gs as u32, 0.0, 0.0, (tcas_data.hdg as f64 / HDG_FACTOR as f64).floor(), false);
                                (pos_rep, None)
                            },
                            Some((details, flight_plan)) => {
                                let alt_diff = ((details.qnh_i_hg - 29.92).mul(100.0).round().div(100.0) * 1000.0) as f64;
                                let position = PilotPositionUpdateMessage::new(callsign, TransponderMode::ModeC, TransponderCode::try_from(details.transponder.parse::<u16>().unwrap_or_default()).unwrap_or(TransponderCode::try_from(2000).unwrap()), PilotRating::Student, tcas_data.lat as f64, tcas_data.lon as f64, tcas_data.alt as f64, tcas_data.alt as f64 - alt_diff, tcas_data.gs as u32, 0.0, 0.0, (tcas_data.hdg as f64 / HDG_FACTOR as f64).floor(), false);
                                let flight_plan = flight_plan.map(|fp| fsd_interface::FlightPlan::from(fp));
                                let fp_update = flight_plan.map(|fp| FlightPlanMessage::new(FLIGHT_PLAN_RECIPIENT, callsign, fp));
                                (position, fp_update)
                            },
                        };

                        msg_sender.send(pos_rep.to_string()).ok();
                        if let Some(flight_plan) = fp_update.map(|fp| fp.to_string()) {
                            msg_sender.send(flight_plan).ok();
                        }
                    };
                } else {
                    ui_link.dispatch_message(Message::MsfsDisconnected);
                }
                


            // Own aircraft
                if let Ok(own_aircraft_data) = fsuipc::get_own_aircraft_data() {
                    ui_link.dispatch_message(Message::MsfsConnected);
                    let my_callsign = if preferences.use_es_callsign() {
                        preferences.es_callsign()
                    } else {
                        preferences.own_callsign()
                    }.unwrap_or_else(|| String::from("ME"));

                    let vatsim_details = if preferences.fetch_flight_plans() {
                        vatsim_data_provider.get_details_and_flight_plan_to_send(&my_callsign)
                    } else {
                        None
                    };
                    let (pos_rep, fp_update) = match vatsim_details {
                        None => {
                            let pos_rep = PilotPositionUpdateMessage::new(my_callsign, TransponderMode::ModeC, TransponderCode::try_from(2000).unwrap(), PilotRating::Student, own_aircraft_data.lat, own_aircraft_data.lon, own_aircraft_data.alt, own_aircraft_data.alt, own_aircraft_data.gs as u32, 0.0, 0.0, own_aircraft_data.true_hdg, false);
                            (pos_rep, None)
                        },
                        Some((details, flight_plan)) => {
                            let alt_diff = ((details.qnh_i_hg - 29.92).mul(100.0).round().div(100.0) * 1000.0) as f64;
                            let position = PilotPositionUpdateMessage::new(my_callsign.clone(), TransponderMode::ModeC, TransponderCode::try_from(details.transponder.parse::<u16>().unwrap_or_default()).unwrap_or(TransponderCode::try_from(2000).unwrap()), PilotRating::Student, own_aircraft_data.lat, own_aircraft_data.lon, own_aircraft_data.alt, own_aircraft_data.alt - alt_diff, own_aircraft_data.gs as u32, 0.0, 0.0, own_aircraft_data.true_hdg, false);
                            let flight_plan = flight_plan.map(|fp| fsd_interface::FlightPlan::from(fp));
                            let fp_update = flight_plan.map(|fp| FlightPlanMessage::new(FLIGHT_PLAN_RECIPIENT, my_callsign, fp));
                            (position, fp_update)
                        },
                    };

                    msg_sender.send(pos_rep.to_string()).ok();
                    if let Some(flight_plan) = fp_update.map(|fp| fp.to_string()) {
                        msg_sender.send(flight_plan).ok();
                    }
                } else {
                    ui_link.dispatch_message(Message::MsfsDisconnected);
                }

            }


            
            thread::sleep(Duration::from_secs(1));
        }
    }).unwrap()
}
