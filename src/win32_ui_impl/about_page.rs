use windows_sys::Win32::UI::{Controls::EM_SETREADONLY, WindowsAndMessaging::{DialogBoxParamW, EndDialog, GetDlgItem, SendMessageW, WM_CLOSE, WM_COMMAND, WM_INITDIALOG}};

use super::{consts::{RES_ABOUT_DIALOG, RES_ABOUT_DIALOG_CREDITS_EDITTEXT, RES_ABOUT_DLG_OK_PUSHBUTTON}, util};




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

pub unsafe fn show_about_window(hinst: isize, main_hwnd: isize) {
    DialogBoxParamW(hinst, util::make_int_resource(RES_ABOUT_DIALOG), main_hwnd, Some(about_dialog_proc), 0);
}