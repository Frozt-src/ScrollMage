use super::*;
const IDS: [u16; 4] = [212, 213, 214, STATUS_SWITCH];

impl Ui {
    pub(super) unsafe fn snap_toggles(&mut self) {
        for (i, id) in IDS.into_iter().enumerate() {
            self.toggle_motion[i].target(checked(self.handle(id)), false);
            InvalidateRect(self.handle(id) as HWND, null(), 0);
        }
        self.toggle_initialized = true;
        KillTimer(self.hwnd as HWND, 6);
    }
    pub(super) unsafe fn sync_toggles(&mut self) {
        let mut animation: i32 = 0;
        let allowed = SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            (&mut animation as *mut i32).cast(),
            0,
        ) != 0
            && animation != 0;
        if !self.toggle_initialized
            || !allowed
            || IsWindowVisible(self.hwnd as HWND) == 0
            || IsIconic(self.hwnd as HWND) != 0
        {
            self.snap_toggles();
            return;
        }
        for (i, id) in IDS.into_iter().enumerate() {
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
        for (i, id) in IDS.into_iter().enumerate() {
            active |= self.toggle_motion[i].step(dt);
            InvalidateRect(self.handle(id) as HWND, null(), 0);
        }
        if !active {
            KillTimer(self.hwnd as HWND, 6);
        }
    }
}
