use std::{mem, ptr};
use windows_sys::{w, Win32::{Foundation::{GetLastError, SetLastError, RECT}, Graphics::Gdi::{CreatePen, DrawTextW, GetSysColor, GetSysColorBrush, Rectangle, SelectObject, SetBkColor, SetBkMode, SetTextColor, UpdateWindow, COLOR_3DFACE, DT_SINGLELINE, DT_VCENTER, PS_SOLID, TRANSPARENT}, System::LibraryLoader::GetModuleHandleW, UI::{Controls::{IsDlgButtonChecked, DRAWITEMSTRUCT, EM_SETLIMITTEXT, EM_SETREADONLY, SBT_OWNERDRAW, SB_SETPARTS, SB_SETTEXT, STATUSCLASSNAME}, WindowsAndMessaging::{CreateDialogParamW, CreateWindowExW, DefWindowProcW, DestroyWindow, DialogBoxParamW, DispatchMessageW, EndDialog, GetClientRect, GetDlgItem, GetMessageW, GetWindowLongPtrW, GetWindowRect, IsDialogMessageW, LoadCursorW, PostQuitMessage, RegisterClassExW, SendMessageW, SetWindowLongPtrW, SetWindowPos, ShowWindow, TranslateMessage, CW_USEDEFAULT, DLGWINDOWEXTRA, ES_UPPERCASE, GWLP_USERDATA, GWL_STYLE, IDC_ARROW, MSG, SWP_NOSIZE, SWP_NOZORDER, SW_SHOW, WM_CLOSE, WM_COMMAND, WM_CREATE, WM_DESTROY, WM_DRAWITEM, WM_GETFONT, WM_INITDIALOG, WNDCLASSEXW, WS_CHILD, WS_VISIBLE}}}};

const RES_MAIN_DIALOG: u32 = 16;
const RES_ABOUT_DIALOG: u32 = 16;
const MAIN_DIALOG_CLASS_NAME: *const u16 = w!("TRAFFIC_VIEWER_MAIN_DIALOG_CLASS");
const RES_CALLSIGN_EDITTEXT: u32 = 202;
const RES_SYNC_WITH_ES_CHECKBOX: u32 = 211;
const RES_FETCH_METARS_FROM_VS_CHECKBOX: u32 = 212;
const RES_ONLY_SHOW_VS_AC_CHECKBOX: u32 = 213;
const RES_FETCH_FPS_FROM_VS_CHECKBOX: u32 = 214;
const RES_METAR_STATION_EDITTEXT: u32 = 222;
const RES_FETCH_METAR_PUSHBUTTON: u32 = 223;
const RES_ABOUT_DIALOG_CREDITS_EDITTEXT: u32 = 305;
const RES_ABOUT_DLG_OK_PUSHBUTTON: u32 = 306;
const RES_MENU_MAIN: u32 = 100;
const RES_MENU_MAIN_FILE: u32 = 110;
const RES_MENU_MAIN_FILE_EXIT: u32 = 111;
const RES_MENU_MAIN_HELP: u32 = 120;
const RES_MENU_MAIN_HELP_ABOUT: u32 = 121;

fn main() {
    
    unsafe {

        let mut wnd_class: WNDCLASSEXW = mem::zeroed();
        wnd_class.cbSize = mem::size_of::<WNDCLASSEXW>() as u32;
        wnd_class.lpfnWndProc = Some(wnd_proc);
        wnd_class.hCursor = LoadCursorW(0, IDC_ARROW);
        wnd_class.hbrBackground = GetSysColorBrush(COLOR_3DFACE);
        wnd_class.lpszClassName = MAIN_DIALOG_CLASS_NAME;
        wnd_class.cbWndExtra = DLGWINDOWEXTRA as i32;
        wnd_class.hInstance = GetModuleHandleW(ptr::null());
        wnd_class.lpszMenuName = make_int_resource(RES_MENU_MAIN);

        RegisterClassExW(&wnd_class);

        let hwnd = CreateDialogParamW(GetModuleHandleW(ptr::null()), make_int_resource(RES_MAIN_DIALOG), 0, None, 0);
        let status = CreateWindowExW(0, STATUSCLASSNAME, ptr::null(), WS_CHILD | WS_VISIBLE , 0, 0, 0, 0, hwnd, 0, GetModuleHandleW(ptr::null()), ptr::null());
        let mut sb_rect: RECT = mem::zeroed();
        GetWindowRect(status, &mut sb_rect);
        let sb_section_width = (sb_rect.right - sb_rect.left) / 4;
        let widths = [sb_section_width, sb_section_width * 2, sb_section_width * 3, -1];
        SendMessageW(status, SB_SETPARTS, mem::size_of_val(&widths) / mem::size_of::<i32>(), widths.as_ptr() as isize);

        let state = Box::into_raw(Box::new(State { status_bar_hwnd: status }));
        SetLastError(0);
        let res = SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
        let le = GetLastError();
        assert!(res == 0 && le == 0);
        // SendMessageW(status, SB_SETTEXT, 0, w!("ES: Disconnected") as isize);
        // SendMessageW(status, SB_SETTEXT, 1, w!("ES: Disconnected") as isize);
        // SendMessageW(status, SB_SETTEXT, 2, w!("ES: Disconnected") as isize);

        SendMessageW(status, SB_SETTEXT, SBT_OWNERDRAW as usize, 0);
        SendMessageW(status, SB_SETTEXT, (SBT_OWNERDRAW | 1) as usize, 0);
        SendMessageW(status, SB_SETTEXT, (SBT_OWNERDRAW | 2) as usize, 0);
        SendMessageW(status, SB_SETTEXT, (SBT_OWNERDRAW | 3) as usize, 0);
        
        let callsign_input_hwnd = GetDlgItem(hwnd, RES_CALLSIGN_EDITTEXT as i32);
        SendMessageW(callsign_input_hwnd, EM_SETLIMITTEXT, 10, 0);
        let mut style = GetWindowLongPtrW(callsign_input_hwnd, GWL_STYLE);
        style |= ES_UPPERCASE as isize;
        SetWindowLongPtrW(callsign_input_hwnd, GWL_STYLE, style);


        let station_input_hwnd = GetDlgItem(hwnd, RES_METAR_STATION_EDITTEXT as i32);
        SendMessageW(station_input_hwnd, EM_SETLIMITTEXT, 4, 0);
        let mut style = GetWindowLongPtrW(station_input_hwnd, GWL_STYLE);
        style |= ES_UPPERCASE as isize;
        SetWindowLongPtrW(station_input_hwnd, GWL_STYLE, style);


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



unsafe extern "system" fn wnd_proc(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize {
    match msg {
        WM_CREATE => {
            let dummy_hwnd = CreateWindowExW(0, w!("STATIC"), w!("STATIC"), 0, CW_USEDEFAULT, CW_USEDEFAULT, 0, 0, 0, 0, GetModuleHandleW(ptr::null()), ptr::null());
            let mut rect: RECT = mem::zeroed();
            GetWindowRect(dummy_hwnd, &mut rect);
            DestroyWindow(dummy_hwnd);
            SetWindowPos(hwnd, 0, rect.left, rect.top, 0, 0, SWP_NOSIZE | SWP_NOZORDER);
            return 0;
        },
        WM_CLOSE => {
            DestroyWindow(hwnd);
            return 0;
        },
        WM_COMMAND => {
            
            let (hi, lo) = split_words(wparam as u32);
            println!("Hi: {}, lo: {}", hi, lo);
            if hi == 0 {
                match lo as u32 {
                    RES_MENU_MAIN_FILE_EXIT => {
                        SendMessageW(hwnd, WM_CLOSE, 0, 0);
                        return 0;
                    },
                    RES_MENU_MAIN_HELP_ABOUT => {
                        DialogBoxParamW(GetModuleHandleW(ptr::null()), make_int_resource(17), hwnd, Some(about_dialog_proc), 0);
                        return 0;
                    },
                    RES_SYNC_WITH_ES_CHECKBOX => {
                        let checked = IsDlgButtonChecked(hwnd, RES_SYNC_WITH_ES_CHECKBOX as i32);
                        SendMessageW(GetDlgItem(hwnd, RES_CALLSIGN_EDITTEXT as i32), EM_SETREADONLY, checked as usize, 0);
                    return 0;
                    },
                    _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
                }
            }
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        },
        WM_DESTROY => {
            PostQuitMessage(0);
            return 0;
        },

        WM_DRAWITEM => {
            let state = &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut State);
            let dis = &mut *(lparam as *mut DRAWITEMSTRUCT);
            if dis.hwndItem != state.status_bar_hwnd { return 0; }
            
            let font = SendMessageW(dis.hwndItem, WM_GETFONT, 0, 0);
            
            let mut rect: RECT = mem::zeroed();
            GetClientRect(dis.hwndItem, &mut rect);
            let mut local_rect = RECT {
                left: rect.left + dis.rcItem.left,
                right: rect.left +  dis.rcItem.right,
                top: rect.top + dis.rcItem.top,
                bottom: rect.top + dis.rcItem.bottom,
            };


            let pen = CreatePen(PS_SOLID, 1, GetSysColor(COLOR_3DFACE));
            let brush = GetSysColorBrush(COLOR_3DFACE);
            SelectObject(dis.hDC, pen);
            SelectObject(dis.hDC, brush);
            Rectangle(dis.hDC, local_rect.left, local_rect.top, local_rect.right, local_rect.bottom);
            let text_colour = if dis.itemID == 0 { solid_colour(5, 150, 5) } else { solid_colour(255, 0, 0) };
            SetTextColor(dis.hDC, text_colour);
            SetBkColor(dis.hDC, solid_colour(0, 255, 0));
            SetBkMode(dis.hDC, TRANSPARENT as i32);
            local_rect.left += 5;
            let old_font = SelectObject(dis.hDC, font);
            match dis.itemID {
                0 => DrawTextW(dis.hDC, w!("EuroScope"), -1, &mut local_rect, DT_VCENTER | DT_SINGLELINE),
                1 => DrawTextW(dis.hDC, w!("MSFS"), -1, &mut local_rect, DT_VCENTER | DT_SINGLELINE),
                2 => DrawTextW(dis.hDC, w!("METARs"), -1, &mut local_rect, DT_VCENTER | DT_SINGLELINE),
                _ => DrawTextW(dis.hDC, w!("VATSIM"), -1, &mut local_rect, DT_VCENTER | DT_SINGLELINE),
            };
            
            SelectObject(dis.hDC, old_font);
            





            return 1;

        }



        _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
const unsafe fn make_int_resource(x: u32) -> *const u16 {
    mem::transmute(x as u64)
}


const fn split_words(wparam: u32) -> (u16, u16) {
    ((wparam >> 16) as u16, (wparam & 0xffff) as u16)
}

#[allow(unused)]
unsafe extern "system" fn about_dialog_proc(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize {
    match msg {
        WM_INITDIALOG => {
            let credits_hwnd = GetDlgItem(hwnd, RES_ABOUT_DIALOG_CREDITS_EDITTEXT as i32);
            SendMessageW(credits_hwnd, EM_SETREADONLY, 1, 0);
            return 0;
        },
        WM_COMMAND => {
            if wparam == RES_ABOUT_DLG_OK_PUSHBUTTON as usize {
                EndDialog(hwnd, 1);
                return 1;
            }
            return 0;
        }
        WM_CLOSE => {
            EndDialog(hwnd, 1);
            return 1;
        },

        _ => return 0,
    }
}



#[allow(unused)]
struct State {
    status_bar_hwnd: isize,
}


const fn solid_colour(r: u8, g: u8, b: u8) -> u32 {
    let b = (b as u32) << 16;
    let g = (g as u32) << 8;
    let r = r as u32;

    b | g | r
}


#[inline]
pub fn wide_null(s: impl AsRef<str>) -> Vec<u16> {
  s.as_ref().encode_utf16().chain(Some(0)).collect()
}