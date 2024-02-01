use std::{collections::HashMap, sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, time::Duration};

use crate::ui::Message;


const VATSIM_METARS_URL: &str = "https://metar.vatsim.net/metar.php?id=all";



#[derive(Clone)]
pub struct MetarProvider {
    metars: Arc<Mutex<HashMap<String, String>>>,
    last_update_successful: Arc<AtomicBool>,
}
impl MetarProvider {

    pub fn new() -> MetarProvider {
        MetarProvider {
            metars: Arc::new(Mutex::new(HashMap::new())),
            last_update_successful: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn last_update_successful(&self) -> bool {
        self.last_update_successful.load(Ordering::Relaxed)
    }

    pub fn update(&mut self) -> bool {
        let success = self.update_inner();
        self.last_update_successful.store(success, Ordering::Relaxed);
        success
    }

    pub fn lookup_metar(&self, station_id: &str) -> Option<String> {
        self.metars.lock().unwrap().get(station_id).clone().map(|x| x.to_owned())
    }

    fn update_inner(&mut self) -> bool {
        let mut map = HashMap::new();
        match ureq::get(VATSIM_METARS_URL).timeout(Duration::from_millis(500)).call().ok().and_then(|response| response.into_string().ok()) {
            Some(metar_file) => {
                for line in metar_file.lines() {
                    let icao = match line.split_whitespace().next() {
                        Some(icao) => icao.to_owned(),
                        None => continue,
                    };
                    if icao.len() < 4 { continue; }
                    map.insert(icao, line.to_owned());
                }
            },
            None => return false,
        }

        let mut lock = self.metars.lock().unwrap();
        *lock = map;
        return true;
    }
}