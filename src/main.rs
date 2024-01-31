use std::{process, ptr};
use win32_ui_impl::{util, window, Win32Ui};
use windows_sys::{w, Win32::{System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR}}};

mod core;
mod ui;
mod win32_ui_impl;



fn main() {
    
    unsafe {
        let hwnd = match window::setup_window(GetModuleHandleW(ptr::null())) {
            Ok(ui) => ui,
            Err(err_string) => {
                let err_string_wide = util::wide_null(err_string);
                MessageBoxW(0, err_string_wide.as_ptr(), w!("Traffic Viewer"), MB_ICONERROR);
                process::exit(1);
            },
        };
        
        Win32Ui::run(hwnd);
    }
}