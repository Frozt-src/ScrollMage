use scrollpilot::brand_lockup::Lockup;
use std::{fs, io::Write, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(std::env::args().nth(1).ok_or("Provide an output folder")?);
    fs::create_dir_all(&output)?;
    let logo = Lockup::new()?;
    fs::write(
        output.join("lockup-transparent.rgba"),
        logo.rgba(1320, 440, 0., false, 0)?,
    )?;
    fs::write(
        output.join("lockup-sidebar.rgba"),
        logo.rgba(216, 72, 0., false, 0xffedeff5)?,
    )?;
    fs::write(
        output.join("lockup-sidebar-168.rgba"),
        logo.rgba(168, 64, 0., false, 0xffedeff5)?,
    )?;
    fs::write(
        output.join("lockup-sidebar-160.rgba"),
        logo.rgba(160, 64, 0., false, 0xffedeff5)?,
    )?;
    fs::write(
        output.join("lockup-sidebar-128.rgba"),
        logo.rgba(128, 64, 0., false, 0xffedeff5)?,
    )?;
    let mut frames = std::io::BufWriter::new(fs::File::create(output.join("lockup-frames.rgba"))?);
    for frame in 0..80 {
        frames.write_all(&logo.rgba(990, 330, frame as f64 / 20., true, 0xffedeff5)?)?;
    }
    frames.flush()?;
    let motion: Vec<_> = (0..=80)
        .map(|frame| scrollpilot::logo_motion::wheel(frame as f64 / 20., true))
        .collect();
    fs::write(
        output.join("motion.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "cycle":scrollpilot::logo_motion::CYCLE,"frames":motion,
            "rest":scrollpilot::logo_motion::wheel(0.,false)
        }))?,
    )?;
    println!("Rendered static and 80 animated lockup frames with the app's native renderer.");
    Ok(())
}
