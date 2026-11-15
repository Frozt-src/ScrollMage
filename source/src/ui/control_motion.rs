use super::*;

impl Ui {
    pub(super) unsafe fn snap_toggles(&mut self) {
        for (i, id) in OPTION_TOGGLES.into_iter().enumerate() {
            self.toggle_motion[i].target(checked(self.handle(id)), false);
            InvalidateRect(self.handle(id) as HWND, null(), 0);
        }
        self.toggle_initialized = true;
        KillTimer(self.hwnd as HWND, 6);
    }
    pub(super) unsafe fn sync_toggles(&mut self) {
        if !self.toggle_initialized
            || !animations_allowed()
            || IsWindowVisible(self.hwnd as HWND) == 0
            || IsIconic(self.hwnd as HWND) != 0
        {
            self.snap_toggles();
            return;
        }
        for (i, id) in OPTION_TOGGLES.into_iter().enumerate() {
            self.toggle_motion[i].target(checked(self.handle(id)), true);
        }
        self.toggle_time = self.shared.now();
        SetTimer(self.hwnd as HWND, 6, 16, None);
    }
    pub(super) unsafe fn step_toggles(&mut self, dt: f64) {
        if IsWindowVisible(self.hwnd as HWND) == 0 || IsIconic(self.hwnd as HWND) != 0 {
            self.snap_toggles();
            return;
        }
        let mut active = false;
        for (i, id) in OPTION_TOGGLES.into_iter().enumerate() {
            active |= self.toggle_motion[i].step(dt);
            InvalidateRect(self.handle(id) as HWND, null(), 0);
        }
        if !active {
            KillTimer(self.hwnd as HWND, 6);
        }
    }
}
