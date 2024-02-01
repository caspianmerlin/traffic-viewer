use std::{mem, ptr};

use windows_sys::{w, Win32::{Foundation::RECT, Graphics::Gdi::{CreatePen, DrawTextW, GetSysColor, GetSysColorBrush, Rectangle, SelectObject, SetBkColor, SetBkMode, SetTextColor, COLOR_3DFACE, DT_SINGLELINE, DT_VCENTER, PS_SOLID, TRANSPARENT}, UI::{Controls::{DRAWITEMSTRUCT, SBT_OWNERDRAW, SB_SETPARTS, SB_SETTEXT, STATUSCLASSNAME}, WindowsAndMessaging::{CreateWindowExW, GetClientRect, GetWindowRect, SendMessageW, WM_GETFONT, WS_CHILD, WS_VISIBLE}}}};

use super::util::solid_colour;



pub struct StatusBar {
    hwnd: isize,
    
    euroscope: Section,
    msfs: Section,
    metars: Section,
    vatsim: Section,

}

impl StatusBar {
    pub unsafe fn new() -> StatusBar {
        
        // Create the sections
        let euroscope = Section::new(0, w!("EuroScope"), 0);
        let msfs = Section::new(1, w!("MSFS"), 0);
        let metars = Section::new(2, w!("METARs"), 0);
        let vatsim = Section::new(3, w!("VATSIM"), 0);

        StatusBar { hwnd: 0, euroscope, msfs, metars, vatsim }
    }

    pub unsafe fn init(&mut self, hinst: isize, parent_hwnd: isize) {
        // Create status bar HWND
        self.hwnd = CreateWindowExW(0, STATUSCLASSNAME, ptr::null(), WS_CHILD | WS_VISIBLE , 0, 0, 0, 0, parent_hwnd, 0, hinst, ptr::null());
        // Figure out how wide each section is going to be and set up the status bar accordingly
        let mut sb_rect: RECT = mem::zeroed();
        GetWindowRect(self.hwnd, &mut sb_rect);
        let sb_section_width = (sb_rect.right - sb_rect.left) / 4;
        let widths = [sb_section_width, sb_section_width * 2, sb_section_width * 3, -1];
        SendMessageW(self.hwnd, SB_SETPARTS, mem::size_of_val(&widths) / mem::size_of::<i32>(), widths.as_ptr() as isize);

        self.euroscope.init(self.hwnd);
        self.msfs.init(self.hwnd);
        self.metars.init(self.hwnd);
        self.vatsim.init(self.hwnd);
    }

    pub unsafe fn draw(&mut self, draw_item_struct: &DRAWITEMSTRUCT) -> bool {

        if draw_item_struct.hwndItem != self.hwnd { return false; }
        match draw_item_struct.itemID {
            0 => self.euroscope.draw(draw_item_struct),
            1 => self.msfs.draw(draw_item_struct),
            2 => self.metars.draw(draw_item_struct),
            3 => self.vatsim.draw(draw_item_struct),
            _ => return false,
        }

        return true;
    }

    pub unsafe fn set_euroscope_connected(&mut self, connected: bool) {
        self.euroscope.set_connected(connected);
    }
    pub unsafe fn set_msfs_connected(&mut self, connected: bool) {
        self.msfs.set_connected(connected);
    }
    pub unsafe fn set_metars_connected(&mut self, connected: bool) {
        self.metars.set_connected(connected);
    }
    pub unsafe fn set_vatsim_connected(&mut self, connected: bool) {
        self.vatsim.set_connected(connected);
    }
}

struct Section {
    id: u32,
    parent_hwnd: isize,
    text: *const u16,
    connected: bool,
}
impl Section {
    pub unsafe fn new(id: u32, text: *const u16, parent_hwnd: isize) -> Section {
        Section { id, parent_hwnd, text, connected: false }
    }
    pub unsafe fn init(&mut self, parent_hwnd: isize) {
        self.parent_hwnd = parent_hwnd;
        SendMessageW(self.parent_hwnd, SB_SETTEXT, (SBT_OWNERDRAW | self.id) as usize, 0);
    }
    unsafe fn draw(&self, draw_item_struct: &DRAWITEMSTRUCT) {
        
        let font = SendMessageW(draw_item_struct.hwndItem, WM_GETFONT, 0, 0);
            
        let mut rect: RECT = mem::zeroed();
        GetClientRect(draw_item_struct.hwndItem, &mut rect);
        let mut local_rect = RECT {
            left: rect.left + draw_item_struct.rcItem.left,
            right: rect.left +  draw_item_struct.rcItem.right,
            top: rect.top + draw_item_struct.rcItem.top,
            bottom: rect.top + draw_item_struct.rcItem.bottom,
        };


        let pen = CreatePen(PS_SOLID, 1, GetSysColor(COLOR_3DFACE));
        let brush = GetSysColorBrush(COLOR_3DFACE);
        SelectObject(draw_item_struct.hDC, pen);
        SelectObject(draw_item_struct.hDC, brush);
        Rectangle(draw_item_struct.hDC, local_rect.left, local_rect.top, local_rect.right, local_rect.bottom);
        let text_colour = if self.connected { solid_colour(5, 150, 5) } else { solid_colour(255, 0, 0) };
        SetTextColor(draw_item_struct.hDC, text_colour);
        SetBkColor(draw_item_struct.hDC, solid_colour(0, 255, 0));
        SetBkMode(draw_item_struct.hDC, TRANSPARENT as i32);
        local_rect.left += 5;
        let old_font = SelectObject(draw_item_struct.hDC, font);
        DrawTextW(draw_item_struct.hDC, self.text, -1, &mut local_rect, DT_VCENTER | DT_SINGLELINE);
        SelectObject(draw_item_struct.hDC, old_font);
    }
    pub unsafe fn set_connected(&mut self, connected: bool) {
        if connected != self.connected {
            self.connected = connected;
            SendMessageW(self.parent_hwnd, SB_SETTEXT, (SBT_OWNERDRAW | self.id) as usize, 0);
        }
    }
}