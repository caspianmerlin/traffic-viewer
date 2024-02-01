use std::{mem, ptr};

use windows_sys::{w, Win32::{Foundation::{HWND, RECT}, Graphics::Gdi::{GetSysColorBrush, COLOR_3DFACE}, System::LibraryLoader::GetModuleHandleW, UI::{Controls::DRAWITEMSTRUCT, Input::KeyboardAndMouse::{GetFocus, IsWindowEnabled}, WindowsAndMessaging::{CreateDialogParamW, CreateWindowExW, DefWindowProcW, DestroyWindow, GetDlgItem, GetWindowLongPtrW, GetWindowRect, LoadCursorW, PostQuitMessage, RegisterClassExW, SendMessageW, SetWindowLongPtrW, SetWindowPos, BM_CLICK, CW_USEDEFAULT, DLGWINDOWEXTRA, EN_CHANGE, GWLP_USERDATA, IDC_ARROW, IDOK, SWP_NOSIZE, SWP_NOZORDER, WM_CLOSE, WM_COMMAND, WM_CREATE, WM_DESTROY, WM_DRAWITEM, WM_NCCREATE, WNDCLASSEXW}}}};

use crate::{core::{App, Preferences}, win32_ui_impl::{consts::{MAIN_DIALOG_CLASS_NAME, RES_MAIN_DIALOG, RES_MENU_MAIN}, util}};

use super::{about_page, consts::{INIT_MESSAGE, RES_FETCH_FPS_FROM_VS_CHECKBOX, RES_FETCH_METARS_FROM_VS_CHECKBOX, RES_FETCH_METAR_PUSHBUTTON, RES_MENU_MAIN_FILE_EXIT, RES_MENU_MAIN_HELP_ABOUT, RES_METAR_STATION_EDITTEXT, RES_CALLSIGN_EDITTEXT, RES_ONLY_SHOW_VS_AC_CHECKBOX, RES_SYNC_WITH_ES_CHECKBOX, UI_MESSAGE}, dispatcher::{MessageDispatcher, UiMessage}, Win32Ui};

pub unsafe fn setup_window(hinst: isize) -> Result<HWND, String> {
    register_window_class(hinst)?;
    create_main_window(hinst)
}

unsafe fn register_window_class(hinst: isize) -> Result<(), String> {
    let mut wnd_class: WNDCLASSEXW = mem::zeroed();
    wnd_class.cbSize = mem::size_of::<WNDCLASSEXW>() as u32;
    wnd_class.lpfnWndProc = Some(wnd_proc);
    wnd_class.hCursor = LoadCursorW(0, IDC_ARROW);
    wnd_class.hbrBackground = GetSysColorBrush(COLOR_3DFACE);
    wnd_class.lpszClassName = MAIN_DIALOG_CLASS_NAME;
    wnd_class.cbWndExtra = DLGWINDOWEXTRA as i32;
    wnd_class.hInstance = hinst;
    wnd_class.lpszMenuName = util::make_int_resource(RES_MENU_MAIN);

    return if RegisterClassExW(&wnd_class) != 0 {
        Ok(())
    } else {
        Err("Unable to register window class".into())
    };
}

unsafe fn create_main_window(hinst: isize) -> Result<HWND, String> {
    let hwnd = CreateDialogParamW(hinst, util::make_int_resource(RES_MAIN_DIALOG), 0, None, 0);
    return if hwnd == 0 {
        Err("Unable to create main window".into())
    } else {
        Ok(hwnd)
    };
}

unsafe extern "system" fn wnd_proc(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize {
    match msg {
        WM_NCCREATE => {
            let app = App::new(Preferences::new(true, true, true, true), MessageDispatcher::new(hwnd));
            let ui_boxed = Box::into_raw(Box::new(Win32Ui::new(GetModuleHandleW(ptr::null()), hwnd, app)));
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, ui_boxed as isize);
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        },
        WM_CREATE => {
            let dummy_hwnd = CreateWindowExW(0, w!("STATIC"), w!("STATIC"), 0, CW_USEDEFAULT, CW_USEDEFAULT, 0, 0, 0, 0, GetModuleHandleW(ptr::null()), ptr::null());
            let mut rect: RECT = mem::zeroed();
            GetWindowRect(dummy_hwnd, &mut rect);
            DestroyWindow(dummy_hwnd);
            SetWindowPos(hwnd, 0, rect.left, rect.top, 0, 0, SWP_NOSIZE | SWP_NOZORDER);
            return 0;
        },

        INIT_MESSAGE => {
            let ui = &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Win32Ui);
            ui.init(ui.hinst, hwnd);
            0
        },
        UI_MESSAGE => {
            let ui = &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Win32Ui);
            let message_type: UiMessage = mem::transmute(wparam);
            match message_type {
                UiMessage::MsfsConnected => ui.status_bar.set_msfs_connected(true),
                UiMessage::MsfsDisconnected => ui.status_bar.set_msfs_connected(false),
                UiMessage::EuroscopeConnected => {
                    let es_callsign = *Box::from_raw(lparam as *mut String);
                    ui.main_page.euroscope_callsign = Some(es_callsign.clone());
                    if ui.main_page.get_sync_with_es_checkbox() {
                        ui.main_page.set_callsign_input_text(&es_callsign);
                    }
                    ui.status_bar.set_euroscope_connected(true);
                }
                UiMessage::EuroscopeDisconnected => {
                    ui.main_page.euroscope_callsign = None;
                    if ui.main_page.get_sync_with_es_checkbox() {
                        ui.main_page.set_callsign_input_text("");
                    }
                    ui.status_bar.set_euroscope_connected(false);
                }
                UiMessage::MetarsRetrieved => {
                    ui.status_bar.set_metars_connected(true);
                    ui.main_page.set_metar_button_enabled(true);
                    ui.main_page.set_metar_station_input_enabled(true);
                    ui.main_page.set_metar_station_input_text("");
                },
                    
                   
                UiMessage::MetarsDisconnected => {
                    ui.status_bar.set_metars_connected(false);
                    ui.main_page.set_metar_button_enabled(false);
                    ui.main_page.set_metar_station_input_text("");
                    ui.main_page.set_metar_station_input_enabled(false);
                    ui.main_page.set_metar_text("");
                    
                },
                UiMessage::VatsimDataRetrieved => ui.status_bar.set_vatsim_connected(true),
                UiMessage::VatsimDataDisconnected => ui.status_bar.set_vatsim_connected(false),

                UiMessage::MetarRetrieved => {
                    let metar = *Box::from_raw(lparam as *mut String);
                    ui.main_page.set_metar_text(&metar);
                },
                UiMessage::MetarNotFound => {
                    ui.main_page.set_metar_text("METAR not found");
                }
                _ => {},
            }
            0
        }
        WM_DESTROY => {
            let ui = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Win32Ui;
            let ui_box = Box::from_raw(ui);
            drop(ui_box);
            PostQuitMessage(0);
            0
        },

        WM_DRAWITEM => {
            let ui = &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Win32Ui);
            let dis = &*(lparam as *const DRAWITEMSTRUCT);
            return if !ui.status_bar.draw(&dis) {
                0
            } else {
                1
            };
        },


        WM_COMMAND => {
            
            let (hi, lo) = util::split_words(wparam as u32);
            if hi == 0 {
                match lo as u32 {
                    RES_MENU_MAIN_FILE_EXIT => {
                        SendMessageW(hwnd, WM_CLOSE, 0, 0);
                        return 0;
                    },
                    RES_MENU_MAIN_HELP_ABOUT => {
                        let ui = &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Win32Ui);
                        about_page::show_about_window(ui.hinst, hwnd);
                        return 0;
                    },
                    RES_SYNC_WITH_ES_CHECKBOX => {
                        let ui = &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Win32Ui);
                        let checked = ui.main_page.get_sync_with_es_checkbox();
                        ui.app.preferences.set_use_es_callsign(checked);
                        ui.main_page.set_callsign_input_enabled(!checked);
                        if !checked {
                            ui.main_page.set_callsign_input_text("");
                            ui.main_page.set_callsign_input_focused();
                        } else {
                            if let Some(es_callsign) = ui.main_page.euroscope_callsign.clone() {
                                ui.main_page.set_callsign_input_text(&es_callsign);
                            } else {
                                ui.main_page.set_callsign_input_text("");
                            }
                        }
                        return 0;
                    },
                    RES_FETCH_METAR_PUSHBUTTON => {
                        let ui = &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Win32Ui);
                        let station = ui.main_page.get_metar_station_input_text();
                        if station.is_empty() { 
                            ui.main_page.set_metar_station_input_focused();
                            return 0;
                        }
                        ui.app.try_lookup_metar(station);
                        ui.main_page.select_all_metar_station_input_text();
                        ui.main_page.set_metar_station_input_focused();
                        return 0;
                    },
                    RES_ONLY_SHOW_VS_AC_CHECKBOX => {
                        let ui = &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Win32Ui);
                        
                        if ui.main_page.get_only_show_vs_ac_checkbox_enabled() {
                            let checked = ui.main_page.get_only_show_vs_ac_checkbox();
                            ui.main_page.only_show_vatsim_aircraft_selected = checked;
                            ui.app.preferences.set_only_show_vatsim(checked);
                        }
                        return 0;
                    }
                    RES_FETCH_FPS_FROM_VS_CHECKBOX => {
                        let ui = &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Win32Ui);
                        let checked = ui.main_page.get_fetch_flight_plans_checkbox();
                        ui.main_page.set_only_show_vs_ac_checkbox_enabled(checked);
                        let only_show_vs_checked = if checked { ui.main_page.only_show_vatsim_aircraft_selected } else { false };
                        ui.app.preferences.set_fetch_flight_plans(checked);
                        ui.app.preferences.set_only_show_vatsim(only_show_vs_checked);
                        ui.main_page.set_only_show_vs_ac_checkbox(only_show_vs_checked);

                        return 0;
                    }
                    RES_FETCH_METARS_FROM_VS_CHECKBOX => {
                        let ui = &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Win32Ui);
                        let checked = ui.main_page.get_fetch_metars_checkbox();
                        ui.app.preferences.set_fetch_metars(checked);
                        return 0;
                    },
                    
                    1 => {
                        let metar_input = GetDlgItem(hwnd, RES_METAR_STATION_EDITTEXT as i32);
                        if GetFocus() == metar_input {
                            SendMessageW(GetDlgItem(hwnd, RES_FETCH_METAR_PUSHBUTTON as i32), BM_CLICK, 0, 0);
                            return 0;
                        }
                        return DefWindowProcW(hwnd, msg, wparam, lparam);
                    }
                    _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
                }
            } else if hi == EN_CHANGE as u16 {
                match lo as u32{
                    RES_CALLSIGN_EDITTEXT => {
                        let ui = &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Win32Ui);
                        if IsWindowEnabled(GetDlgItem(hwnd, RES_CALLSIGN_EDITTEXT as i32)) > 0 {
                            let text = ui.main_page.get_callsign_input_text();
                            ui.app.preferences.set_own_callsign(text);
                        }
                        return 0;
                    },
                    _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
                }
            }
             else { return DefWindowProcW(hwnd, msg, wparam, lparam); }
        },


        _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}