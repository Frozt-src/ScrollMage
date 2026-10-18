use scrollpilot::{
    frame_style::apply_small_corners,
    layout::{DEFAULT_HEIGHT, DEFAULT_WIDTH},
};
use std::{
    mem::{size_of, zeroed},
    ptr::{null, null_mut},
};
use windows_sys::Win32::{
    Foundation::*, Graphics::Dwm::*, System::LibraryLoader::*, UI::WindowsAndMessaging::*,
};

unsafe extern "system" fn window(h: HWND, msg: u32, w: usize, l: isize) -> isize {
    if msg == WM_NCCALCSIZE && w != 0 {
        0
    } else {
        DefWindowProcW(h, msg, w, l)
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let name: Vec<u16> = "ScrollMage.FrameVerification\0".encode_utf16().collect();
        let instance = GetModuleHandleW(null());
        let class = WNDCLASSW {
            lpfnWndProc: Some(window),
            hInstance: instance,
            lpszClassName: name.as_ptr(),
            ..zeroed()
        };
        if RegisterClassW(&class) == 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        let foreground = GetForegroundWindow();
        let h = CreateWindowExW(
            WS_EX_NOACTIVATE,
            name.as_ptr(),
            name.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
            0,
            0,
            DEFAULT_WIDTH,
            DEFAULT_HEIGHT,
            null_mut(),
            null_mut(),
            instance,
            null(),
        );
        if h.is_null() {
            return Err(std::io::Error::last_os_error().into());
        }
        SetWindowPos(
            h,
            null_mut(),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
        let set_status = apply_small_corners(h);
        let mut preference = -1i32;
        let read_status = DwmGetWindowAttribute(
            h,
            DWMWA_WINDOW_CORNER_PREFERENCE as u32,
            (&mut preference as *mut i32).cast(),
            size_of::<i32>() as u32,
        );
        let mut client: RECT = zeroed();
        let mut frame: RECT = zeroed();
        GetClientRect(h, &mut client);
        GetWindowRect(h, &mut frame);
        let report = serde_json::json!({
            "corner_set_hresult":set_status,"corner_read_hresult":read_status,"corner_preference":preference,
            "small_corners_accepted":set_status >= 0 && read_status >= 0 && preference == DWMWCP_ROUNDSMALL,
            "client_size":[client.right,client.bottom],"outer_size":[frame.right-frame.left,frame.bottom-frame.top],
            "default_size_matches":client.right == DEFAULT_WIDTH && client.bottom == DEFAULT_HEIGHT,
            "window_visible":IsWindowVisible(h) != 0,"desktop_focus_unchanged":GetForegroundWindow() == foreground,
            "exact_radius_pixels":"Controlled by Windows; not established by the preference readback"
        });
        DestroyWindow(h);
        UnregisterClassW(name.as_ptr(), instance);
        println!("{}", serde_json::to_string_pretty(&report)?);
        if !report["default_size_matches"].as_bool().unwrap_or(false) {
            return Err("Frame sizing validation failed.".into());
        }
    }
    Ok(())
}
