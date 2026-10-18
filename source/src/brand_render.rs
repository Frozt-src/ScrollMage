//! The app and asset exporter use this same Windows vector renderer.
use crate::branding::{self, Command, Glyph, Pill};
use std::{
    mem::zeroed,
    ptr::{null, null_mut},
};
use windows_sys::Win32::Graphics::GdiPlus::*;

pub struct Renderer {
    token: usize,
    paths: Vec<*mut GpPath>,
    word: *mut GpPath,
}
fn check(status: Status) -> Result<(), String> {
    if status == 0 {
        Result::Ok(())
    } else {
        Err(format!("Logo renderer error {status}"))
    }
}
unsafe fn outline(glyph: &Glyph) -> Result<*mut GpPath, String> {
    let mut path = null_mut();
    check(GdipCreatePath(FillModeAlternate, &mut path))?;
    let mut previous = (0., 0.);
    for command in glyph.path {
        match *command {
            Command::Move(x, y) => {
                GdipStartPathFigure(path);
                previous = (x, y);
            }
            Command::Line(x, y) => {
                GdipAddPathLine(path, previous.0, previous.1, x, y);
                previous = (x, y);
            }
            Command::Curve(a, b, c, d, x, y) => {
                GdipAddPathBezier(path, previous.0, previous.1, a, b, c, d, x, y);
                previous = (x, y);
            }
            Command::Close => {
                GdipClosePathFigure(path);
            }
        }
    }
    let mut pen = null_mut();
    GdipCreatePen1(0xff000000, glyph.stroke, UnitPixel, &mut pen);
    GdipSetPenStartCap(pen, LineCapRound);
    GdipSetPenEndCap(pen, LineCapRound);
    GdipSetPenLineJoin(pen, LineJoinRound);
    let status = GdipWidenPath(path, pen, null_mut(), 0.025);
    GdipDeletePen(pen);
    if status != 0 {
        GdipDeletePath(path);
        check(status)?;
    }
    check(GdipFlattenPath(path, null_mut(), 0.025))?;
    Result::Ok(path)
}
unsafe fn rounded(p: Pill) -> *mut GpPath {
    let mut path = null_mut();
    GdipCreatePath(FillModeAlternate, &mut path);
    let x = p.x as f32;
    let y = p.y as f32;
    let w = p.width as f32;
    let h = p.height as f32;
    GdipAddPathArc(path, x, y, w, w, 180., 180.);
    GdipAddPathArc(path, x, y + h - w, w, w, 0., 180.);
    GdipClosePathFigure(path);
    path
}
impl Renderer {
    pub fn new() -> Result<Self, String> {
        unsafe {
            let mut token = 0;
            check(GdiplusStartup(
                &mut token,
                &GdiplusStartupInput {
                    GdiplusVersion: 1,
                    ..zeroed()
                },
                null_mut(),
            ))?;
            let mut renderer = Self {
                token,
                paths: Vec::new(),
                word: null_mut(),
            };
            for glyph in branding::GLYPHS {
                renderer.paths.push(outline(glyph)?);
            }
            check(GdipCreatePath(FillModeAlternate, &mut renderer.word))?;
            for &(index, offset) in branding::WORD {
                let mut path = null_mut();
                GdipClonePath(renderer.paths[index], &mut path);
                let mut matrix = null_mut();
                GdipCreateMatrix2(1., 0., 0., 1., offset, 0., &mut matrix);
                GdipTransformPath(path, matrix);
                GdipAddPathPath(renderer.word, path, 0);
                GdipDeleteMatrix(matrix);
                GdipDeletePath(path);
            }
            Result::Ok(renderer)
        }
    }
    /// Caller owns the graphics object. The counter uses the supplied host surface color.
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
        GdipSetSmoothingMode(graphics, SmoothingModeAntiAlias);
        let mut transform = null_mut();
        GdipCreateMatrix2(scale, 0., 0., scale, x, y, &mut transform);
        GdipSetWorldTransform(graphics, transform);
        let mut brush = null_mut();
        GdipCreateSolidFill(ink, &mut brush);
        GdipFillPath(graphics, brush.cast(), self.word);
        GdipDeleteBrush(brush.cast());
        let path = rounded(branding::pill(time, animate));
        GdipSetCompositingMode(graphics, CompositingModeSourceCopy);
        GdipCreateSolidFill(surface, &mut brush);
        GdipFillPath(graphics, brush.cast(), path);
        GdipSetCompositingMode(graphics, CompositingModeSourceOver);
        GdipDeleteBrush(brush.cast());
        GdipDeleteMatrix(transform);
        GdipDeletePath(path);
        GdipResetWorldTransform(graphics);
    }
    pub fn contours(&self) -> Result<serde_json::Value, String> {
        unsafe {
            let mut glyphs = Vec::new();
            for (glyph, &path) in branding::GLYPHS.iter().zip(&self.paths) {
                let mut count = 0;
                check(GdipGetPointCount(path, &mut count))?;
                let mut points = vec![PointF { X: 0., Y: 0. }; count as usize];
                let mut types = vec![0u8; count as usize];
                check(GdipGetPathPoints(path, points.as_mut_ptr(), count))?;
                check(GdipGetPathTypes(path, types.as_mut_ptr(), count))?;
                let mut contours: Vec<Vec<[f32; 2]>> = Vec::new();
                let mut current = Vec::new();
                for (point, kind) in points.into_iter().zip(types) {
                    if kind & 7 == 0 && !current.is_empty() {
                        contours.push(std::mem::take(&mut current));
                    }
                    if kind & 7 == 3 {
                        return Err("Expected a flattened outline.".into());
                    }
                    current.push([point.X, point.Y]);
                    if kind & 0x80 != 0 {
                        contours.push(std::mem::take(&mut current));
                    }
                }
                if !current.is_empty() {
                    contours.push(current);
                }
                glyphs.push(serde_json::json!({"name":glyph.name,"character":glyph.character,"advance":glyph.advance,"contours":contours}));
            }
            Result::Ok(
                serde_json::json!({"width":branding::WIDTH,"height":branding::HEIGHT,"glyphs":glyphs,"word":branding::WORD,"pill":branding::pill(0.,false)}),
            )
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn rgba(
        &self,
        width: i32,
        height: i32,
        x: f32,
        y: f32,
        scale: f32,
        time: f64,
        animate: bool,
        surface: u32,
    ) -> Result<Vec<u8>, String> {
        render_rgba(width, height, surface, |graphics| unsafe {
            self.draw(graphics, x, y, scale, time, animate, 0xff000000, surface)
        })
    }
}
impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            for path in &self.paths {
                GdipDeletePath(*path);
            }
            if !self.word.is_null() {
                GdipDeletePath(self.word);
            }
            GdiplusShutdown(self.token);
        }
    }
}

pub(crate) fn render_rgba(
    width: i32,
    height: i32,
    surface: u32,
    paint: impl FnOnce(*mut GpGraphics),
) -> Result<Vec<u8>, String> {
    unsafe {
        // PixelFormat32bppARGB from the Windows GDI+ PixelFormat definition.
        const ARGB32: i32 = 0x26200a;
        let mut bitmap = null_mut();
        check(GdipCreateBitmapFromScan0(
            width,
            height,
            0,
            ARGB32,
            null(),
            &mut bitmap,
        ))?;
        let mut graphics = null_mut();
        check(GdipGetImageGraphicsContext(bitmap.cast(), &mut graphics))?;
        GdipGraphicsClear(graphics, surface);
        paint(graphics);
        GdipDeleteGraphics(graphics);
        let rect = Rect {
            X: 0,
            Y: 0,
            Width: width,
            Height: height,
        };
        let mut data = BitmapData::default();
        check(GdipBitmapLockBits(
            bitmap,
            &rect,
            ImageLockModeRead as u32,
            ARGB32,
            &mut data,
        ))?;
        let mut output = Vec::with_capacity((width * height * 4) as usize);
        for row in 0..height {
            let source = std::slice::from_raw_parts(
                (data.Scan0 as *const u8).offset((row * data.Stride) as isize),
                (width * 4) as usize,
            );
            for p in source.as_chunks::<4>().0 {
                output.extend_from_slice(&[p[2], p[1], p[0], p[3]]);
            }
        }
        GdipBitmapUnlockBits(bitmap, &mut data);
        GdipDisposeImage(bitmap.cast());
        Result::Ok(output)
    }
}
