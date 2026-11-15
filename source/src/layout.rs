#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}
impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
    pub fn overlaps(self, b: Self) -> bool {
        self.x < b.x + b.w && b.x < self.x + self.w && self.y < b.y + b.h && b.y < self.y + self.h
    }
}
pub struct Shell {
    pub rail: i32,
    pub body: Rect,
    pub footer: Rect,
    exclusions: usize,
    settings_height: i32,
}
pub const CAPTION_HEIGHT: i32 = 48;
pub const TEST_HEADER: i32 = 104;
pub const DEFAULT_WIDTH: i32 = 1140;
pub const DEFAULT_HEIGHT: i32 = 768;
pub const FORM_ROW_START: i32 = 68;
pub const FORM_ROW_GAP: i32 = 38;
/// Scroll settings card height with only the preset row showing.
pub const SETTINGS_COLLAPSED: i32 = 80;
/// Scroll settings card height with every tuning row showing (Custom).
pub const SETTINGS_EXPANDED: i32 = 320;
/// Card-relative top of tuning row `i`; the two Scroll Test rows sit a little lower.
pub fn form_row_y(i: i32) -> i32 {
    FORM_ROW_START + i * FORM_ROW_GAP + if i >= 4 { 8 } else { 0 }
}
/// A tuning row shows only once the card is tall enough to hold all of it.
pub fn form_row_visible(i: i32, card_height: i32) -> bool {
    form_row_y(i) + 36 + 4 <= card_height
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameHit {
    Client,
    Caption,
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}
pub fn frame_hit(x: i32, y: i32, w: i32, h: i32, maximized: bool) -> FrameHit {
    use FrameHit::*;
    if !maximized {
        let left = x < 8;
        let right = x >= w - 8;
        let top = y < 8;
        let bottom = y >= h - 8;
        match (left, right, top, bottom) {
            (true, _, true, _) => return TopLeft,
            (_, true, true, _) => return TopRight,
            (true, _, _, true) => return BottomLeft,
            (_, true, _, true) => return BottomRight,
            (true, _, _, _) => return Left,
            (_, true, _, _) => return Right,
            (_, _, true, _) => return Top,
            (_, _, _, true) => return Bottom,
            _ => {}
        }
    }
    if y < CAPTION_HEIGHT && x < w - 160 {
        Caption
    } else {
        Client
    }
}
impl Shell {
    pub fn new(w: i32, h: i32) -> Self {
        Self::with_exclusions(w, h, 0)
    }
    pub fn with_exclusions(w: i32, h: i32, exclusions: usize) -> Self {
        let rail = if w >= 1100 {
            200
        } else if w < 760 {
            128
        } else {
            184
        };
        Self {
            rail,
            body: Rect::new(rail + 24, 64, w - rail - 48, h - 148),
            footer: Rect::new(rail + 24, h - 64, w - rail - 48, 44),
            exclusions,
            settings_height: SETTINGS_EXPANDED,
        }
    }
    /// Lay out with the Scroll settings card at this height (it animates).
    pub fn with_settings_height(mut self, height: i32) -> Self {
        self.settings_height = height.max(SETTINGS_COLLAPSED);
        self
    }
    /// Scroll settings, options, bounded exclusions, and full-surface Scroll Test.
    pub fn cards(&self) -> [Rect; 4] {
        let cards = self.cards_at_width(self.body.w);
        if cards.iter().map(|r| r.y + r.h).max().unwrap_or(0) + 8 > self.body.h {
            self.cards_at_width(self.body.w - 18)
        } else {
            cards
        }
    }
    fn cards_at_width(&self, w: i32) -> [Rect; 4] {
        let excluded_height = 96 + (self.exclusions.max(1).div_ceil(2).min(3) as i32) * 32;
        let sh = self.settings_height;
        if w >= 800 {
            let left = (w - 16) * 47 / 100;
            let right = w - left - 16;
            [
                Rect::new(0, 0, left, sh),
                Rect::new(0, sh + 16, left, 276),
                Rect::new(left + 16, 0, right, excluded_height),
                Rect::new(
                    left + 16,
                    excluded_height + 16,
                    right,
                    (self.body.h - excluded_height - 24)
                        .max(sh + 292 - excluded_height - 16)
                        .max(280),
                ),
            ]
        } else {
            [
                Rect::new(0, 0, w, sh),
                Rect::new(0, sh + 16, w, 276),
                Rect::new(0, sh + 308, w, excluded_height),
                Rect::new(0, sh + 324 + excluded_height, w, 400),
            ]
        }
    }
    pub fn exclusion(&self, index: usize) -> Rect {
        let c = self.exclusion_viewport();
        // Reserve a native scrollbar lane even when it is currently hidden.
        let width = (c.w - 26) / 2;
        Rect::new(
            (index % 2) as i32 * (width + 8),
            (index / 2) as i32 * 32,
            width,
            26,
        )
    }
    pub fn exclusion_viewport(&self) -> Rect {
        let c = self.cards()[2];
        Rect::new(c.x + 20, c.y + 88, c.w - 40, c.h - 96)
    }
    /// Input and Add together occupy 75% of the previous full-width form.
    pub fn exclusion_entry(&self) -> (Rect, Rect) {
        let c = self.cards()[2];
        let width = (c.w - 40) * 3 / 4;
        let right = c.x + c.w - 20;
        (
            Rect::new(right - width, c.y + 48, width - 56, 32),
            Rect::new(right - 48, c.y + 48, 48, 32),
        )
    }
    pub fn exclusion_content_height(&self) -> i32 {
        (self.exclusions.div_ceil(2) as i32 * 32 - 6).max(0)
    }
    pub fn content_height(&self) -> i32 {
        self.cards().iter().map(|r| r.y + r.h).max().unwrap_or(0) + 8
    }
}
