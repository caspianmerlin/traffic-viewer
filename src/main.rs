use std::{mem, process, ptr};
use win32_ui_impl::{util, window, Win32Ui};
use windows_sys::{w, Win32::{Foundation::{GetLastError, SetLastError, RECT}, Graphics::Gdi::{CreatePen, DrawTextW, GetSysColor, GetSysColorBrush, Rectangle, SelectObject, SetBkColor, SetBkMode, SetTextColor, UpdateWindow, COLOR_3DFACE, DT_SINGLELINE, DT_VCENTER, PS_SOLID, TRANSPARENT}, System::LibraryLoader::GetModuleHandleW, UI::{Controls::{IsDlgButtonChecked, DRAWITEMSTRUCT, EM_SETLIMITTEXT, EM_SETREADONLY, SBT_OWNERDRAW, SB_SETPARTS, SB_SETTEXT, STATUSCLASSNAME}, WindowsAndMessaging::{CreateDialogParamW, CreateWindowExW, DefWindowProcW, DestroyWindow, DialogBoxParamW, DispatchMessageW, EndDialog, GetClientRect, GetDlgItem, GetMessageW, GetWindowLongPtrW, GetWindowRect, IsDialogMessageW, LoadCursorW, MessageBoxW, PostQuitMessage, RegisterClassExW, SendMessageW, SetWindowLongPtrW, SetWindowPos, ShowWindow, TranslateMessage, CW_USEDEFAULT, DLGWINDOWEXTRA, ES_UPPERCASE, GWLP_USERDATA, GWL_STYLE, IDC_ARROW, MB_ICONERROR, MSG, SWP_NOSIZE, SWP_NOZORDER, SW_SHOW, WM_CLOSE, WM_COMMAND, WM_CREATE, WM_DESTROY, WM_DRAWITEM, WM_GETFONT, WM_INITDIALOG, WNDCLASSEXW, WS_CHILD, WS_VISIBLE}}}};

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



// unsafe extern "system" fn wnd_proc(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize {
//     match msg {
//         WM_CREATE => {
//             let dummy_hwnd = CreateWindowExW(0, w!("STATIC"), w!("STATIC"), 0, CW_USEDEFAULT, CW_USEDEFAULT, 0, 0, 0, 0, GetModuleHandleW(ptr::null()), ptr::null());
//             let mut rect: RECT = mem::zeroed();
//             GetWindowRect(dummy_hwnd, &mut rect);
//             DestroyWindow(dummy_hwnd);
//             SetWindowPos(hwnd, 0, rect.left, rect.top, 0, 0, SWP_NOSIZE | SWP_NOZORDER);
//             return 0;
//         },
//         WM_CLOSE => {
//             DestroyWindow(hwnd);
//             return 0;
//         },
//         WM_COMMAND => {
            
//             let (hi, lo) = split_words(wparam as u32);
//             println!("Hi: {}, lo: {}", hi, lo);
//             if hi == 0 {
//                 match lo as u32 {
//                     RES_MENU_MAIN_FILE_EXIT => {
//                         SendMessageW(hwnd, WM_CLOSE, 0, 0);
//                         return 0;
//                     },
//                     RES_MENU_MAIN_HELP_ABOUT => {
//                         DialogBoxParamW(GetModuleHandleW(ptr::null()), make_int_resource(17), hwnd, Some(about_dialog_proc), 0);
//                         return 0;
//                     },
//                     RES_SYNC_WITH_ES_CHECKBOX => {
//                         let checked = IsDlgButtonChecked(hwnd, RES_SYNC_WITH_ES_CHECKBOX as i32);
//                         SendMessageW(GetDlgItem(hwnd, RES_CALLSIGN_EDITTEXT as i32), EM_SETREADONLY, checked as usize, 0);
//                     return 0;
//                     },
//                     _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
//                 }
//             }
//             return DefWindowProcW(hwnd, msg, wparam, lparam);
//         },
//         WM_DESTROY => {
//             PostQuitMessage(0);
//             return 0;
//         },

//         WM_DRAWITEM => {
//             let state = &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut State);
//             let dis = &mut *(lparam as *mut DRAWITEMSTRUCT);
//             if dis.hwndItem != state.status_bar_hwnd { return 0; }
            
//             let font = SendMessageW(dis.hwndItem, WM_GETFONT, 0, 0);
            
//             let mut rect: RECT = mem::zeroed();
//             GetClientRect(dis.hwndItem, &mut rect);
//             let mut local_rect = RECT {
//                 left: rect.left + dis.rcItem.left,
//                 right: rect.left +  dis.rcItem.right,
//                 top: rect.top + dis.rcItem.top,
//                 bottom: rect.top + dis.rcItem.bottom,
//             };


//             let pen = CreatePen(PS_SOLID, 1, GetSysColor(COLOR_3DFACE));
//             let brush = GetSysColorBrush(COLOR_3DFACE);
//             SelectObject(dis.hDC, pen);
//             SelectObject(dis.hDC, brush);
//             Rectangle(dis.hDC, local_rect.left, local_rect.top, local_rect.right, local_rect.bottom);
//             let text_colour = if dis.itemID == 0 { solid_colour(5, 150, 5) } else { solid_colour(255, 0, 0) };
//             SetTextColor(dis.hDC, text_colour);
//             SetBkColor(dis.hDC, solid_colour(0, 255, 0));
//             SetBkMode(dis.hDC, TRANSPARENT as i32);
//             local_rect.left += 5;
//             let old_font = SelectObject(dis.hDC, font);
//             match dis.itemID {
//                 0 => DrawTextW(dis.hDC, w!("EuroScope"), -1, &mut local_rect, DT_VCENTER | DT_SINGLELINE),
//                 1 => DrawTextW(dis.hDC, w!("MSFS"), -1, &mut local_rect, DT_VCENTER | DT_SINGLELINE),
//                 2 => DrawTextW(dis.hDC, w!("METARs"), -1, &mut local_rect, DT_VCENTER | DT_SINGLELINE),
//                 _ => DrawTextW(dis.hDC, w!("VATSIM"), -1, &mut local_rect, DT_VCENTER | DT_SINGLELINE),
//             };
            
//             SelectObject(dis.hDC, old_font);
            





//             return 1;

//         }



//         _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
//     }
// }
