use super::*;

pub(super) const APP: u16 = 101;
pub(super) const TRAY_ICON: u16 = 102;

pub(super) struct Icons {
    pub(super) large: HICON,
    pub(super) small: HICON,
    pub(super) tray: HICON,
}
impl Icons {
    pub(super) unsafe fn load(dpi: u32) -> Result<Self, String> {
        let mut icons = Self {
            large: null_mut(),
            small: null_mut(),
            tray: null_mut(),
        };
        icons.large = load(APP, GetSystemMetricsForDpi(SM_CXICON, dpi))?;
        icons.small = load(APP, GetSystemMetricsForDpi(SM_CXSMICON, dpi))?;
        icons.tray = load(
            TRAY_ICON,
            GetSystemMetricsForDpi(SM_CXSMICON, GetDpiForSystem()),
        )?;
        Ok(icons)
    }
}
unsafe fn load(id: u16, size: i32) -> Result<HICON, String> {
    // Separate owned handles preserve exact sizes; LR_SHARED may reuse a different size.
    let icon = LoadImageW(
        GetModuleHandleW(null()),
        id as usize as *const u16,
        IMAGE_ICON,
        size,
        size,
        LR_DEFAULTCOLOR,
    ) as HICON;
    if icon.is_null() {
        Err("Windows could not load the Scroll Mage icon.".into())
    } else {
        Ok(icon)
    }
}
impl Drop for Icons {
    fn drop(&mut self) {
        unsafe {
            for icon in [self.large, self.small, self.tray] {
                if !icon.is_null() {
                    DestroyIcon(icon);
                }
            }
        }
    }
}
impl Ui {
    pub(super) unsafe fn refresh_icons(&mut self) {
        let Ok(icons) = Icons::load((self.scale * 96.).round() as u32) else {
            return;
        };
        SendMessageW(
            self.hwnd as HWND,
            WM_SETICON,
            ICON_BIG as usize,
            icons.large as isize,
        );
        SendMessageW(
            self.hwnd as HWND,
            WM_SETICON,
            ICON_SMALL as usize,
            icons.small as isize,
        );
        SetClassLongPtrW(self.hwnd as HWND, GCLP_HICON, icons.large as isize);
        self.icon = icons.tray as usize;
        let previous = self.app_icons.replace(icons);
        if self.ready {
            self.tray(NIM_MODIFY);
        }
        drop(previous);
    }
}
