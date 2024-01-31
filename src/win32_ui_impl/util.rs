use std::mem;

pub const fn solid_colour(r: u8, g: u8, b: u8) -> u32 {
    let b = (b as u32) << 16;
    let g = (g as u32) << 8;
    let r = r as u32;

    b | g | r
}

#[inline]
pub fn wide_null(s: impl AsRef<str>) -> Vec<u16> {
  s.as_ref().encode_utf16().chain(Some(0)).collect()
}

#[inline]
pub const unsafe fn make_int_resource(x: u32) -> *const u16 {
    mem::transmute(x as u64)
}

#[inline]
pub const fn split_words(wparam: u32) -> (u16, u16) {
    ((wparam >> 16) as u16, (wparam & 0xffff) as u16)
}