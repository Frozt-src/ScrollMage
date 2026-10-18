use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub preset: Preset,
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
    pub fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::MacosStyle => "macOS style",
            Self::SuperSmooth => "Super smooth",
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Profile {
    pub executable: String,
    pub distance: f64,
    pub omega: f64,
    #[serde(default)]
    pub bypass: bool,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct WheelTuning {
    pub distance: f64,
    pub omega: f64,
    pub acceleration_ms: f64,
    pub acceleration_max: f64,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            preset: Preset::Default,
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
    /// Our Windows wheel tuning, not Apple's private scrolling constants.
    /// A slower critical spring spreads each notch across smaller output steps.
    /// Friction and elastic response affect the owned test surface only.
    pub fn apply_preset(&mut self, preset: Preset) {
        let values = match preset {
            Preset::Default => [120., 30., 70., 1., 4., 22.],
            Preset::MacosStyle => [120., 24., 100., 1.5, 4.5, 24.],
            Preset::SuperSmooth => [120., 18., 100., 1., 5., 20.],
        };
        self.preset = preset;
        [
            self.distance,
            self.omega,
            self.acceleration_ms,
            self.acceleration_max,
            self.friction,
            self.elastic_omega,
        ] = values;
    }
    /// Preserve legacy bypass choices as visible exclusions; use global tuning.
    pub fn migrate_profiles(&mut self) {
        for profile in self.profiles.drain(..) {
            if profile.bypass
                && !self
                    .exclusions
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(&profile.executable))
            {
                self.exclusions.push(profile.executable);
            }
        }
    }
    pub fn validate(&self) -> Result<(), String> {
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
        WheelTuning {
            distance: self.distance,
            omega: self.omega,
            acceleration_ms: self.acceleration_ms,
            acceleration_max: self.acceleration_max,
        }
    }
    pub fn for_app(&self, name: &str) -> WheelTuning {
        let mut tuning = self.wheel();
        if let Some(p) = self
            .profiles
            .iter()
            .find(|p| p.executable.eq_ignore_ascii_case(name))
        {
            tuning.distance = p.distance;
            tuning.omega = p.omega;
        }
        tuning
    }
}
fn valid_executable(name: &str) -> bool {
    name.len() > 4
        && name.len() <= 260
        && name.to_ascii_lowercase().ends_with(".exe")
        && !name.contains(['\\', '/', '\0', '\r', '\n', '|'])
}
pub fn parse_profiles(text: &str) -> Result<Vec<Profile>, String> {
    let mut profiles = Vec::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let parts: Vec<_> = line.split('|').map(str::trim).collect();
        let p = match parts.as_slice() {
            [name, "off"] => Profile {
                executable: (*name).into(),
                distance: 120.,
                omega: 30.,
                bypass: true,
            },
            [name, distance, omega] => Profile {
                executable: (*name).into(),
                distance: distance
                    .parse()
                    .map_err(|_| "Profile distance must be a number.")?,
                omega: omega
                    .parse()
                    .map_err(|_| "Profile response must be a number.")?,
                bypass: false,
            },
            _ => return Err("Profiles: app.exe | distance | response, or app.exe | off".into()),
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
            } else {
                format!("{} | {} | {}", p.executable, p.distance, p.omega)
            }
        })
        .collect::<Vec<_>>()
        .join("\r\n")
}
pub fn load(path: &std::path::Path) -> Result<Settings, String> {
    match std::fs::read(path) {
        Ok(bytes) => {
            if bytes.len() > 65536 {
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
pub fn save(path: &std::path::Path, settings: &Settings) -> Result<(), String> {
    use std::io::Write;
    settings.validate()?;
    let temp = path.with_extension(format!("{}.tmp", std::process::id()));
    let mut f = std::fs::File::create(&temp).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(settings).map_err(|e| e.to_string())?;
    f.write_all(&bytes)
        .and_then(|_| f.sync_all())
        .map_err(|e| e.to_string())?;
    drop(f);
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
