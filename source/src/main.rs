#![windows_subsystem = "windows"]
mod native;
mod ui;
fn main() {
    let _guard = match ui::single_instance() {
        Ok(h) => h,
        Err(_) => return,
    };
    let path = match std::env::current_exe() {
        Ok(p) => p.with_file_name("settings.json"),
        Err(_) => return,
    };
    let (settings, note) = match scrollpilot::settings::load(&path) {
        Ok(s) => (s, String::new()),
        Err(e) => (Default::default(), e),
    };
    let args: Vec<_> = std::env::args().collect();
    let paused = args.iter().any(|a| a == "--paused");
    let tray = args.iter().any(|a| a == "--tray");
    let result = match native::Runtime::start(settings, !paused && note.is_empty()) {
        Ok(runtime) => ui::run(&runtime, path.clone(), note, tray),
        Err(e) => Err(e),
    };
    if let Err(e) = result {
        let _ = std::fs::write(path.with_file_name("startup-error.txt"), e);
    }
}
