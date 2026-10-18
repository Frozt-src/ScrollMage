//! Native vector lockup reconstructed from the supplied tail-connected logo.
use crate::{brand_render::render_rgba, branding::Pill, logo_motion};
use std::ptr::null_mut;
use windows_sys::Win32::Graphics::GdiPlus::*;

pub const WIDTH: f32 = 1320.;
pub const HEIGHT: f32 = 440.;
const PURPLE: u32 = 0xff8818ff;

#[derive(serde::Deserialize)]
struct Outlines {
    ink: Vec<Vec<[f32; 2]>>,
    mage: Vec<Vec<[f32; 2]>>,
}
fn checked(status: Status) -> Result<(), String> {
    if status == 0 {
        Result::Ok(())
    } else {
        Err(format!("Logo renderer error {status}"))
    }
}
unsafe fn outline(contours: &[Vec<[f32; 2]>]) -> Result<*mut GpPath, String> {
    let mut path = null_mut();
    checked(GdipCreatePath(FillModeAlternate, &mut path))?;
    for contour in contours {
        let points: Vec<_> = contour
            .iter()
            .map(|p| PointF { X: p[0], Y: p[1] })
            .collect();
        GdipStartPathFigure(path);
        let status = GdipAddPathPolygon(path, points.as_ptr(), points.len() as i32);
        if status != 0 {
            GdipDeletePath(path);
            checked(status)?;
        }
    }
    Result::Ok(path)
}
unsafe fn capsule(p: Pill) -> Result<*mut GpPath, String> {
    let mut path = null_mut();
    checked(GdipCreatePath(FillModeAlternate, &mut path))?;
    let (x, y, w, h) = (p.x as f32, p.y as f32, p.width as f32, p.height as f32);
    let first = GdipAddPathArc(path, x, y, w, w, 180., 180.);
    let second = GdipAddPathArc(path, x, y + h - w, w, w, 0., 180.);
    if first != 0 || second != 0 {
        GdipDeletePath(path);
        checked(if first != 0 { first } else { second })?;
    }
    GdipClosePathFigure(path);
    Result::Ok(path)
}

pub struct Lockup {
    token: usize,
    ink: *mut GpPath,
    mage: *mut GpPath,
}
impl Lockup {
    pub fn new() -> Result<Self, String> {
        let data: Outlines =
            serde_json::from_str(include_str!("../assets/brand/lockup-v2.paths.json"))
                .map_err(|e| format!("Could not read embedded logo outlines: {e}"))?;
        unsafe {
            let mut token = 0;
            checked(GdiplusStartup(
                &mut token,
                &GdiplusStartupInput {
                    GdiplusVersion: 1,
                    ..std::mem::zeroed()
                },
                null_mut(),
            ))?;
            let mut logo = Self {
                token,
                ink: null_mut(),
                mage: null_mut(),
            };
            logo.ink = outline(&data.ink)?;
            logo.mage = outline(&data.mage)?;
            let stem = capsule(Pill {
                x: 1161.,
                y: 18.,
                width: 49.,
                height: 256.,
            })?;
            let status = GdipAddPathPath(logo.ink, stem, 0);
            GdipDeletePath(stem);
            checked(status)?;
            Result::Ok(logo)
        }
    }
    /// # Safety
    /// graphics must be a live GDI+ graphics object on the calling thread.
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn draw(
        &self,
        graphics: *mut GpGraphics,
        x: f32,
        y: f32,
        scale: f32,
        time: f64,
        animate: bool,
        ink: u32,
        surface: u32,
    ) {
        let mut saved = 0;
        if GdipSaveGraphics(graphics, &mut saved) != 0 {
            return;
        }
        GdipSetSmoothingMode(graphics, SmoothingModeAntiAlias);
        let mut matrix = null_mut();
        GdipCreateMatrix2(scale, 0., 0., scale, x, y, &mut matrix);
        GdipSetWorldTransform(graphics, matrix);
        GdipDeleteMatrix(matrix);
        for (path, color) in [(self.ink, ink), (self.mage, PURPLE)] {
            let mut brush = null_mut();
            if GdipCreateSolidFill(color, &mut brush) == 0 {
                GdipFillPath(graphics, brush.cast(), path);
                GdipDeleteBrush(brush.cast());
            }
        }
        if let Result::Ok(slot) = capsule(logo_motion::wheel(time, animate)) {
            GdipSetCompositingMode(graphics, CompositingModeSourceCopy);
            let mut brush = null_mut();
            if GdipCreateSolidFill(surface, &mut brush) == 0 {
                GdipFillPath(graphics, brush.cast(), slot);
                GdipDeleteBrush(brush.cast());
            }
            GdipDeletePath(slot);
        }
        GdipRestoreGraphics(graphics, saved);
    }
    pub fn rgba(
        &self,
        width: i32,
        height: i32,
        time: f64,
        animate: bool,
        surface: u32,
    ) -> Result<Vec<u8>, String> {
        let scale = (width as f32 / WIDTH).min(height as f32 / HEIGHT);
        render_rgba(width, height, surface, |graphics| unsafe {
            self.draw(
                graphics,
                (width as f32 - WIDTH * scale) / 2.,
                (height as f32 - HEIGHT * scale) / 2.,
                scale,
                time,
                animate,
                0xff000000,
                surface,
            )
        })
    }
}
impl Drop for Lockup {
    fn drop(&mut self) {
        unsafe {
            if !self.ink.is_null() {
                GdipDeletePath(self.ink);
            }
            if !self.mage.is_null() {
                GdipDeletePath(self.mage);
            }
            GdiplusShutdown(self.token);
        }
    }
}
