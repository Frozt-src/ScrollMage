use crate::native::wide;
use scrollpilot::layout::Rect;
use std::{mem::zeroed, ptr::null_mut};
use windows_sys::Win32::{
    Foundation::*,
    Graphics::{Gdi::*, GdiPlus::*},
    UI::WindowsAndMessaging::*,
};

pub const CANVAS: u32 = 0xf6f7fb;
pub const RAIL: u32 = 0xedeff5;
pub const WHITE: u32 = 0xffffff;
pub const INK: u32 = 0x10132d;
pub const MUTED: u32 = 0x636e82;
pub const BORDER: u32 = 0xdce1eb;
pub const ACCENT: u32 = 0x8e3555;
pub const ACCENT_TEXT: u32 = 0x792a48;
pub const ACCENT_PRESSED: u32 = 0x722541;
pub const SELECTED: u32 = 0xf3e4ea;
pub const TEST_OFF_WHITE: u32 = 0xf8f7f5;
/// Read-only value fields, such as a fixed preset's tuning.
pub const LOCKED_FIELD: u32 = 0xf3f4f8;
/// Text and glyphs on controls that cannot be used right now.
pub const DISABLED: u32 = 0xa3aab8;
pub fn color(rgb: u32) -> u32 {
    ((rgb & 255) << 16) | (rgb & 0xff00) | ((rgb >> 16) & 255)
}
pub fn blend(from: u32, to: u32, progress: f64) -> u32 {
    let channel = |shift: u32| {
        let a = f64::from((from >> shift) & 255);
        let b = f64::from((to >> shift) & 255);
        (a + (b - a) * progress.clamp(0., 1.)).round() as u32
    };
    channel(16) << 16 | channel(8) << 8 | channel(0)
}
pub unsafe fn outline(dc: HDC, r: Rect, radius: i32, scale: f64, rgb: u32) {
    let pen = CreatePen(PS_SOLID, px(1, scale).max(1), color(rgb));
    let old_pen = SelectObject(dc, pen);
    let old_brush = SelectObject(dc, GetStockObject(HOLLOW_BRUSH));
    RoundRect(
        dc,
        px(r.x, scale),
        px(r.y, scale),
        px(r.x + r.w, scale),
        px(r.y + r.h, scale),
        px(radius * 2, scale),
        px(radius * 2, scale),
    );
    SelectObject(dc, old_brush);
    SelectObject(dc, old_pen);
    DeleteObject(pen);
}
pub fn px(v: i32, s: f64) -> i32 {
    (f64::from(v) * s).round() as i32
}

pub struct GraphicsToken(usize);
impl GraphicsToken {
    pub unsafe fn start() -> Result<Self, String> {
        let mut token = 0;
        let input = GdiplusStartupInput {
            GdiplusVersion: 1,
            ..zeroed()
        };
        if GdiplusStartup(&mut token, &input, null_mut()) != 0 {
            return Err("Windows could not initialize the interface renderer.".into());
        }
        Result::Ok(Self(token))
    }
}
impl Drop for GraphicsToken {
    fn drop(&mut self) {
        unsafe { GdiplusShutdown(self.0) }
    }
}

pub struct Fonts {
    pub body: usize,
    pub small: usize,
    pub section: usize,
    pub display: usize,
    pub brand: usize,
    pub number: usize,
    /// Windows' own icon font, shipped with Windows 10 and 11.
    pub icon: usize,
}
unsafe fn font(face: &str, size: i32, weight: u32, scale: f64) -> usize {
    CreateFontW(
        -px(size, scale),
        0,
        0,
        0,
        weight as i32,
        0,
        0,
        0,
        DEFAULT_CHARSET as u32,
        OUT_DEFAULT_PRECIS as u32,
        CLIP_DEFAULT_PRECIS as u32,
        CLEARTYPE_QUALITY as u32,
        DEFAULT_PITCH as u32,
        wide(face).as_ptr(),
    ) as usize
}
impl Fonts {
    pub unsafe fn new(scale: f64) -> Self {
        Self {
            body: font("Segoe UI", 14, 400, scale),
            small: font("Segoe UI", 12, 400, scale),
            section: font("Segoe UI", 17, 600, scale),
            display: font("Segoe UI Variable Display", 34, 600, scale),
            brand: font("Segoe UI Variable Display", 24, 600, scale),
            number: font("Segoe UI", 14, 400, scale),
            icon: font("Segoe MDL2 Assets", 16, 400, scale),
        }
    }
}
impl Drop for Fonts {
    fn drop(&mut self) {
        unsafe {
            for h in [
                self.body,
                self.small,
                self.section,
                self.display,
                self.brand,
                self.number,
                self.icon,
            ] {
                if h != 0 {
                    DeleteObject(h as HGDIOBJ);
                }
            }
        }
    }
}

pub unsafe fn fill(dc: HDC, r: Rect, scale: f64, rgb: u32) {
    let brush = CreateSolidBrush(color(rgb));
    let rect = RECT {
        left: px(r.x, scale),
        top: px(r.y, scale),
        right: px(r.x + r.w, scale),
        bottom: px(r.y + r.h, scale),
    };
    FillRect(dc, &rect, brush);
    DeleteObject(brush);
}
pub unsafe fn round(dc: HDC, r: Rect, radius: i32, scale: f64, rgb: u32, stroke: Option<u32>) {
    if r.w <= 0 || r.h <= 0 {
        return;
    }
    let mut graphics = null_mut();
    if GdipCreateFromHDC(dc, &mut graphics) != 0 {
        return;
    }
    GdipSetSmoothingMode(graphics, SmoothingModeAntiAlias);
    let mut path = null_mut();
    GdipCreatePath(FillModeAlternate, &mut path);
    let x = (f64::from(r.x) * scale) as f32 + 0.5;
    let y = (f64::from(r.y) * scale) as f32 + 0.5;
    let w = (f64::from(r.w) * scale) as f32 - 1.;
    let h = (f64::from(r.h) * scale) as f32 - 1.;
    let d = (f64::from(radius * 2) * scale) as f32;
    for (a, b, c) in [
        (x, y, 180.),
        (x + w - d, y, 270.),
        (x + w - d, y + h - d, 0.),
        (x, y + h - d, 90.),
    ] {
        GdipAddPathArc(path, a, b, d, d, c, 90.);
    }
    GdipClosePathFigure(path);
    let mut brush = null_mut();
    GdipCreateSolidFill(0xff000000 | rgb, &mut brush);
    GdipFillPath(graphics, brush.cast(), path);
    GdipDeleteBrush(brush.cast());
    if let Some(stroke) = stroke {
        let mut pen = null_mut();
        GdipCreatePen1(0xff000000 | stroke, scale as f32, UnitPixel, &mut pen);
        GdipDrawPath(graphics, pen, path);
        GdipDeletePen(pen);
    }
    GdipDeletePath(path);
    GdipDeleteGraphics(graphics);
}
// Drawing primitive with two endpoints, scale, color, and stroke width.
#[allow(clippy::too_many_arguments)]
pub unsafe fn line(dc: HDC, x: i32, y: i32, x2: i32, y2: i32, scale: f64, rgb: u32, width: i32) {
    let mut graphics = null_mut();
    GdipCreateFromHDC(dc, &mut graphics);
    GdipSetSmoothingMode(graphics, SmoothingModeAntiAlias);
    let mut pen = null_mut();
    GdipCreatePen1(
        0xff000000 | rgb,
        (f64::from(width) * scale) as f32,
        UnitPixel,
        &mut pen,
    );
    GdipDrawLine(
        graphics,
        pen,
        (f64::from(x) * scale) as f32,
        (f64::from(y) * scale) as f32,
        (f64::from(x2) * scale) as f32,
        (f64::from(y2) * scale) as f32,
    );
    GdipDeletePen(pen);
    GdipDeleteGraphics(graphics);
}
pub unsafe fn label(dc: HDC, r: Rect, scale: f64, font: usize, rgb: u32, value: &str, flags: u32) {
    let previous = SelectObject(dc, font as HGDIOBJ);
    SetTextColor(dc, color(rgb));
    SetBkMode(dc, TRANSPARENT as i32);
    let mut rect = RECT {
        left: px(r.x, scale),
        top: px(r.y, scale),
        right: px(r.x + r.w, scale),
        bottom: px(r.y + r.h, scale),
    };
    let value = wide(value);
    DrawTextW(
        dc,
        value.as_ptr(),
        (value.len() - 1) as i32,
        &mut rect,
        flags | DT_NOPREFIX,
    );
    SelectObject(dc, previous);
}
/// Paint into one bitmap to keep resizing and the live preview free of erase flicker.
pub unsafe fn buffered(hwnd: HWND, paint: impl FnOnce(HDC, i32, i32)) {
    let mut ps = zeroed();
    let screen = BeginPaint(hwnd, &mut ps);
    let mut r = zeroed();
    GetClientRect(hwnd, &mut r);
    let dc = CreateCompatibleDC(screen);
    let bitmap = CreateCompatibleBitmap(screen, r.right.max(1), r.bottom.max(1));
    let previous = SelectObject(dc, bitmap);
    let surface = CreateSolidBrush(color(CANVAS));
    FillRect(dc, &r, surface);
    DeleteObject(surface);
    paint(dc, r.right, r.bottom);
    BitBlt(screen, 0, 0, r.right, r.bottom, dc, 0, 0, SRCCOPY);
    SelectObject(dc, previous);
    DeleteObject(bitmap);
    DeleteDC(dc);
    EndPaint(hwnd, &ps);
}
