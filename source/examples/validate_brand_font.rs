use std::{fs, mem::zeroed, ptr::null};
use windows_sys::Win32::Graphics::Gdi::*;
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let bytes = fs::read(std::env::args().nth(1).ok_or("Provide a font file")?)?;
        let mut count = 0u32;
        let handle = AddFontMemResourceEx(
            bytes.as_ptr().cast(),
            bytes.len() as u32,
            null(),
            &raw mut count,
        );
        if handle.is_null() {
            return Err("Windows rejected the lettering font".into());
        }
        let dc = CreateCompatibleDC(std::ptr::null_mut());
        let font = CreateFontW(
            -80,
            0,
            0,
            0,
            400,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            0,
            0,
            0,
            0,
            wide("Scroll Mage Scroll").as_ptr(),
        );
        let old = SelectObject(dc, font);
        let mut family = [0u16; 64];
        let length = GetTextFaceW(dc, family.len() as i32, family.as_mut_ptr());
        let family = String::from_utf16_lossy(&family[..length.saturating_sub(1) as usize]);
        let word = wide("scroll");
        let mut glyphs = [0u16; 6];
        let result = GetGlyphIndicesW(
            dc,
            word.as_ptr(),
            6,
            glyphs.as_mut_ptr(),
            GGI_MARK_NONEXISTING_GLYPHS,
        );
        let mut metrics = zeroed();
        GetTextMetricsW(dc, &mut metrics);
        SelectObject(dc, old);
        DeleteObject(font);
        DeleteDC(dc);
        RemoveFontMemResourceEx(handle);
        if family != "Scroll Mage Scroll" || result == u32::MAX || glyphs.contains(&0xffff) {
            return Err(format!("Font validation failed: {family:?} {glyphs:?}").into());
        }
        println!(
            "{}",
            serde_json::json!({"windows_memory_font_load":"passed","family":family,"font_faces":count,"glyph_indices":glyphs,"permanent_font_install":false})
        );
        Ok(())
    }
}
