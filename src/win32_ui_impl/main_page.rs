use windows_sys::Win32::{Foundation::GetLastError, UI::{Controls::{CheckDlgButton, BST_CHECKED, BST_UNCHECKED, EM_SETLIMITTEXT, EM_SETREADONLY}, Input::KeyboardAndMouse::EnableWindow, WindowsAndMessaging::{GetDlgItem, GetWindowLongPtrW, SendMessageW, SetWindowLongPtrW, ES_UPPERCASE, GWL_STYLE, WM_GETTEXT, WM_GETTEXTLENGTH, WM_SETTEXT}}};

use super::{consts::{RES_CALLSIGN_EDITTEXT, RES_FETCH_FPS_FROM_VS_CHECKBOX, RES_FETCH_METARS_FROM_VS_CHECKBOX, RES_FETCH_METAR_PUSHBUTTON, RES_METAR_STATION_EDITTEXT, RES_ONLY_SHOW_VS_AC_CHECKBOX, RES_SYNC_WITH_ES_CHECKBOX}, util};




pub struct MainPage {
    main_hwnd: isize,
    euroscope_callsign: Option<String>,

    callsign_input_hwnd: isize,
    metar_station_hwnd: isize,
    fetch_metar_pushbutton_hwnd: isize,
}
impl MainPage {
    pub unsafe fn new() -> MainPage {
        
        MainPage { main_hwnd: 0, euroscope_callsign: None, callsign_input_hwnd: 0, metar_station_hwnd: 0, fetch_metar_pushbutton_hwnd: 0 }
        
        

    }

    pub unsafe fn init(&mut self, main_hwnd: isize) {
        println!("Main hwnd: {}", main_hwnd);
        self.main_hwnd = main_hwnd;
        self.callsign_input_hwnd = GetDlgItem(main_hwnd, RES_CALLSIGN_EDITTEXT as i32);
        println!("Err: {}", GetLastError());
        println!("CS: {}", self.callsign_input_hwnd);
        self.metar_station_hwnd = GetDlgItem(main_hwnd, RES_METAR_STATION_EDITTEXT as i32);
        self.fetch_metar_pushbutton_hwnd = GetDlgItem(main_hwnd, RES_FETCH_METAR_PUSHBUTTON as i32);


        // Set max lengths and uppercase only
        SendMessageW(self.callsign_input_hwnd, EM_SETLIMITTEXT, 10, 0);
        let current_style = GetWindowLongPtrW(self.callsign_input_hwnd, GWL_STYLE);
        SetWindowLongPtrW(self.callsign_input_hwnd, GWL_STYLE, current_style | ES_UPPERCASE as isize);

        SendMessageW(self.metar_station_hwnd, EM_SETLIMITTEXT, 4, 0);
        let current_style = GetWindowLongPtrW(self.metar_station_hwnd, GWL_STYLE);
        SetWindowLongPtrW(self.metar_station_hwnd, GWL_STYLE, current_style | ES_UPPERCASE as isize);

        // Set initial UI control state
        self.set_callsign_input_enabled(false);
        self.set_callsign_input_text("EZY1234");
        
        self.set_sync_with_es_checkbox(true);
        self.set_fetch_flight_plans_checkbox(true);
        self.set_fetch_metars_checkbox(true);
        self.set_only_show_vs_ac_checkbox(true);

        self.set_metar_station_input_enabled(false);
        self.set_metar_station_input_text("EGKK");
        self.set_metar_button_enabled(false);
    }
    

    pub unsafe fn set_metar_button_enabled(&mut self, enabled: bool) {
        let enabled = if enabled { 1 } else { 0 };
        EnableWindow(self.fetch_metar_pushbutton_hwnd, enabled);
    }

    pub unsafe fn set_sync_with_es_checkbox(&mut self, checked: bool) {
        self.set_checkbox_state(RES_SYNC_WITH_ES_CHECKBOX, checked);
    }
    pub unsafe fn set_fetch_metars_checkbox(&mut self, checked: bool) {
        self.set_checkbox_state(RES_FETCH_METARS_FROM_VS_CHECKBOX, checked);
    }
    pub unsafe fn set_only_show_vs_ac_checkbox(&mut self, checked: bool) {
        self.set_checkbox_state(RES_ONLY_SHOW_VS_AC_CHECKBOX, checked);
    }
    pub unsafe fn set_fetch_flight_plans_checkbox(&mut self, checked: bool) {
        self.set_checkbox_state(RES_FETCH_FPS_FROM_VS_CHECKBOX, checked);
    }

    unsafe fn set_checkbox_state(&mut self, checkbox_id: u32, checked: bool) {
        let ucheck = if checked { BST_CHECKED } else { BST_UNCHECKED };
        CheckDlgButton(self.main_hwnd, checkbox_id as i32, ucheck);
    }

    pub unsafe fn set_callsign_input_enabled(&mut self, enabled: bool) {
        self.set_text_edit_enabled(self.callsign_input_hwnd, enabled);
    }
    pub unsafe fn set_metar_station_input_enabled(&mut self, enabled: bool) {
        self.set_text_edit_enabled(self.metar_station_hwnd, enabled);
    }
    unsafe fn set_text_edit_enabled(&mut self, text_edit_hwnd: isize, enabled: bool) {
        let checked = if enabled { 1 } else { 0 };
        EnableWindow(text_edit_hwnd, checked);
    }
    unsafe fn set_text_edit_text(&mut self, text_edit_hwnd: isize, text: &str) {
        let text = util::wide_null(text);
        SendMessageW(text_edit_hwnd, WM_SETTEXT, 0, text.as_ptr() as isize);
    }
    unsafe fn get_text_edit_text(&self, text_edit_hwnd: isize) -> String {
        let length = (SendMessageW(text_edit_hwnd, WM_GETTEXTLENGTH, 0, 0) + 1) as usize;
        let mut buffer = vec![0_u16; length];
        let actual_length = (SendMessageW(text_edit_hwnd, WM_GETTEXT, length, buffer.as_mut_ptr() as isize)) as usize;
        buffer.truncate(actual_length);
        String::from_utf16_lossy(&buffer)
    }
    pub unsafe fn set_metar_station_input_text(&mut self, text: &str) {
        self.set_text_edit_text(self.metar_station_hwnd, text);
    }
    pub unsafe fn set_callsign_input_text(&mut self, text: &str) {
        self.set_text_edit_text(self.callsign_input_hwnd, text);
    }
    pub unsafe fn get_metar_station_input_text(&self) -> String {
        self.get_text_edit_text(self.metar_station_hwnd)
    }
    pub unsafe fn get_callsign_input_text(&self) -> String {
        self.get_text_edit_text(self.callsign_input_hwnd)
    }
}