use std::{sync::{atomic::{AtomicBool, Ordering}, mpsc::{Receiver, Sender, TryRecvError}, Arc}, thread::{self, JoinHandle}, time::Duration};

use crate::ui::Ui;

use super::{metar::MetarProvider, vatsim::VatsimDataProvider, Preferences};

pub fn worker_thread<U: Ui>(should_terminate: Arc<AtomicBool>, preferences: Preferences, ui_link: U, mut metar_provider: MetarProvider, mut vatsim_data_provider: VatsimDataProvider) -> JoinHandle<()> {
    thread::Builder::new().name("TrafficViewerWorkerThread".into()).spawn(move || {
        for i in 0..usize::MAX {
            if should_terminate.load(Ordering::Relaxed) { break };

            let metar_refresh_due = preferences.fetch_metars() && if metar_provider.last_update_successful() { i % 120 == 0 } else { true };
            if metar_refresh_due {
                metar_provider.update();
            }

            let vatsim_data_refresh_due = preferences.fetch_flight_plans() && if vatsim_data_provider.last_update_successful() { i % 15 == 0 } else { true };
            if vatsim_data_refresh_due {
                vatsim_data_provider.update();
            }
            
            thread::sleep(Duration::from_secs(1));
        }
    }).unwrap()
}

pub enum Message {

}