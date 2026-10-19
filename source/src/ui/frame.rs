use super::*;
use scrollpilot::layout::{frame_hit, FrameHit};

impl Ui {
    pub(super) unsafe fn frame_message(
        &self,
        h: HWND,
        msg: u32,
        w: usize,
        l: isize,
    ) -> Option<isize> {
        match msg {
            WM_NCCALCSIZE if w != 0 => {
                if IsZoomed(h) != 0 {
                    let mut monitor = MONITORINFO {
                        cbSize: size_of::<MONITORINFO>() as u32,
                        ..zeroed()
                    };
                    if GetMonitorInfoW(MonitorFromWindow(h, MONITOR_DEFAULTTONEAREST), &mut monitor)
                        != 0
                    {
                        (*(l as *mut NCCALCSIZE_PARAMS)).rgrc[0] = monitor.rcWork;
                    }
                }
                Some(0)
            }
            WM_NCHITTEST => {
                let maximize = self.handle(WINDOW_MAXIMIZE) as HWND;
                if !maximize.is_null() {
                    let mut bounds: RECT = zeroed();
                    GetWindowRect(maximize, &mut bounds);
                    let px = (l as u16 as i16) as i32;
                    let py = ((l >> 16) as u16 as i16) as i32;
                    if px >= bounds.left
                        && px < bounds.right
                        && py >= bounds.top
                        && py < bounds.bottom
                    {
                        return Some(HTMAXBUTTON as isize);
                    }
                }
                let mut rect = zeroed();
                GetWindowRect(h, &mut rect);
                let x = (l as u16 as i16) as i32 - rect.left;
                let y = ((l >> 16) as u16 as i16) as i32 - rect.top;
                let hit = frame_hit(
                    (f64::from(x) / self.scale) as i32,
                    (f64::from(y) / self.scale) as i32,
                    (f64::from(rect.right - rect.left) / self.scale) as i32,
                    (f64::from(rect.bottom - rect.top) / self.scale) as i32,
                    IsZoomed(h) != 0,
                );
                Some(match hit {
                    FrameHit::Client => HTCLIENT,
                    FrameHit::Caption => HTCAPTION,
                    FrameHit::Left => HTLEFT,
                    FrameHit::Right => HTRIGHT,
                    FrameHit::Top => HTTOP,
                    FrameHit::Bottom => HTBOTTOM,
                    FrameHit::TopLeft => HTTOPLEFT,
                    FrameHit::TopRight => HTTOPRIGHT,
                    FrameHit::BottomLeft => HTBOTTOMLEFT,
                    FrameHit::BottomRight => HTBOTTOMRIGHT,
                } as isize)
            }
            _ => None,
        }
    }

    pub(super) unsafe fn paint_caption_button(
        &self,
        h: HWND,
        dc: HDC,
        id: u16,
        width: i32,
        height: i32,
        pressed: bool,
    ) {
        let mut cursor = zeroed();
        GetCursorPos(&mut cursor);
        ScreenToClient(h, &mut cursor);
        let hover = cursor.x >= 0
            && cursor.y >= 0
            && cursor.x < self.px(width)
            && cursor.y < self.px(height);
        let active = hover || pressed;
        let close = id == WINDOW_CLOSE;
        let tint = if close && active { WHITE } else { INK };
        round(
            dc,
            Rect::new(0, 0, width, height),
            7,
            self.scale,
            if close && active {
                0xc53b45
            } else if active {
                SELECTED
            } else {
                CANVAS
            },
            None,
        );
        let x = width / 2;
        let y = height / 2;
        match id {
            WINDOW_MINIMIZE => line(dc, x - 5, y + 3, x + 5, y + 3, self.scale, tint, 1),
            WINDOW_MAXIMIZE => {
                let restore = IsZoomed(self.hwnd as HWND) != 0;
                let draw_box = |left, top| {
                    line(dc, left, top, left + 9, top, self.scale, tint, 1);
                    line(dc, left + 9, top, left + 9, top + 9, self.scale, tint, 1);
                    line(dc, left + 9, top + 9, left, top + 9, self.scale, tint, 1);
                    line(dc, left, top + 9, left, top, self.scale, tint, 1);
                };
                if restore {
                    draw_box(x - 2, y - 6);
                    fill(
                        dc,
                        Rect::new(x - 5, y - 3, 10, 10),
                        self.scale,
                        if active { SELECTED } else { CANVAS },
                    );
                }
                draw_box(x - 5, y - if restore { 3 } else { 5 });
            }
            WINDOW_CLOSE => {
                line(dc, x - 5, y - 5, x + 5, y + 5, self.scale, tint, 1);
                line(dc, x + 5, y - 5, x - 5, y + 5, self.scale, tint, 1);
            }
            _ => {}
        }
    }
}
