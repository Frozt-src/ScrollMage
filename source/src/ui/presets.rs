use super::*;

impl Ui {
    pub(super) unsafe fn select_preset(&mut self) {
        let index = SendMessageW(self.handle(PRESET) as HWND, CB_GETCURSEL, 0, 0);
        let Some(preset) = Preset::ALL.get(index as usize).copied() else {
            return;
        };
        let mut settings = self.shared.settings.read().unwrap().clone();
        settings.apply_preset(preset);
        self.preset = preset;
        for (input, value) in self.inputs.into_iter().zip([
            settings.distance,
            settings.omega,
            settings.acceleration_ms,
            settings.acceleration_max,
            settings.friction,
            settings.elastic_omega,
        ]) {
            set(input, &value.to_string());
        }
        self.note = format!("{} selected. Apply settings to use it.", preset.label());
        self.update();
        InvalidateRect(self.handle(PRESET) as HWND, null(), 0);
    }

    pub(super) unsafe fn paint_preset(&self, h: HWND, dc: HDC, width: i32, height: i32) {
        // Work in physical pixels: outward logical rounding clipped the bottom stroke.
        let rect = RECT {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        };
        let brush = CreateSolidBrush(color(WHITE));
        FillRect(dc, &rect, brush);
        DeleteObject(brush);
        round(
            dc,
            Rect::new(1, 1, width - 2, height - 2),
            (height - 2) / 2,
            1.,
            SELECTED,
            Some(if GetFocus() == h { ACCENT } else { BORDER }),
        );
        label(
            dc,
            Rect::new(self.px(13), 0, width - self.px(40), height),
            1.,
            self.fonts.small,
            ACCENT_TEXT,
            self.preset.label(),
            DT_SINGLELINE | DT_VCENTER,
        );
        let x = width - self.px(17);
        let y = height / 2;
        line(
            dc,
            x - self.px(3),
            y - self.px(1),
            x,
            y + self.px(2),
            1.,
            ACCENT_TEXT,
            1,
        );
        line(
            dc,
            x,
            y + self.px(2),
            x + self.px(3),
            y - self.px(1),
            1.,
            ACCENT_TEXT,
            1,
        );
    }
    pub(super) unsafe fn draw_preset_item(&self, item: &DRAWITEMSTRUCT) {
        if item.itemState & windows_sys::Win32::UI::Controls::ODS_COMBOBOXEDIT != 0 {
            let mut rect: RECT = zeroed();
            GetClientRect(self.handle(PRESET) as HWND, &mut rect);
            self.paint_preset(
                self.handle(PRESET) as HWND,
                item.hDC,
                rect.right,
                rect.bottom,
            );
            return;
        }
        let rect = item.rcItem;
        let selected = item.itemState & ODS_SELECTED != 0;
        let mut info: windows_sys::Win32::UI::Controls::COMBOBOXINFO = zeroed();
        info.cbSize = size_of::<windows_sys::Win32::UI::Controls::COMBOBOXINFO>() as u32;
        windows_sys::Win32::UI::Controls::GetComboBoxInfo(self.handle(PRESET) as HWND, &mut info);
        self.paint_profile_backdrop(info.hwndList, item.hDC, rect);
        let row = Rect::new(
            (f64::from(rect.left) / self.scale).round() as i32 + 4,
            (f64::from(rect.top) / self.scale).round() as i32 + 2,
            (f64::from(rect.right - rect.left) / self.scale).round() as i32 - 8,
            (f64::from(rect.bottom - rect.top) / self.scale).round() as i32 - 4,
        );
        if selected {
            round(item.hDC, row, 12, self.scale, SELECTED, None);
        }
        if let Some(preset) = Preset::ALL.get(item.itemID as usize) {
            label(
                item.hDC,
                Rect::new(
                    (f64::from(rect.left) / self.scale).round() as i32 + 12,
                    (f64::from(rect.top) / self.scale).round() as i32,
                    (f64::from(rect.right - rect.left) / self.scale).round() as i32 - 24,
                    (f64::from(rect.bottom - rect.top) / self.scale).round() as i32,
                ),
                self.scale,
                self.fonts.small,
                if selected { ACCENT_TEXT } else { INK },
                preset.label(),
                DT_SINGLELINE | DT_VCENTER,
            );
        }
    }

    pub(super) unsafe fn restore(&mut self) {
        let hwnd = self.hwnd as HWND;
        ShowWindow(
            hwnd,
            if IsIconic(hwnd) != 0 {
                SW_RESTORE
            } else {
                SW_SHOW
            },
        );
        self.layout();
        self.update();
        RedrawWindow(
            hwnd,
            null(),
            null_mut(),
            RDW_INVALIDATE | RDW_ALLCHILDREN | RDW_UPDATENOW,
        );
        SetForegroundWindow(hwnd);
    }
}

// Preserve the native list's keyboard, dismissal and accessibility behavior.
// The backdrop mixes 14.9% blurred app content into a tint; text stays opaque.
pub(super) unsafe fn style_popup(combo: HWND, scale: f64) {
    use windows_sys::Win32::UI::Controls::{GetComboBoxInfo, SetWindowTheme, COMBOBOXINFO};
    let mut info: COMBOBOXINFO = zeroed();
    info.cbSize = size_of::<COMBOBOXINFO>() as u32;
    if GetComboBoxInfo(combo, &mut info) == 0 || info.hwndList.is_null() {
        return;
    }
    let list = info.hwndList;
    SetWindowTheme(list, wide("").as_ptr(), wide("").as_ptr());
    let style = GetWindowLongPtrW(list, GWL_STYLE) as u32 & !WS_BORDER;
    SetWindowLongPtrW(list, GWL_STYLE, style as isize);
    let ex = GetWindowLongPtrW(list, GWL_EXSTYLE) as u32 & !(WS_EX_CLIENTEDGE | WS_EX_LAYERED);
    SetWindowLongPtrW(list, GWL_EXSTYLE, ex as isize);
    let ui = state(combo);
    if !ui.is_null() {
        (*ui).profile_backdrop.borrow_mut().take();
    }
    SetWindowSubclass(list, Some(popup), 7, ui as usize);
    SetWindowPos(
        list,
        null_mut(),
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
    );
    round_popup(list, scale);
}
unsafe fn round_popup(h: HWND, scale: f64) {
    let mut rect = zeroed();
    GetWindowRect(h, &mut rect);
    let region = CreateRoundRectRgn(
        0,
        0,
        rect.right - rect.left + 1,
        rect.bottom - rect.top + 1,
        px(28, scale),
        px(28, scale),
    );
    if SetWindowRgn(h, region, 1) == 0 {
        DeleteObject(region);
    }
}
unsafe extern "system" fn popup(
    h: HWND,
    msg: u32,
    w: usize,
    l: isize,
    _: usize,
    data: usize,
) -> isize {
    let ptr = data as *mut Ui;
    if msg == WM_ERASEBKGND && !ptr.is_null() {
        let mut rect = zeroed();
        GetClientRect(h, &mut rect);
        (*ptr).paint_profile_backdrop(h, w as HDC, rect);
        return 1;
    }
    let result = DefSubclassProc(h, msg, w, l);
    if msg == WM_SIZE && !ptr.is_null() {
        round_popup(h, (*ptr).scale);
    }
    if msg == WM_WINDOWPOSCHANGED && !ptr.is_null() && IsWindowVisible(h) != 0 {
        (*ptr).prepare_profile_backdrop(h);
        InvalidateRect(h, null(), 0);
    }
    if msg == WM_SHOWWINDOW && w == 0 && !ptr.is_null() {
        (*ptr).profile_backdrop.borrow_mut().take();
    }
    if msg == WM_NCDESTROY {
        RemoveWindowSubclass(h, Some(popup), 7);
    }
    result
}

pub(super) unsafe extern "system" fn preset_combo(
    h: HWND,
    msg: u32,
    w: usize,
    l: isize,
    _: usize,
    _: usize,
) -> isize {
    let ptr = state(h);
    if ptr.is_null() {
        return DefSubclassProc(h, msg, w, l);
    }
    let ui = &*ptr;
    let paint = |dc: HDC, width: i32, height: i32| ui.paint_preset(h, dc, width, height);
    match msg {
        WM_PAINT => {
            buffered(h, paint);
            0
        }
        WM_PRINTCLIENT => {
            let mut rect = zeroed();
            GetClientRect(h, &mut rect);
            paint(w as HDC, rect.right, rect.bottom);
            0
        }
        WM_ERASEBKGND => 1,
        _ => {
            let result = DefSubclassProc(h, msg, w, l);
            if matches!(
                msg,
                WM_SETFOCUS | WM_KILLFOCUS | WM_SETTEXT | WM_SETFONT | CB_SETCURSEL
            ) {
                InvalidateRect(h, null(), 0);
            }
            result
        }
    }
}
