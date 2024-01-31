use windows_sys::Win32::{Foundation::HWND, UI::WindowsAndMessaging::PostMessageW};

use crate::ui::Ui;

use super::consts::UI_MESSAGE;

#[derive(Clone)]
pub struct MessageDispatcher {
    hwnd: HWND,
}
impl Ui for MessageDispatcher {
    fn dispatch_message(&self, message: crate::ui::Message) {
        let boxed_message_raw = Box::into_raw(Box::new(message));
        unsafe {
            PostMessageW(self.hwnd, UI_MESSAGE, 0, boxed_message_raw as isize);
        }
        
    }
}