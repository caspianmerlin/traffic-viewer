use std::{collections::HashMap, sync::{atomic::{AtomicBool, Ordering}, mpsc::Sender, Arc, Mutex}, thread::JoinHandle};

use crate::ui::{Message, Ui};

use self::{fsd::Server, metar::MetarProvider, vatsim::VatsimDataProvider};

mod worker;
mod fsd;
mod metar;
mod vatsim;
mod fsuipc;
pub struct App<U: Ui> {
    thread: Option<JoinHandle<()>>,
    fsd: Server,
    metar_provider: MetarProvider,
    vatsim_data_provider: VatsimDataProvider,
    preferences: Preferences,
    should_terminate: Arc<AtomicBool>,
    ui_link: U
}
impl<U> App<U> where U: Ui + 'static {
    pub fn new(preferences: Preferences, ui_link: U) -> Self {
        let metar_provider = MetarProvider::new();
        let vatsim_data_provider = VatsimDataProvider::new();
        let should_terminate = Arc::new(AtomicBool::new(false));
        let fsd = Server::new(preferences.clone(), vatsim_data_provider.clone(), metar_provider.clone(), ui_link.clone(), Arc::clone(&should_terminate));
        let thread = Some(worker::worker_thread(Arc::clone(&should_terminate), preferences.clone(), ui_link.clone(), metar_provider.clone(), vatsim_data_provider.clone(), fsd.sender()));
        Self { thread, fsd, metar_provider, vatsim_data_provider, preferences, should_terminate, ui_link }
    }
    pub fn try_lookup_metar(&self, station_id: String) {
        let message = match self.metar_provider.lookup_metar(&station_id) {
            Some(metar) => Message::MetarRetrieved(metar),
            None => Message::MetarNotFound,
        };
        self.ui_link.dispatch_message(message);
    }
}
impl<U> Drop for App<U> where U: Ui {
    fn drop(&mut self) {
        self.should_terminate.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap();
        }
    }
}




#[derive(Clone)]
pub struct Preferences {
    own_callsign: Arc<Mutex<String>>,
    es_callsign: Arc<Mutex<String>>,
    use_es_callsign: Arc<AtomicBool>,
    fetch_metars: Arc<AtomicBool>,
    fetch_flight_plans: Arc<AtomicBool>,
    only_show_vatsim: Arc<AtomicBool>,
}
impl Preferences {
    pub fn new(use_es_callsign: bool, fetch_metars: bool, fetch_flight_plans: bool, only_show_vatsim: bool) -> Preferences {
        Preferences {
            own_callsign: Arc::new(Mutex::new(String::with_capacity(10))),
            es_callsign: Arc::new(Mutex::new(String::new())),
            use_es_callsign: Arc::new(AtomicBool::new(use_es_callsign)),
            fetch_metars: Arc::new(AtomicBool::new(fetch_metars)),
            fetch_flight_plans: Arc::new(AtomicBool::new(fetch_flight_plans)),
            only_show_vatsim: Arc::new(AtomicBool::new(only_show_vatsim)),
        }
    }
    pub fn own_callsign(&self) -> Option<String> {
        let own_callsign = self.own_callsign.lock().unwrap();
        return if own_callsign.is_empty() {
            None
        } else {
            Some(own_callsign.clone())
        };
    }
    pub fn es_callsign(&self) -> Option<String> {
        let es_callsign = self.es_callsign.lock().unwrap();
        return if es_callsign.is_empty() {
            None
        } else {
            Some(es_callsign.clone())
        };
    }
    pub fn use_es_callsign(&self) -> bool {
        self.use_es_callsign.load(Ordering::Relaxed)
    }
    pub fn fetch_metars(&self) -> bool {
        self.fetch_metars.load(Ordering::Relaxed)
    }
    pub fn fetch_flight_plans(&self) -> bool {
        self.fetch_flight_plans.load(Ordering::Relaxed)
    }
    pub fn only_show_vatsim(&self) -> bool {
        self.only_show_vatsim.load(Ordering::Relaxed)
    }

    pub fn set_own_callsign(&mut self, callsign: String) {
        let mut own_callsign = self.own_callsign.lock().unwrap();
        *own_callsign = callsign;
    }
    pub fn set_es_callsign(&mut self, callsign: String) {
        let mut es_callsign = self.es_callsign.lock().unwrap();
        *es_callsign = callsign;
    }
    pub fn set_use_es_callsign(&self, val: bool) {
        self.use_es_callsign.store(val, Ordering::Relaxed)
    }
    pub fn set_fetch_metars(&self, val: bool) {
        self.fetch_metars.store(val, Ordering::Relaxed)
    }
    pub fn set_fetch_flight_plans(&self, val: bool) {
        self.fetch_flight_plans.store(val, Ordering::Relaxed)
    }
    pub fn set_only_show_vatsim(&self, val: bool) {
        self.only_show_vatsim.store(val, Ordering::Relaxed)
    }
}