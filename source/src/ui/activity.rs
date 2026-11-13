//! Activity page: a status header, live session counters and a short explainer.
use super::*;

const TITLE: u16 = 3110;
const ABOUT_TEXT: u16 = 3111;
const OBSERVATION: u16 = 3112;
const ABOUT_TITLE: u16 = 3113;
/// Per tile: name, number, detail. Created in that order so screen readers
/// announce "Accepted, 1,204, Wheel notches smoothed".
const TILES: [(u16, u16, u16, &str, &str); 3] = [
    (3114, 3115, 3116, "Accepted", "Wheel notches smoothed"),
    (3117, 3118, 3119, "Output", "Smooth steps sent to Windows"),
    (3120, 3121, 3122, "Bypassed", "Passed through unchanged"),
];
const ABOUT: &str = "Point at an app and scroll to see whether its input is smoothed or passed through. The counters cover this session only; they are not a frame rate.\r\n\r\nTouchpads, fine-resolution wheels and input injected by other software pass through unchanged.\r\n\r\nSome apps turn small wheel movements into line steps, so the result can differ from app to app.\r\n\r\nWith Super smooth, a quick flick keeps coasting. Reverse the wheel, click or pause to stop it.\r\n\r\nMomentum friction and elastic response only affect Scroll Test.";
const LONGEST_OBSERVATION: &str =
    "ApplicationFrameHost.exe: Windows wheel routing unavailable; ordinary wheel input passes through.";
/// Status dot beside the title, in body coordinates before scrolling.
const DOT: Rect = Rect {
    x: 24,
    y: 30,
    w: 12,
    h: 12,
};

pub(super) fn muted(id: u16) -> bool {
    id == OBSERVATION || TILES.iter().any(|t| t.2 == id)
}

/// Body-relative geometry before scrolling.
pub(super) struct ActivityLayout {
    width: i32,
    header: Rect,
    observation_h: i32,
    tiles: [Rect; 3],
    about: Rect,
    about_text_h: i32,
    pub(super) height: i32,
}

fn grouped(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

impl Ui {
    pub(super) unsafe fn build_activity(&mut self) {
        self.static_text("", TITLE, true, self.fonts.brand);
        self.static_text("", OBSERVATION, true, self.fonts.body);
        for (name, number, detail, label, help) in TILES {
            self.static_text(label, name, true, self.fonts.body);
            self.static_text("0", number, true, self.fonts.display);
            self.static_text(help, detail, true, self.fonts.small);
        }
        self.static_text("How it works", ABOUT_TITLE, true, self.fonts.section);
        self.static_text(ABOUT, ABOUT_TEXT, true, self.fonts.body);
    }

    /// Wrapped height of `value` in logical units for a column `w` wide.
    unsafe fn text_height(&self, font: usize, value: &str, w: i32) -> i32 {
        let dc = GetDC(null_mut());
        let previous = SelectObject(dc, font as HGDIOBJ);
        // Measure slightly narrower than the control so rounding never adds a line.
        let mut r = RECT {
            left: 0,
            top: 0,
            right: self.px(w - 2),
            bottom: 0,
        };
        let value = wide(value);
        DrawTextW(
            dc,
            value.as_ptr(),
            (value.len() - 1) as i32,
            &mut r,
            DT_CALCRECT | DT_WORDBREAK | DT_NOPREFIX,
        );
        SelectObject(dc, previous);
        ReleaseDC(null_mut(), dc);
        (f64::from(r.bottom) / self.scale).ceil() as i32
    }

    unsafe fn activity_layout_at(&self, w: i32) -> ActivityLayout {
        // The observation changes live, so reserve room for the longest reason
        // with a long app name at this width.
        let observation_h = self.text_height(self.fonts.body, LONGEST_OBSERVATION, w - 48) + 4;
        let header = Rect::new(0, 0, w, 64 + observation_h + 14);
        let top = header.h + 16;
        let tile_h = 116;
        let tiles = if (w - 32) / 3 >= 200 {
            let tile_w = (w - 32) / 3;
            [
                Rect::new(0, top, tile_w, tile_h),
                Rect::new(tile_w + 16, top, tile_w, tile_h),
                Rect::new(tile_w * 2 + 32, top, w - tile_w * 2 - 32, tile_h),
            ]
        } else {
            [0, 1, 2].map(|i| Rect::new(0, top + i * (tile_h + 12), w, tile_h))
        };
        let about_y = tiles[2].y + tiles[2].h + 16;
        let about_text_h = self.text_height(self.fonts.body, ABOUT, w - 48) + 4;
        let about = Rect::new(0, about_y, w, 54 + about_text_h + 20);
        ActivityLayout {
            width: w,
            header,
            observation_h,
            tiles,
            about,
            about_text_h,
            height: about.y + about.h + 8,
        }
    }

    /// Leaves room for the body scrollbar when the page is taller than the view.
    pub(super) unsafe fn activity_layout(&self) -> ActivityLayout {
        let shell = self.shell();
        let full = self.activity_layout_at(shell.body.w);
        if full.height > shell.body.h {
            self.activity_layout_at(shell.body.w - 18)
        } else {
            full
        }
    }

    pub(super) unsafe fn layout_activity(&self, a: &ActivityLayout) {
        let y = -self.scroll;
        let w = a.width;
        self.at(
            TITLE,
            Rect::new(DOT.x + DOT.w + 10, y + 18, w - DOT.x - DOT.w - 34, 36),
        );
        self.at(OBSERVATION, Rect::new(24, y + 62, w - 48, a.observation_h));
        for (t, (name, number, detail, _, _)) in a.tiles.iter().zip(TILES) {
            let ty = t.y + y;
            self.at(number, Rect::new(t.x + 20, ty + 12, t.w - 40, 48));
            self.at(name, Rect::new(t.x + 20, ty + 62, t.w - 40, 22));
            self.at(detail, Rect::new(t.x + 20, ty + 86, t.w - 40, 18));
        }
        let ay = a.about.y + y;
        self.at(ABOUT_TITLE, Rect::new(24, ay + 18, w - 48, 26));
        self.at(ABOUT_TEXT, Rect::new(24, ay + 54, w - 48, a.about_text_h));
    }

    pub(super) unsafe fn paint_activity(&self, dc: HDC) {
        let a = self.activity_layout();
        let shift = |mut r: Rect| {
            r.y -= self.scroll;
            r
        };
        for card in [a.header, a.tiles[0], a.tiles[1], a.tiles[2], a.about] {
            round(dc, shift(card), 4, self.scale, WHITE, Some(BORDER));
        }
        round(
            dc,
            shift(DOT),
            DOT.w / 2,
            self.scale,
            if self.shared.enabled.load(Ordering::Acquire) {
                ACCENT
            } else {
                AMBER
            },
            None,
        );
    }

    /// Refreshes the live text. Controls keep the values readable by screen readers.
    pub(super) unsafe fn update_activity(&self, title: &str, copy: &str) {
        let put = |id: u16, value: &str| {
            let h = self.handle(id);
            if text(h) != value {
                set(h, value);
            }
        };
        put(TITLE, title);
        put(OBSERVATION, copy.lines().next().unwrap_or(""));
        let (accepted, bypassed, emitted) = self.shared.diagnostics.counts();
        for ((_, number, _, _, _), count) in TILES.into_iter().zip([accepted, emitted, bypassed]) {
            put(number, &grouped(count));
        }
        if self.view == NAV_ACTIVITY {
            let r = shift_px(self, DOT);
            InvalidateRect(self.body as HWND, &r, 0);
        }
    }
}

/// The dot's physical rectangle in the scrolled body, padded for antialiasing.
fn shift_px(ui: &Ui, r: Rect) -> RECT {
    RECT {
        left: ui.px(r.x - 2),
        top: ui.px(r.y - ui.scroll - 2),
        right: ui.px(r.x + r.w + 2),
        bottom: ui.px(r.y - ui.scroll + r.h + 2),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn counts_use_digit_grouping() {
        assert_eq!(super::grouped(0), "0");
        assert_eq!(super::grouped(999), "999");
        assert_eq!(super::grouped(1000), "1,000");
        assert_eq!(super::grouped(1234567), "1,234,567");
    }
}
