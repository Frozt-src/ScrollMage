use super::*;

impl Ui {
    pub(super) unsafe fn show_tuning(&mut self, settings: &Settings) {
        self.loading_controls = true;
        self.preset = settings.preset;
        for (input, value) in self.inputs.into_iter().zip([
            settings.distance,
            settings.omega,
            settings.acceleration_ms,
            settings.acceleration_max,
            settings.friction,
            settings.elastic_omega,
        ]) {
            set(input, &value.to_string());
        }
        SendMessageW(
            self.handle(PRESET) as HWND,
            CB_SETCURSEL,
            self.preset as usize,
            0,
        );
        self.loading_controls = false;
        self.sync_preset_controls();
    }

    pub(super) unsafe fn sync_preset_controls(&self) {
        let editable = !self.preset.fixed();
        for (i, input) in self.inputs.iter().enumerate() {
            EnableWindow(*input as HWND, i32::from(editable));
            EnableWindow(self.handle(400 + i as u16 * 2) as HWND, i32::from(editable));
            EnableWindow(self.handle(401 + i as u16 * 2) as HWND, i32::from(editable));
        }
        self.update_tuning_hint();
    }

    unsafe fn update_tuning_hint(&self) {
        let original = Preset::Default.tuning();
        let custom = self
            .inputs
            .iter()
            .zip([
                original.distance,
                original.omega,
                original.acceleration_ms,
                original.acceleration_max,
                original.friction,
                original.elastic_omega,
            ])
            .any(|(input, value)| text(*input).trim().parse::<f64>() != Ok(value));
        set(
            self.handle(1110),
            match self.preset {
                Preset::SuperSmooth => "Softest glide. Flick to coast.",
                Preset::MacosStyle => "Soft glide, faster as you spin.",
                Preset::Default if custom => "Custom Default tuning.",
                Preset::Default => "Crisp and precise. Editable.",
            },
        );
    }

    pub(super) unsafe fn pending_changes(&self) -> bool {
        if !self.ready || self.loading_controls {
            return false;
        }
        let Ok(saved) = self.shared.settings.read() else {
            return false;
        };
        let values = [
            saved.distance,
            saved.omega,
            saved.acceleration_ms,
            saved.acceleration_max,
            saved.friction,
            saved.elastic_omega,
        ];
        (!self.preset.fixed()
            && self
                .inputs
                .iter()
                .zip(values)
                .any(|(input, saved)| text(*input).trim().parse::<f64>() != Ok(saved)))
            || checked(self.horizontal) != saved.horizontal
            || checked(self.reverse) != saved.reverse
            || checked(self.startup) != startup_enabled()
    }

    pub(super) unsafe fn update_pending(&self) {
        if !self.ready || self.loading_controls {
            return;
        }
        self.update_tuning_hint();
        let pending = self.pending_changes();
        set(
            self.handle(APPLY),
            if pending {
                "Apply changes"
            } else {
                "Settings saved"
            },
        );
        EnableWindow(self.handle(APPLY) as HWND, i32::from(pending));
        let copy = if !self.note.is_empty() {
            self.note.clone()
        } else if pending {
            "Unsaved changes. Apply to activate.\nAlt + P pauses or resumes.".into()
        } else {
            "Alt + P pauses or resumes.".into()
        };
        set(self.handle(904), &copy);
    }
}
