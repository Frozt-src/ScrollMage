use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub preset: Preset,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_tuning: Option<Tuning>,
    pub distance: f64,
    pub omega: f64,
    pub acceleration_ms: f64,
    pub acceleration_max: f64,
    pub friction: f64,
    pub elastic_omega: f64,
    pub reverse: bool,
    pub horizontal: bool,
    pub exclusions: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<Profile>,
    pub incompatible_processes: Vec<String>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Preset {
    #[default]
    Default,
    MacosStyle,
    SuperSmooth,
}
impl Preset {
    pub const ALL: [Self; 3] = [Self::Default, Self::MacosStyle, Self::SuperSmooth];
    pub fn fixed(self) -> bool {
        self != Self::Default
    }
    /// Canonical values for this preset; Default can be edited in Settings.
    pub fn tuning(self) -> Tuning {
        let [distance, omega, acceleration_ms, acceleration_max, friction, elastic_omega] =
            match self {
                Self::Default => [120., 30., 70., 1., 4., 22.],
                Self::MacosStyle => [120., 24., 100., 1.5, 4.5, 24.],
                Self::SuperSmooth => [120., 18., 100., 1., 5., 20.],
            };
        Tuning {
            distance,
            omega,
            acceleration_ms,
            acceleration_max,
            friction,
            elastic_omega,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::MacosStyle => "macOS style",
            Self::SuperSmooth => "Super smooth",
        }
    }
    fn wheel(self) -> WheelTuning {
        WheelTuning {
            free_spin: self == Self::SuperSmooth,
            ..self.tuning().wheel()
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tuning {
    pub distance: f64,
    pub omega: f64,
    pub acceleration_ms: f64,
    pub acceleration_max: f64,
    pub friction: f64,
    pub elastic_omega: f64,
}
impl Tuning {
    fn validate(self) -> Result<(), String> {
        for (label, value, min, max) in [
            ("Distance", self.distance, 1., 960.),
            ("Spring response", self.omega, 10., 80.),
            ("Acceleration window", self.acceleration_ms, 10., 500.),
            ("Maximum acceleration", self.acceleration_max, 1., 10.),
            ("Preview friction", self.friction, 1., 12.),
            ("Elastic response", self.elastic_omega, 10., 50.),
        ] {
            if !value.is_finite() || value < min || value > max {
                return Err(format!("{label} must be between {min} and {max}."));
            }
        }
        Ok(())
    }
    fn wheel(self) -> WheelTuning {
        WheelTuning {
            distance: self.distance,
            omega: self.omega,
            acceleration_ms: self.acceleration_ms,
            acceleration_max: self.acceleration_max,
            free_spin: false,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Profile {
    pub executable: String,
    #[serde(default)]
    pub preset: Option<Preset>,
    pub distance: f64,
    pub omega: f64,
    #[serde(default)]
    pub bypass: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WheelTuning {
    pub distance: f64,
    pub omega: f64,
    pub acceleration_ms: f64,
    pub acceleration_max: f64,
    pub free_spin: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            preset: Preset::Default,
            default_tuning: None,
            distance: 120.,
            omega: 30.,
            acceleration_ms: 70.,
            acceleration_max: 1.,
            friction: 4.,
            elastic_omega: 22.,
            reverse: false,
            horizontal: true,
            exclusions: ["Diablo IV.exe", "firefox.exe", "chrome.exe", "msedge.exe"]
                .into_iter()
                .map(String::from)
                .collect(),
            profiles: Vec::new(),
            incompatible_processes: Vec::new(),
        }
    }
}
impl Settings {
    pub fn tuning(&self) -> Tuning {
        Tuning {
            distance: self.distance,
            omega: self.omega,
            acceleration_ms: self.acceleration_ms,
            acceleration_max: self.acceleration_max,
            friction: self.friction,
            elastic_omega: self.elastic_omega,
        }
    }
    fn set_tuning(&mut self, tuning: Tuning) {
        self.distance = tuning.distance;
        self.omega = tuning.omega;
        self.acceleration_ms = tuning.acceleration_ms;
        self.acceleration_max = tuning.acceleration_max;
        self.friction = tuning.friction;
        self.elastic_omega = tuning.elastic_omega;
    }
    /// Remember the editable Default while switching between fixed presets.
    pub fn select_preset(&mut self, preset: Preset) {
        if self.preset == Preset::Default {
            if preset == Preset::Default {
                return;
            }
            self.default_tuning = Some(self.tuning());
        }
        self.preset = preset;
        self.set_tuning(if preset.fixed() {
            preset.tuning()
        } else {
            self.default_tuning.unwrap_or_else(|| preset.tuning())
        });
    }
    /// Our Windows wheel tuning, not Apple's private scrolling constants.
    /// Friction and elastic response affect the owned test surface only.
    pub fn apply_preset(&mut self, preset: Preset) {
        self.select_preset(preset);
    }
    /// Normalize legacy preset values without discarding per-app overrides.
    pub fn migrate_profiles(&mut self) {
        if self.preset.fixed() {
            let canonical = self.preset.tuning();
            if self.default_tuning.is_none() && self.tuning() != canonical {
                self.default_tuning = Some(self.tuning());
            }
            self.set_tuning(canonical);
        }
        for profile in &mut self.profiles {
            if let Some(preset) = profile.preset.filter(|preset| preset.fixed()) {
                profile.distance = preset.tuning().distance;
                profile.omega = preset.tuning().omega;
            }
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        self.tuning().validate()?;
        if let Some(tuning) = self.default_tuning {
            tuning.validate()?;
        }
        if self.exclusions.len() > 128
            || self.incompatible_processes.len() > 128
            || self
                .exclusions
                .iter()
                .chain(&self.incompatible_processes)
                .any(|x| !valid_executable(x))
        {
            return Err("Use up to 128 executable names, one per line (example: game.exe).".into());
        }
        if self.profiles.len() > 128 {
            return Err("Use up to 128 app profiles.".into());
        }
        for (i, p) in self.profiles.iter().enumerate() {
            if !valid_executable(&p.executable)
                || !p.distance.is_finite()
                || !(1. ..=960.).contains(&p.distance)
                || !p.omega.is_finite()
                || !(10. ..=80.).contains(&p.omega)
            {
                return Err(
                    "Each profile needs an .exe name, distance 1 to 960, and response 10 to 80."
                        .into(),
                );
            }
            if self.profiles[..i]
                .iter()
                .any(|q| q.executable.eq_ignore_ascii_case(&p.executable))
            {
                return Err("Use only one profile for each app.".into());
            }
        }
        Ok(())
    }
    pub fn excluded(&self, name: &str) -> bool {
        self.exclusions.iter().any(|x| x.eq_ignore_ascii_case(name))
            || self
                .profiles
                .iter()
                .any(|p| p.bypass && p.executable.eq_ignore_ascii_case(name))
    }
    pub fn wheel(&self) -> WheelTuning {
        if self.preset.fixed() {
            self.preset.wheel()
        } else {
            self.tuning().wheel()
        }
    }
    pub fn for_app(&self, name: &str) -> WheelTuning {
        let mut tuning = self.wheel();
        if let Some(p) = self
            .profiles
            .iter()
            .find(|p| !p.bypass && p.executable.eq_ignore_ascii_case(name))
        {
            if let Some(preset) = p.preset.filter(|preset| preset.fixed()) {
                return preset.wheel();
            }
            tuning.distance = p.distance;
            tuning.omega = p.omega;
            tuning.free_spin = false;
        }
        tuning
    }
}
fn valid_executable(name: &str) -> bool {
    name.len() > 4
        && name.len() <= 260
        && name.to_ascii_lowercase().ends_with(".exe")
        && !name.contains(['\\', '/', ':', '*', '?', '"', '<', '>', '|'])
        && !name.chars().any(char::is_control)
}
pub fn parse_profiles(text: &str) -> Result<Vec<Profile>, String> {
    let mut profiles = Vec::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let parts: Vec<_> = line.split('|').map(str::trim).collect();
        let p = match parts.as_slice() {
            [name, "off"] => Profile {
                executable: (*name).into(),
                preset: None,
                distance: 120.,
                omega: 30.,
                bypass: true,
            },
            [name, preset @ ("macos_style" | "super_smooth")] => {
                let preset = if *preset == "macos_style" {
                    Preset::MacosStyle
                } else {
                    Preset::SuperSmooth
                };
                let tuning = preset.tuning();
                Profile {
                    executable: (*name).into(),
                    preset: Some(preset),
                    distance: tuning.distance,
                    omega: tuning.omega,
                    bypass: false,
                }
            }
            [name, distance, omega] => Profile {
                executable: (*name).into(),
                preset: None,
                distance: distance
                    .parse()
                    .map_err(|_| "Profile distance must be a number.")?,
                omega: omega
                    .parse()
                    .map_err(|_| "Profile response must be a number.")?,
                bypass: false,
            },
            _ => return Err("Profiles: app.exe | distance | response, app.exe | macos_style, app.exe | super_smooth, or app.exe | off".into()),
        };
        profiles.push(p);
    }
    let s = Settings {
        profiles,
        ..Settings::default()
    };
    s.validate()?;
    Ok(s.profiles)
}
pub fn format_profiles(profiles: &[Profile]) -> String {
    profiles
        .iter()
        .map(|p| {
            if p.bypass {
                format!("{} | off", p.executable)
            } else if let Some(preset) = p.preset.filter(|preset| preset.fixed()) {
                let name = match preset {
                    Preset::MacosStyle => "macos_style",
                    Preset::SuperSmooth => "super_smooth",
                    Preset::Default => unreachable!(),
                };
                format!("{} | {}", p.executable, name)
            } else {
                format!("{} | {} | {}", p.executable, p.distance, p.omega)
            }
        })
        .collect::<Vec<_>>()
        .join("\r\n")
}
pub fn load(path: &std::path::Path) -> Result<Settings, String> {
    use std::io::Read;
    match std::fs::File::open(path) {
        Ok(file) => {
            let mut bytes = Vec::new();
            file.take(MAX_SETTINGS_BYTES as u64 + 1)
                .read_to_end(&mut bytes)
                .map_err(|e| format!("Cannot read settings: {e}"))?;
            if bytes.len() > MAX_SETTINGS_BYTES {
                return Err("Settings file is too large; original file preserved.".into());
            }
            let mut s: Settings = serde_json::from_slice(&bytes).map_err(|_| {
                "Settings file could not be read; original file preserved. Defaults loaded."
                    .to_string()
            })?;
            s.validate()?;
            s.migrate_profiles();
            s.validate()?;
            Ok(s)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(e) => Err(format!("Cannot read settings: {e}")),
    }
}
const MAX_SETTINGS_BYTES: usize = 65536;
static SAVE_SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

struct PendingSettings {
    path: std::path::PathBuf,
    file: Option<std::fs::File>,
}
impl Drop for PendingSettings {
    fn drop(&mut self) {
        // Close first: Windows does not permit removal of an open temporary file.
        self.file.take();
        let _ = std::fs::remove_file(&self.path);
    }
}
pub fn save(path: &std::path::Path, settings: &Settings) -> Result<(), String> {
    use std::io::Write;
    settings.validate()?;
    let mut normalized = settings.clone();
    normalized.migrate_profiles();
    normalized.validate()?;
    let bytes = serde_json::to_vec_pretty(&normalized).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_SETTINGS_BYTES {
        return Err("Settings exceed the 64 KiB limit; existing file preserved.".into());
    }
    let sequence = SAVE_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let temp = path.with_extension(format!("{}.{}.tmp", std::process::id(), sequence));
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .map_err(|e| e.to_string())?;
    let mut pending = PendingSettings {
        path: temp.clone(),
        file: Some(file),
    };
    let f = pending.file.as_mut().unwrap();
    f.write_all(&bytes)
        .and_then(|_| f.sync_all())
        .map_err(|e| e.to_string())?;
    pending.file.take();
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::*;
        let from: Vec<_> = temp.as_os_str().encode_wide().chain(Some(0)).collect();
        let to: Vec<_> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        if unsafe {
            MoveFileExW(
                from.as_ptr(),
                to.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        } == 0
        {
            return Err(std::io::Error::last_os_error().to_string());
        }
    }
    #[cfg(not(windows))]
    std::fs::rename(temp, path).map_err(|e| e.to_string())?;
    Ok(())
}
