
use std::{collections::HashMap, sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, time::Duration};

use serde::Deserialize;
use serde_json::Value;

const VATSIM_DATA_URL: &str = "https://data.vatsim.net/v3/vatsim-data.json";

#[derive(Clone)]
pub struct VatsimDataProvider {
    vatsim_aircraft: Arc<Mutex<HashMap<String, VatsimAircraft>>>,
    last_update_successful: Arc<AtomicBool>,
}

impl VatsimDataProvider {
    pub fn new() -> VatsimDataProvider {
        VatsimDataProvider {
            vatsim_aircraft: Arc::new(Mutex::new(HashMap::new())),
            last_update_successful: Arc::new(AtomicBool::new(false)),
        }
    }
    pub fn get_aircraft_details(&self, callsign: &str) -> Option<Details> {
        let lock = self.vatsim_aircraft.lock().unwrap();
        lock.get(callsign).map(|aircraft| aircraft.details.clone())
    }

    pub fn last_update_successful(&self) -> bool {
        self.last_update_successful.load(Ordering::Relaxed)
    }

    pub fn update(&mut self) -> bool {
        let success = self.update_inner();
        self.last_update_successful.store(success, Ordering::Relaxed);
        success
    }

    fn update_inner(&mut self) -> bool {
        let res = match ureq::get(VATSIM_DATA_URL).timeout(Duration::from_secs(1)).call() {
            Ok(res) => res,
            Err(_) => return false,
        };
        let json = match res.into_json::<Value>() {
            Ok(json) => json,
            Err(_) => return false,
        };

        let pilots = match json.get("pilots").and_then(|pilots| pilots.as_array()) {
            Some(pilots) => pilots,
            None => return false,
        };

        for value in pilots {
            let new = match serde_json::from_value::<Details>(value.clone()) {
                Ok(details) => details,
                Err(_) => continue,
            };

            let mut details_map = self.vatsim_aircraft.lock().unwrap();
            if let Some(existing_aircraft) = details_map.get_mut(&new.callsign) {
                existing_aircraft.update(new);
            } else {
                let callsign = new.callsign.clone();
                let new_record = VatsimAircraft::new(new);
                details_map.insert(callsign, new_record);
            }
        };

        return true;
    }

    pub fn get_details_and_flight_plan_to_send(&mut self, callsign: &str) -> Option<(Details, Option<FlightPlan>)> {
        let mut map = self.vatsim_aircraft.lock().unwrap();
        map.get_mut(callsign).map(|vatsim_aircraft| vatsim_aircraft.get_details_and_flight_plan_to_send())
    }
}


struct VatsimAircraft {
    last_sent_flight_plan_revision_id: Option<usize>,
    unsent_flight_plan: Option<FlightPlan>,
    details: Details,
}
impl VatsimAircraft {
    fn new(mut details: Details) -> VatsimAircraft {
        let unsent_flight_plan = details.flight_plan.take();
        VatsimAircraft {
            last_sent_flight_plan_revision_id: None,
            unsent_flight_plan,
            details
        }
    }

    fn update(&mut self, new_details: Details) {
        self.details = new_details;
        let new_unsent_flight_plan = if let Some(new_flight_plan) = self.details.flight_plan.clone() {
            match self.last_sent_flight_plan_revision_id {
                None => Some(new_flight_plan),
                Some(last_sent_flight_plan_revision_id) => {
                    if last_sent_flight_plan_revision_id > new_flight_plan.revision_id {
                        self.last_sent_flight_plan_revision_id = Some(new_flight_plan.revision_id);
                        Some(new_flight_plan)
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        };
        self.unsent_flight_plan = new_unsent_flight_plan;
    }

    fn get_details_and_flight_plan_to_send(&mut self) -> (Details, Option<FlightPlan>) {
        let details = self.details.clone();
        let flight_plan_to_send = self.unsent_flight_plan.take();
        self.last_sent_flight_plan_revision_id = flight_plan_to_send.as_ref().map(|fp| fp.revision_id);
        (details, flight_plan_to_send)
    }

    pub fn get_flight_plan(&self) -> Option<FlightPlan> {
        self.details.flight_plan.clone()
    }
    pub fn get_details(&self) -> &Details {
        &self.details
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Details {
    pub cid: i32,
    pub name: String,
    pub callsign: String,
    pub transponder: String,
    pub altitude: i32,
    pub heading: u32,
    pub qnh_i_hg: f32,
    pub flight_plan: Option<FlightPlan>,
}


#[derive(Debug, Deserialize, Clone)]
pub struct FlightPlan {
    pub flight_rules: FlightRules,
    #[serde(rename = "aircraft_faa")]
    pub aircraft_type: String,
    #[serde(rename = "departure")]
    pub departure_icao: String,
    #[serde(rename = "arrival")]
    pub arrival_icao: String,
    #[serde(rename = "alternate")]
    pub alternate_icao: String,
    pub cruise_tas: String,
    altitude: String,
    #[serde(rename = "deptime")]
    departure_time: String,
    enroute_time: String,
    fuel_time: String,
    pub remarks: String,
    pub route: String,
    pub revision_id: usize,
    pub assigned_transponder: String,
}
impl FlightPlan {
    pub fn altitude(&self) -> i32 {
        match self.altitude.parse::<i32>() {
            Ok(alt) => return alt,
            Err(_) if self.altitude.starts_with("FL") && self.altitude.len() > 2 => {
                match self.altitude[2..].parse::<i32>() {
                    Ok(alt) => return alt * 100,
                    Err(_) => return 0,
                }
            },
            Err(_) => return 0,
        }
    }
    pub fn departure_time(&self) -> (u8, u8) {
        let hours = self.departure_time.get(0..2).and_then(|hours| hours.parse::<u8>().ok()).unwrap_or_default();
        let mins = self.departure_time.get(2..2).and_then(|mins| mins.parse::<u8>().ok()).unwrap_or_default();
        (hours, mins)
    }
    pub fn enroute_time(&self) -> (u8, u8) {
        let hours = self.enroute_time.get(0..2).and_then(|hours| hours.parse::<u8>().ok()).unwrap_or_default();
        let mins = self.enroute_time.get(2..2).and_then(|mins| mins.parse::<u8>().ok()).unwrap_or_default();
        (hours, mins)
    }
    pub fn fuel_time(&self) -> (u8, u8) {
        let hours = self.fuel_time.get(0..2).and_then(|hours| hours.parse::<u8>().ok()).unwrap_or_default();
        let mins = self.fuel_time.get(2..2).and_then(|mins| mins.parse::<u8>().ok()).unwrap_or_default();
        (hours, mins)
    }
}

impl From<FlightPlan> for fsd_interface::FlightPlan {
    fn from(value: FlightPlan) -> Self {
        let (hours_enroute, mins_enroute) = value.enroute_time();
        let (hours_fuel, mins_fuel) = value.fuel_time();
        let alt = value.altitude() as u32;
        Self {
            flight_rules: match value.flight_rules {
                FlightRules::DVFR => fsd_interface::FlightRules::DVFR,
                FlightRules::VFR => fsd_interface::FlightRules::VFR,
                FlightRules::SVFR => fsd_interface::FlightRules::SVFR,
                FlightRules::IFR => fsd_interface::FlightRules::IFR,
            },
            ac_type: value.aircraft_type,
            filed_tas: value.cruise_tas.parse().unwrap_or(0),
            origin: value.departure_icao,
            etd: value.departure_time.parse().unwrap_or(0),
            atd: value.departure_time.parse().unwrap_or(0),
            cruise_level: alt,
            destination: value.arrival_icao,
            hours_enroute,
            mins_enroute,
            hours_fuel,
            mins_fuel,
            alternate: value.alternate_icao,
            remarks: value.remarks,
            route: value.route,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub enum FlightRules {
    #[serde(rename = "D")]
    DVFR,
    #[serde(rename = "V")]
    VFR,
    #[serde(rename = "S")]
    SVFR,
    #[serde(rename = "I")]
    IFR,
}