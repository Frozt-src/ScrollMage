use super::*;
/// Timer that drives the Scroll settings card's elastic reveal.
pub(super) const REVEAL_TIMER: usize = 8;

impl Ui {
    /// Show every tuning row for Custom; fixed presets keep the card collapsed.
    pub(super) unsafe fn sync_reveal(&mut self, animate: bool) {
        let open = self.preset == Preset::Custom;
        if open == self.reveal.open() && animate {
            return;
        }
        let mut animation: i32 = 0;
        let allowed = SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            (&mut animation as *mut i32).cast(),
            0,
        ) != 0
            && animation != 0;
        let animate = animate
            && allowed
            && IsWindowVisible(self.hwnd as HWND) != 0
            && IsIconic(self.hwnd as HWND) == 0;
        if !open {
            // Hidden rows cannot keep focus; hand it to the preset picker.
            let focus = GetFocus() as usize;
            if self.inputs.contains(&focus) || (400..412).any(|id| self.handle(id) == focus) {
                SetFocus(self.handle(PRESET) as HWND);
            }
        }
        self.reveal.target(open, animate);
        if animate {
            self.reveal_time = self.shared.now();
            // Windows timers fire on a ~15.6 ms tick; asking for 16 ms waits two
            // ticks (~30 ms). Below one tick fires every tick, about 64 frames/s.
            SetTimer(self.hwnd as HWND, REVEAL_TIMER, 10, None);
        } else {
            KillTimer(self.hwnd as HWND, REVEAL_TIMER);
        }
        self.layout();
    }
    pub(super) unsafe fn step_reveal(&mut self) {
        let now = self.shared.now();
        let dt = (now - self.reveal_time).max(0.) / 1000.;
        self.reveal_time = now;
        let moving = IsWindowVisible(self.hwnd as HWND) != 0
            && IsIconic(self.hwnd as HWND) == 0
            && self.reveal.step(dt);
        if !moving {
            let open = self.reveal.open();
            self.reveal.target(open, false);
            KillTimer(self.hwnd as HWND, REVEAL_TIMER);
        }
        self.layout();
    }
    pub(super) fn settings_height(&self) -> i32 {
        self.reveal.height(SETTINGS_COLLAPSED, SETTINGS_EXPANDED)
    }
}
