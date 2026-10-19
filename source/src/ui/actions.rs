use super::*;
impl Ui {
    pub(super) unsafe fn toggle(&mut self) {
        if self.shared.enabled.load(Ordering::Acquire) {
            self.shared.pause();
        } else {
            if self
                .shared
                .settings
                .read()
                .map(|s| crate::native::conflicting_engine_running(&s))
                .unwrap_or(true)
            {
                self.note =
                    "Pause the configured competing scroll app before enabling Scroll Mage.".into();
                self.update();
                return;
            }
            if !self.shared.hook_live.load(Ordering::Acquire) {
                self.note = "Mouse hook is unavailable. Restart Scroll Mage to retry.".into();
                self.update();
                return;
            }
            if let Ok(mut f) = self.shared.fault.lock() {
                f.clear();
            }
            self.shared.refresh_target();
            self.shared.enabled.store(true, Ordering::Release);
        }
        self.note.clear();
        self.tray(NIM_MODIFY);
        self.update();
    }
    pub(super) unsafe fn apply(&mut self) {
        let mut values = [0.; 6];
        for (i, v) in values.iter_mut().enumerate() {
            match text(self.inputs[i]).trim().parse::<f64>() {
                Ok(n) => *v = n,
                Err(_) => {
                    self.note = "Enter a number in each scroll setting.".into();
                    self.update();
                    return;
                }
            }
        }
        let mut opts = Settings {
            preset: self.preset,
            default_tuning: self.shared.settings.read().unwrap().default_tuning,
            distance: values[0],
            omega: values[1],
            acceleration_ms: values[2],
            acceleration_max: values[3],
            friction: values[4],
            elastic_omega: values[5],
            profiles: self.shared.settings.read().unwrap().profiles.clone(),
            incompatible_processes: self
                .shared
                .settings
                .read()
                .map(|s| s.incompatible_processes.clone())
                .unwrap_or_default(),
            horizontal: checked(self.horizontal),
            reverse: checked(self.reverse),
            exclusions: self.excluded_apps.clone(),
        };
        if self.preset.fixed() {
            opts.apply_preset(self.preset);
        } else {
            opts.default_tuning = Some(opts.tuning());
        }
        if let Err(e) = settings::save(&self.path, &opts) {
            self.note = e;
            self.update();
            return;
        }
        if let Ok(mut s) = self.shared.settings.write() {
            *s = opts.clone();
        }
        self.show_tuning(&opts);
        self.shared.refresh_target();
        let requested = checked(self.startup);
        self.note = if requested != startup_enabled() {
            match set_startup(requested) {
                Ok(()) => "Settings saved.".into(),
                Err(e) => format!("Settings saved; sign-in option failed: {e}"),
            }
        } else {
            "Settings saved.".into()
        };
        self.update();
    }
    pub(super) unsafe fn tray(&self, action: u32) -> bool {
        let mut n: NOTIFYICONDATAW = zeroed();
        n.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
        n.hWnd = self.hwnd as HWND;
        n.uID = 1;
        n.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        n.uCallbackMessage = TRAY;
        n.hIcon = self.icon as HICON;
        let title = if self.shared.enabled.load(Ordering::Acquire) {
            "Scroll Mage: smoothing on"
        } else {
            "Scroll Mage: paused"
        };
        let title = wide(title);
        n.szTip[..title.len()].copy_from_slice(&title);
        Shell_NotifyIconW(action, &n) != 0
    }
}
