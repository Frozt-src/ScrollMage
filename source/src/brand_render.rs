//! Off-screen GDI+ rendering to RGBA bytes, shared by the logo exporter and tests.
use std::ptr::{null, null_mut};
use windows_sys::Win32::Graphics::GdiPlus::*;

fn check(status: Status) -> Result<(), String> {
    if status == 0 {
        Result::Ok(())
    } else {
        Err(format!("Logo renderer error {status}"))
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
