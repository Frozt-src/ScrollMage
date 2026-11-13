use super::*;

impl Ui {
    // Commit only exclusions, leaving unrelated settings drafts untouched.
    unsafe fn commit_exclusions(&mut self, apps: Vec<String>) -> Result<(), String> {
        let mut current = self
            .shared
            .settings
            .write()
            .map_err(|_| "Settings are unavailable.")?;
        let mut candidate = current.clone();
        candidate.exclusions = apps.clone();
        candidate.migrate_profiles();
        settings::save(&self.path, &candidate)?;
        *current = candidate;
        drop(current);
        self.excluded_apps = apps;
        self.shared.refresh_target();
        Ok(())
    }

    pub(super) unsafe fn rebuild_exclusions(&mut self) {
        self.exclusion_generation = self.exclusion_generation.wrapping_add(1);
        // Remove the rows from the list before destroying them: DestroyWindow
        // re-enters this window's procedures, which read `controls`.
        let is_row = |id: u16| (EXCLUSION_ROW..EXCLUSION_ROW + 256).contains(&id);
        let rows: Vec<usize> = self
            .controls
            .iter()
            .filter(|&&(id, _, _)| is_row(id))
            .map(|&(_, h, _)| h)
            .collect();
        self.controls.retain(|&(id, _, _)| !is_row(id));
        for h in rows {
            DestroyWindow(h as HWND);
        }
        for (i, name) in self.excluded_apps.clone().into_iter().enumerate() {
            let label =
                self.static_text(&name, EXCLUSION_ROW + i as u16 * 2, true, self.fonts.small);
            let remove = self.button(
                &format!("Remove {name}"),
                EXCLUSION_ROW + i as u16 * 2 + 1,
                true,
                false,
            );
            SetParent(label as HWND, self.handle(EXCLUSION_PANEL) as HWND);
            SetParent(remove as HWND, self.handle(EXCLUSION_PANEL) as HWND);
        }
    }

    pub(super) unsafe fn layout_exclusions(&mut self) {
        let shell = self.shell();
        let viewport = shell.exclusion_viewport();
        let content = shell.exclusion_content_height();
        self.exclusion_scroll = self
            .exclusion_scroll
            .clamp(0, (content - viewport.h).max(0));
        let panel = self.handle(EXCLUSION_PANEL) as HWND;
        let info = SCROLLINFO {
            cbSize: size_of::<SCROLLINFO>() as u32,
            fMask: SIF_RANGE | SIF_PAGE | SIF_POS,
            nMin: 0,
            nMax: content.max(1) - 1,
            nPage: viewport.h.max(1) as u32,
            nPos: self.exclusion_scroll,
            ..zeroed()
        };
        SetScrollInfo(panel, SB_VERT, &info, 1);
        if self.excluded_apps.is_empty() {
            self.at(1116, Rect::new(0, 0, viewport.w - 18, 22));
        } else {
            ShowWindow(self.handle(1116) as HWND, SW_HIDE);
        }
        for i in 0..self.excluded_apps.len() {
            let row = shell.exclusion(i);
            let y = row.y - self.exclusion_scroll;
            self.at(
                EXCLUSION_ROW + i as u16 * 2,
                Rect::new(row.x + 8, y + 2, row.w - 36, 22),
            );
            self.at(
                EXCLUSION_ROW + i as u16 * 2 + 1,
                Rect::new(row.x + row.w - 26, y + 2, 24, 22),
            );
        }
        InvalidateRect(panel, null(), 0);
    }

    pub(super) unsafe fn reveal_exclusion(&mut self, index: usize) {
        let row = self.shell().exclusion(index);
        let height = self.shell().exclusion_viewport().h;
        if row.y < self.exclusion_scroll {
            self.exclusion_scroll = row.y;
        } else if row.y + row.h > self.exclusion_scroll + height {
            self.exclusion_scroll = row.y + row.h - height;
        }
        self.layout_exclusions();
    }

    pub(super) unsafe fn add_exclusion(&mut self) {
        let input = text(self.exclusion_input);
        let raw = input.trim();
        let raw = raw
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .unwrap_or(raw);
        let name = raw.rsplit(['\\', '/']).next().unwrap_or("").to_owned();
        if name.contains(['"', '<', '>', ':', '?', '*']) {
            self.note = "Enter an .exe name or paste its full path without arguments.".into();
            self.update();
            return;
        }
        let mut candidate = self.shared.settings.read().unwrap().clone();
        candidate.exclusions = self.excluded_apps.clone();
        candidate.exclusions.push(name.clone());
        if let Err(error) = candidate.validate() {
            self.note = error;
        } else if self
            .excluded_apps
            .iter()
            .any(|app| app.eq_ignore_ascii_case(&name))
        {
            self.note = "This app is already excluded.".into();
        } else {
            if let Err(error) = self.commit_exclusions(candidate.exclusions) {
                self.note = format!("Could not save exclusion: {error}");
                self.update();
                return;
            }
            self.exclusion_scroll = i32::MAX;
            set(self.exclusion_input, "");
            self.rebuild_exclusions();
            self.note = "App excluded. Saved and active.".into();
            self.layout();
        }
        self.update();
        SetFocus(self.exclusion_input as HWND);
    }

    pub(super) unsafe fn remove_exclusion(&mut self, index: usize) {
        if index < self.excluded_apps.len() {
            let mut apps = self.excluded_apps.clone();
            apps.remove(index);
            if let Err(error) = self.commit_exclusions(apps) {
                self.note = format!("Could not remove exclusion: {error}");
                self.update();
                return;
            }
            self.rebuild_exclusions();
            self.note = "Exclusion removed. Saved and active.".into();
            self.layout();
            self.update();
            SetFocus(self.exclusion_input as HWND);
        }
    }
}

pub(super) unsafe extern "system" fn panel(h: HWND, msg: u32, w: usize, l: isize) -> isize {
    let ptr = state(h);
    if ptr.is_null() {
        return DefWindowProcW(h, msg, w, l);
    }
    let ui = &mut *ptr;
    if let Some(result) = colors(ui, msg, w, l) {
        return result;
    }
    match msg {
        WM_PAINT | WM_PRINTCLIENT => {
            let paint = |dc, width, height| {
                fill(
                    dc,
                    Rect::new(
                        0,
                        0,
                        (f64::from(width) / ui.scale).ceil() as i32,
                        (f64::from(height) / ui.scale).ceil() as i32,
                    ),
                    ui.scale,
                    WHITE,
                );
                for i in 0..ui.excluded_apps.len() {
                    let mut row = ui.shell().exclusion(i);
                    row.y -= ui.exclusion_scroll;
                    if row.y + row.h > 0 && row.y < ui.shell().exclusion_viewport().h {
                        round(dc, row, 4, ui.scale, CANVAS, Some(BORDER));
                    }
                }
            };
            if msg == WM_PAINT {
                buffered(h, paint);
            } else {
                let mut rect = zeroed();
                GetClientRect(h, &mut rect);
                paint(w as HDC, rect.right, rect.bottom);
            }
            0
        }
        WM_ERASEBKGND => 1,
        WM_COMMAND => {
            if (w >> 16) as u32 == BN_CLICKED {
                PostMessageW(
                    ui.hwnd as HWND,
                    REMOVE_EXCLUSION,
                    w & 0xffff,
                    ui.exclusion_generation as isize,
                );
            }
            0
        }
        WM_VSCROLL => {
            let mut info = SCROLLINFO {
                cbSize: size_of::<SCROLLINFO>() as u32,
                fMask: SIF_TRACKPOS,
                ..zeroed()
            };
            GetScrollInfo(h, SB_VERT, &mut info);
            let old = ui.exclusion_scroll;
            ui.exclusion_scroll = match (w & 0xffff) as i32 {
                SB_LINEUP => old - 32,
                SB_LINEDOWN => old + 32,
                SB_PAGEUP => old - ui.shell().exclusion_viewport().h,
                SB_PAGEDOWN => old + ui.shell().exclusion_viewport().h,
                SB_TOP => 0,
                SB_BOTTOM => i32::MAX,
                SB_THUMBTRACK => info.nTrackPos,
                _ => old,
            };
            ui.layout_exclusions();
            0
        }
        WM_MOUSEWHEEL => {
            let delta = ((w >> 16) as u16 as i16) as i32;
            ui.exclusion_scroll -= delta * 32 / 120;
            ui.layout_exclusions();
            0
        }
        _ => DefWindowProcW(h, msg, w, l),
    }
}
