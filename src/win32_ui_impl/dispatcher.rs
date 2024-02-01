use windows_sys::Win32::{Foundation::HWND, UI::WindowsAndMessaging::PostMessageW};

use crate::ui::{Message, Ui};

use super::consts::UI_MESSAGE;

#[derive(Clone, Copy)]
pub struct MessageDispatcher {
    hwnd: HWND,
}
impl MessageDispatcher {
    pub fn new(hwnd: HWND) -> MessageDispatcher {
        MessageDispatcher { hwnd }
    }
}
impl Ui for MessageDispatcher {
    fn dispatch_message(&self, message: crate::ui::Message) {
        let mut lparam = 0;
        let wparam = match message {
            Message::MsfsConnected => UiMessage::MsfsConnected,
            Message::MsfsDisconnected => UiMessage::MsfsDisconnected,
            Message::EuroscopeConnected(callsign) => {
                lparam = Box::into_raw(Box::new(callsign)) as isize;
                UiMessage::EuroscopeConnected
            },
            Message::EuroscopeDisconnected => UiMessage::EuroscopeDisconnected,
            Message::MetarsRetrieved => UiMessage::MetarsRetrieved,
            Message::MetarsDisconnected => UiMessage::MetarsDisconnected,
            Message::MetarRetrieved(metar) => {
                lparam = Box::into_raw(Box::new(metar)) as isize;
                UiMessage::MetarRetrieved
            },
            Message::MetarNotFound => UiMessage::MetarNotFound,
            Message::VatsimDataRetrieved => UiMessage::VatsimDataRetrieved,
            Message::VatsimDataDisconnected => UiMessage::VatsimDataDisconnected,
        } as usize;
        unsafe {
            PostMessageW(self.hwnd, UI_MESSAGE, wparam, lparam);
        }
        
    }
}



#[repr(usize)]
pub enum UiMessage {
    MsfsConnected,
    MsfsDisconnected,

    EuroscopeConnected,
    EuroscopeDisconnected,
    
    MetarsRetrieved,
    MetarsDisconnected,

    MetarNotFound,
    MetarRetrieved,

    VatsimDataRetrieved,
    VatsimDataDisconnected,
}
// Euroscope connected
// 
