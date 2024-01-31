#![allow(unused)]

use windows_sys::{w, Win32::UI::WindowsAndMessaging::WM_APP};




pub const UI_MESSAGE: u32 = WM_APP + 16;
pub const INIT_MESSAGE: u32 = WM_APP + 17;

pub const MAIN_DIALOG_CLASS_NAME: *const u16 = w!("TRAFFIC_VIEWER_MAIN_DIALOG_CLASS");

pub const RES_MAIN_DIALOG: u32 = 16;
pub const RES_ABOUT_DIALOG: u32 = 16;

pub const RES_CALLSIGN_EDITTEXT: u32 = 202;
pub const RES_SYNC_WITH_ES_CHECKBOX: u32 = 211;
pub const RES_FETCH_METARS_FROM_VS_CHECKBOX: u32 = 212;
pub const RES_ONLY_SHOW_VS_AC_CHECKBOX: u32 = 213;
pub const RES_FETCH_FPS_FROM_VS_CHECKBOX: u32 = 214;
pub const RES_METAR_STATION_EDITTEXT: u32 = 222;
pub const RES_FETCH_METAR_PUSHBUTTON: u32 = 223;
pub const RES_ABOUT_DIALOG_CREDITS_EDITTEXT: u32 = 305;
pub const RES_ABOUT_DLG_OK_PUSHBUTTON: u32 = 306;
pub const RES_MENU_MAIN: u32 = 100;
pub const RES_MENU_MAIN_FILE: u32 = 110;
pub const RES_MENU_MAIN_FILE_EXIT: u32 = 111;
pub const RES_MENU_MAIN_HELP: u32 = 120;
pub const RES_MENU_MAIN_HELP_ABOUT: u32 = 121;