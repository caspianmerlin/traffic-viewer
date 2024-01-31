use std::{mem, ptr};

use windows_sys::{w, Win32::{Foundation::{HWND, RECT}, Graphics::Gdi::{GetSysColorBrush, COLOR_3DFACE}, System::LibraryLoader::GetModuleHandleW, UI::{Controls::DRAWITEMSTRUCT, WindowsAndMessaging::{CreateDialogParamW, CreateWindowExW, DefWindowProcW, DestroyWindow, GetWindowLongPtrW, GetWindowRect, LoadCursorW, PostQuitMessage, RegisterClassExW, SendMessageW, SetWindowLongPtrW, SetWindowPos, CW_USEDEFAULT, DLGWINDOWEXTRA, GWLP_USERDATA, IDC_ARROW, SWP_NOSIZE, SWP_NOZORDER, WM_CLOSE, WM_CREATE, WM_COMMAND, WM_DESTROY, WM_DRAWITEM, WM_NCCREATE, WNDCLASSEXW}}}};

use crate::win32_ui_impl::{consts::{MAIN_DIALOG_CLASS_NAME, RES_MAIN_DIALOG, RES_MENU_MAIN}, util};

use super::{about_page, consts::{INIT_MESSAGE, RES_MENU_MAIN_FILE_EXIT, RES_MENU_MAIN_HELP_ABOUT, RES_SYNC_WITH_ES_CHECKBOX}, Win32Ui};

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
            let ui_boxed = Box::into_raw(Box::new(Win32Ui::new(GetModuleHandleW(ptr::null()), hwnd)));
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
        WM_DESTROY => {
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
                        ui.main_page.set_callsign_input_enabled(!checked);
                        if !checked {
                            ui.main_page.select_all_callsign_input_text();
                            ui.main_page.set_callsign_input_focused();
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