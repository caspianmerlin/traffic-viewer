use std::mem;

use windows_sys::Win32::{Foundation::HWND, Graphics::Gdi::UpdateWindow, UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, IsDialogMessageW, SendMessageW, ShowWindow, TranslateMessage, MSG, SW_SHOW}};

use crate::win32_ui_impl::status_bar::StatusBar;

use self::{consts::INIT_MESSAGE, main_page::MainPage};



mod dispatcher;
mod consts;
pub mod util;
pub mod window;
mod status_bar;
mod main_page;
mod about_page;



pub struct Win32Ui {
    hinst: isize,
    main_hwnd: HWND,
    status_bar: StatusBar,
    main_page: MainPage,
}

impl Win32Ui {
    pub unsafe fn new(hinst: isize, main_hwnd: isize) -> Win32Ui {
        let status_bar = StatusBar::new();
        let main_page = MainPage::new();
        Win32Ui {
            hinst,
            main_hwnd,
            status_bar,
            main_page,
        } 
        
    }
    pub unsafe fn init(&mut self, hinst: isize, parent_hwnd: isize) {
        self.status_bar.init(hinst, parent_hwnd);
        self.main_page.init(parent_hwnd);
    }
    pub unsafe fn run(hwnd: isize) {
        SendMessageW(hwnd, INIT_MESSAGE, 0, 0);
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        let mut msg: MSG = mem::zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            if IsDialogMessageW(hwnd, &msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}
