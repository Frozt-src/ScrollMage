use std::mem::size_of_val;
use windows_sys::Win32::{Foundation::HWND, Graphics::Dwm::*};

/// Ask Windows for its small-radius corners; older systems may reject the hint.
/// # Safety
/// window must be a valid top-level window owned by the caller.
pub unsafe fn apply_small_corners(window: HWND) -> i32 {
    let preference = DWMWCP_ROUNDSMALL;
    DwmSetWindowAttribute(
        window,
        DWMWA_WINDOW_CORNER_PREFERENCE as u32,
        (&preference as *const i32).cast(),
        size_of_val(&preference) as u32,
    )
}
