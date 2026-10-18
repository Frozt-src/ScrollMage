use scrollpilot::{brand_render::Renderer, branding};
use std::{fs, io::Write, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(std::env::args().nth(1).ok_or("Provide an output folder")?);
    fs::create_dir_all(&output)?;
    let renderer = Renderer::new()?;
    fs::write(
        output.join("outlines.json"),
        serde_json::to_vec_pretty(&renderer.contours()?)?,
    )?;
    let frames = 200;
    let mut raw = std::io::BufWriter::new(fs::File::create(output.join("frames.rgba"))?);
    let mut samples = Vec::new();
    for i in 0..=frames {
        let time = i as f64 / 50.;
        samples.push(branding::pill(time, true));
        if i < frames {
            raw.write_all(&renderer.rgba(672, 402, 132., 145., 1., time, true, 0xffe0e0e0)?)?;
        }
    }
    raw.flush()?;
    fs::write(
        output.join("animation.json"),
        serde_json::to_vec_pretty(&samples)?,
    )?;
    fs::write(
        output.join("static.rgba"),
        renderer.rgba(672, 402, 132., 145., 1., 0., false, 0xffe0e0e0)?,
    )?;
    fs::write(
        output.join("transparent.rgba"),
        renderer.rgba(460, 140, 0., 0., 1., 0., false, 0)?,
    )?;
    println!("Rendered {frames} frames with the same vector renderer used by the app.");
    Ok(())
}
