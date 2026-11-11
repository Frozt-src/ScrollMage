use super::*;
use scrollpilot::settings::Profile;

const LIST: u16 = 3001;
const EXE: u16 = 3002;
const KIND: u16 = 3003;
const DISTANCE: u16 = 3004;
const RESPONSE: u16 = 3005;
const SAVE: u16 = 3006;
const REMOVE: u16 = 3007;
const KIND_LABELS: [&str; 4] = [
    "Custom",
    "macOS style (fixed)",
    "Super smooth (fixed)",
    "Pass through / excluded",
];

impl Ui {
    pub(super) unsafe fn build_profiles(&mut self) {
        self.static_text("App profiles", 3010, true, self.fonts.section);
        self.static_text("Choose an app to edit, or enter an EXE name to add one. Apps without a profile use global settings.", 3011, true, self.fonts.small);
        self.create(
            "LISTBOX",
            "Saved app profiles",
            WS_TABSTOP | WS_VSCROLL | WS_BORDER | LBS_NOTIFY as u32 | LBS_NOINTEGRALHEIGHT as u32,
            LIST,
            true,
            self.fonts.body,
        );
        for (title, id) in [
            ("EXE name or full path", 3020),
            ("Scrolling profile", 3021),
            ("Distance", 3022),
            ("Response", 3023),
        ] {
            self.static_text(title, id, true, self.fonts.small);
        }
        for (id, name) in [
            (EXE, "Application executable"),
            (DISTANCE, "App scroll distance"),
            (RESPONSE, "App response"),
        ] {
            self.create(
                "EDIT",
                name,
                WS_TABSTOP | WS_BORDER | ES_AUTOHSCROLL as u32,
                id,
                true,
                self.fonts.body,
            );
        }
        set(self.handle(EXE), "");
        set(self.handle(DISTANCE), "120");
        set(self.handle(RESPONSE), "30");
        let combo = self.create(
            "COMBOBOX",
            "App scrolling profile",
            WS_TABSTOP
                | CBS_DROPDOWNLIST as u32
                | CBS_OWNERDRAWFIXED as u32
                | CBS_HASSTRINGS as u32,
            KIND,
            true,
            self.fonts.body,
        ) as HWND;
        // Native selection/keyboard semantics with the app's shared field painter.
        windows_sys::Win32::UI::Controls::SetWindowTheme(
            combo,
            wide("").as_ptr(),
            wide("").as_ptr(),
        );
        for label in KIND_LABELS {
            SendMessageW(combo, CB_ADDSTRING, 0, wide(label).as_ptr() as isize);
        }
        SendMessageW(combo, CB_SETCURSEL, 0, 0);
        SendMessageW(combo, CB_SETITEMHEIGHT, usize::MAX, self.px(28) as isize);
        SendMessageW(combo, CB_SETITEMHEIGHT, 0, self.px(28) as isize);
        self.button("Save app profile", SAVE, true, false);
        self.button("Remove profile", REMOVE, true, false);
        self.static_text("Save applies this app profile immediately. Global settings drafts stay unchanged. Custom profiles override distance and response; acceleration follows the global profile. Fixed profiles include their own acceleration. Excluded apps still pass through.", 3012, true, self.fonts.small);
        self.rebuild_profiles(None);
    }

    pub(super) unsafe fn layout_profiles(&self) {
        let w = self.shell().body.w;
        let y = -self.scroll;
        self.at(3010, Rect::new(20, y + 16, w - 40, 30));
        self.at(3011, Rect::new(20, y + 54, w - 40, 48));
        self.at(LIST, Rect::new(20, y + 108, w - 40, 148));
        self.at(3020, Rect::new(20, y + 274, w - 40, 22));
        self.at(EXE, Rect::new(20, y + 300, w - 40, 30));
        self.at(3021, Rect::new(20, y + 344, w - 40, 22));
        self.at(KIND, Rect::new(20, y + 370, w - 40, 180));
        let half = (w - 60) / 2;
        self.at(3022, Rect::new(20, y + 414, half, 22));
        self.at(3023, Rect::new(40 + half, y + 414, half, 22));
        self.at(DISTANCE, Rect::new(20, y + 440, half, 30));
        self.at(RESPONSE, Rect::new(40 + half, y + 440, half, 30));
        self.at(SAVE, Rect::new(20, y + 488, half, 38));
        self.at(REMOVE, Rect::new(40 + half, y + 488, half, 38));
        self.at(3012, Rect::new(20, y + 544, w - 40, 160));
    }

    pub(super) unsafe fn draw_app_profile_item(&self, item: &DRAWITEMSTRUCT) {
        let selected = item.itemState & ODS_SELECTED != 0;
        let brush = CreateSolidBrush(color(if selected { SELECTED } else { WHITE }));
        FillRect(item.hDC, &item.rcItem, brush);
        DeleteObject(brush);
        if let Some(text) = KIND_LABELS.get(item.itemID as usize) {
            let r = item.rcItem;
            label(
                item.hDC,
                Rect::new(
                    (f64::from(r.left) / self.scale).round() as i32 + 8,
                    (f64::from(r.top) / self.scale).round() as i32,
                    (f64::from(r.right - r.left) / self.scale).round() as i32 - 16,
                    (f64::from(r.bottom - r.top) / self.scale).round() as i32,
                ),
                self.scale,
                self.fonts.body,
                if selected { ACCENT_TEXT } else { INK },
                text,
                DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
            );
        }
    }

    pub(super) unsafe fn paint_app_profile_combo(&self, h: HWND, dc: HDC, width: i32, height: i32) {
        let rect = RECT {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        };
        let brush = CreateSolidBrush(color(WHITE));
        FillRect(dc, &rect, brush);
        DeleteObject(brush);
        round(
            dc,
            Rect::new(1, 1, width - 2, height - 2),
            self.px(4),
            1.,
            WHITE,
            Some(if GetFocus() == h { ACCENT } else { BORDER }),
        );
        let index = SendMessageW(h, CB_GETCURSEL, 0, 0);
        if let Some(text) = KIND_LABELS.get(index as usize) {
            label(
                dc,
                Rect::new(self.px(8), 0, width - self.px(40), height),
                1.,
                self.fonts.body,
                INK,
                text,
                DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS,
            );
        }
        let x = width - self.px(16);
        let y = height / 2;
        line(dc, x - self.px(3), y - 1, x, y + self.px(2), 1., MUTED, 1);
        line(dc, x, y + self.px(2), x + self.px(3), y - 1, 1., MUTED, 1);
    }

    unsafe fn rebuild_profiles(&self, selected: Option<&str>) {
        let list = self.handle(LIST) as HWND;
        SendMessageW(list, LB_RESETCONTENT, 0, 0);
        if let Ok(settings) = self.shared.settings.read() {
            for (i, profile) in settings.profiles.iter().enumerate() {
                let label = if profile.bypass {
                    "Pass through"
                } else {
                    profile
                        .preset
                        .filter(|p| p.fixed())
                        .map_or("Custom", Preset::label)
                };
                SendMessageW(
                    list,
                    LB_ADDSTRING,
                    0,
                    wide(&format!("{}  —  {label}", profile.executable)).as_ptr() as isize,
                );
                if selected.is_some_and(|s| s.eq_ignore_ascii_case(&profile.executable)) {
                    SendMessageW(list, LB_SETCURSEL, i, 0);
                }
            }
        }
        EnableWindow(
            self.handle(REMOVE) as HWND,
            i32::from(SendMessageW(list, LB_GETCURSEL, 0, 0) >= 0),
        );
    }

    unsafe fn selected_profile(&mut self) {
        let index = SendMessageW(self.handle(LIST) as HWND, LB_GETCURSEL, 0, 0);
        let profile = self
            .shared
            .settings
            .read()
            .ok()
            .and_then(|s| s.profiles.get(index as usize).cloned());
        if let Some(profile) = profile {
            set(self.handle(EXE), &profile.executable);
            set(self.handle(DISTANCE), &profile.distance.to_string());
            set(self.handle(RESPONSE), &profile.omega.to_string());
            let index = if profile.bypass {
                3
            } else {
                match profile.preset {
                    Some(Preset::MacosStyle) => 1,
                    Some(Preset::SuperSmooth) => 2,
                    _ => 0,
                }
            };
            SendMessageW(self.handle(KIND) as HWND, CB_SETCURSEL, index, 0);
            self.profile_kind();
            EnableWindow(self.handle(REMOVE) as HWND, 1);
        }
    }

    unsafe fn profile_kind(&self) {
        let index = SendMessageW(self.handle(KIND) as HWND, CB_GETCURSEL, 0, 0);
        if index == 1 || index == 2 {
            let tuning = Preset::ALL[index as usize].tuning();
            set(self.handle(DISTANCE), &tuning.distance.to_string());
            set(self.handle(RESPONSE), &tuning.omega.to_string());
        }
        EnableWindow(self.handle(DISTANCE) as HWND, i32::from(index == 0));
        EnableWindow(self.handle(RESPONSE) as HWND, i32::from(index == 0));
    }

    unsafe fn save_profile(&mut self) {
        let raw = text(self.handle(EXE));
        let exe = raw
            .trim()
            .trim_matches('"')
            .rsplit(['\\', '/'])
            .next()
            .unwrap_or("")
            .to_owned();
        let index = SendMessageW(self.handle(KIND) as HWND, CB_GETCURSEL, 0, 0);
        let preset = match index {
            1 => Some(Preset::MacosStyle),
            2 => Some(Preset::SuperSmooth),
            _ => None,
        };
        let values = if let Some(preset) = preset {
            let t = preset.tuning();
            Ok((t.distance, t.omega))
        } else if index == 3 {
            Ok((120., 30.))
        } else {
            text(self.handle(DISTANCE))
                .trim()
                .parse::<f64>()
                .and_then(|d| {
                    text(self.handle(RESPONSE))
                        .trim()
                        .parse::<f64>()
                        .map(|o| (d, o))
                })
        };
        let (distance, omega) = match values {
            Ok(v) => v,
            Err(_) => {
                self.note = "Enter valid app distance and response numbers.".into();
                self.update();
                return;
            }
        };
        let mut candidate = self.shared.settings.read().unwrap().clone();
        let profile = Profile {
            executable: exe.clone(),
            preset,
            distance,
            omega,
            bypass: index == 3,
        };
        if let Some(existing) = candidate
            .profiles
            .iter_mut()
            .find(|p| p.executable.eq_ignore_ascii_case(&exe))
        {
            *existing = profile;
        } else {
            candidate.profiles.push(profile);
        }
        if let Err(error) = settings::save(&self.path, &candidate) {
            self.note = format!("App profile not saved: {error}");
        } else {
            *self.shared.settings.write().unwrap() = candidate;
            self.shared.refresh_target();
            self.rebuild_profiles(Some(&exe));
            self.note = format!("Profile for {exe} saved and active.");
        }
        self.update();
    }

    unsafe fn remove_profile(&mut self) {
        let index = SendMessageW(self.handle(LIST) as HWND, LB_GETCURSEL, 0, 0);
        let mut candidate = self.shared.settings.read().unwrap().clone();
        if index < 0 || index as usize >= candidate.profiles.len() {
            return;
        }
        candidate.profiles.remove(index as usize);
        if let Err(error) = settings::save(&self.path, &candidate) {
            self.note = format!("App profile not removed: {error}");
        } else {
            *self.shared.settings.write().unwrap() = candidate;
            self.shared.refresh_target();
            self.rebuild_profiles(None);
            self.note =
                "App profile removed. Global settings apply unless the app is excluded.".into();
        }
        self.update();
    }

    pub(super) unsafe fn profile_command(&mut self, id: u16, notification: u32) -> bool {
        match (id, notification) {
            (LIST, LBN_SELCHANGE) => self.selected_profile(),
            (KIND, CBN_SELCHANGE) => self.profile_kind(),
            (SAVE, BN_CLICKED) => self.save_profile(),
            (REMOVE, BN_CLICKED) => self.remove_profile(),
            _ => return false,
        }
        true
    }
}
