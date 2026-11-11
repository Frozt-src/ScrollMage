//! Observed wheel delivery, kept separate from global enablement and hook installation.
use std::sync::{
    atomic::{AtomicU64, Ordering},
    RwLock,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Routing {
    Focus,
    Hybrid,
    MousePosition,
    #[default]
    Unknown,
}
impl Routing {
    pub fn from_raw(value: Option<u32>) -> Self {
        match value {
            Some(0) => Self::Focus,
            Some(1) => Self::Hybrid,
            Some(2) => Self::MousePosition,
            _ => Self::Unknown,
        }
    }
    pub fn select(self, foreground: usize, hovered_root: usize) -> Option<usize> {
        let hwnd = match self {
            Self::Focus | Self::Hybrid => foreground,
            Self::MousePosition => hovered_root,
            Self::Unknown => 0,
        };
        (hwnd != 0).then_some(hwnd)
    }
    /// Hybrid can route differently for Store apps. Requiring both roots to agree
    /// avoids guessing the app model. Focus mode retains the same cursor guard.
    pub fn safe(self, target: usize, foreground: usize, hovered_root: usize) -> bool {
        self.select(foreground, hovered_root) == Some(target) && target == hovered_root
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TargetReason {
    Ready,
    #[default]
    NoTarget,
    Excluded,
    HigherIntegrity,
    UnknownProcess,
    UnknownIntegrity,
    RoutingUnavailable,
}
impl TargetReason {
    fn text(self) -> &'static str {
        match self {
            Self::Ready => "Waiting for a compatible wheel notch",
            Self::NoTarget => "No eligible window under the pointer",
            Self::Excluded => "Excluded app; ordinary wheel input passes through",
            Self::HigherIntegrity => {
                "Target has higher integrity; ordinary wheel input passes through"
            }
            Self::UnknownProcess => {
                "Target identity unavailable; ordinary wheel input passes through"
            }
            Self::UnknownIntegrity => {
                "Target integrity unavailable; ordinary wheel input passes through"
            }
            Self::RoutingUnavailable => {
                "Windows wheel routing unavailable; ordinary wheel input passes through"
            }
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputReason {
    #[default]
    None,
    Accepted,
    Emitted,
    Tagged,
    Precision,
    Modifiers,
    ExternalInjected,
    Disabled,
    TargetUnavailable,
    TargetChanged,
    HorizontalDisabled,
    Busy,
}
impl InputReason {
    fn from_raw(value: u8) -> Self {
        match value {
            1 => Self::Accepted,
            2 => Self::Emitted,
            3 => Self::Tagged,
            4 => Self::Precision,
            5 => Self::Modifiers,
            6 => Self::ExternalInjected,
            7 => Self::Disabled,
            8 => Self::TargetUnavailable,
            9 => Self::TargetChanged,
            10 => Self::HorizontalDisabled,
            11 => Self::Busy,
            _ => Self::None,
        }
    }
    fn text(self) -> &'static str {
        match self {
            Self::None | Self::Tagged => "No wheel input observed",
            Self::Accepted => "Wheel notch accepted; output pending",
            Self::Emitted => "Wheel output sent to Windows",
            Self::Precision => "Touchpad or fine wheel input passes through to Windows",
            Self::Modifiers => "Modifier/button held; ordinary wheel input passes through",
            Self::ExternalInjected => "Externally injected input passes through unchanged",
            Self::Disabled => "Smoothing paused; ordinary wheel input passes through",
            Self::TargetUnavailable => "No eligible target; ordinary wheel input passes through",
            Self::TargetChanged => "Target changed; ordinary wheel input passes through",
            Self::HorizontalDisabled => {
                "Horizontal scrolling disabled; ordinary wheel input passes through"
            }
            Self::Busy => "Engine busy; ordinary wheel input passes through",
        }
    }
}

#[derive(Default)]
struct TargetInfo {
    app: String,
    reason: TargetReason,
}
#[derive(Default)]
pub struct Diagnostics {
    target: RwLock<TargetInfo>,
    accepted: AtomicU64,
    bypassed: AtomicU64,
    emitted: AtomicU64,
    // Reason and time share one atomic so a reader never combines separate events.
    last_event: AtomicU64,
    last_ui: AtomicU64,
}
impl Diagnostics {
    pub fn set_target(&self, app: String, reason: TargetReason) {
        if let Ok(mut t) = self.target.write() {
            if t.app != app || t.reason != reason {
                self.invalidate();
                *t = TargetInfo { app, reason };
            }
        }
    }
    pub fn invalidate(&self) {
        self.last_event.store(0, Ordering::Release);
    }
    /// Hook-safe: atomics only. Our tagged emission never changes input counters.
    pub fn record_input(&self, reason: InputReason, now_ms: u64) {
        if matches!(reason, InputReason::Tagged | InputReason::None) {
            return;
        }
        if reason == InputReason::Accepted {
            self.accepted.fetch_add(1, Ordering::Relaxed);
        } else {
            self.bypassed.fetch_add(1, Ordering::Relaxed);
        }
        self.last_event
            .store((now_ms << 8) | reason as u64, Ordering::Release);
    }
    pub fn record_output(&self, now_ms: u64) {
        self.emitted.fetch_add(1, Ordering::Relaxed);
        self.last_event.store(
            (now_ms << 8) | InputReason::Emitted as u64,
            Ordering::Release,
        );
    }
    pub fn counts(&self) -> (u64, u64, u64) {
        (
            self.accepted.load(Ordering::Relaxed),
            self.bypassed.load(Ordering::Relaxed),
            self.emitted.load(Ordering::Relaxed),
        )
    }
    pub fn should_post_ui(&self, now_ms: u64) -> bool {
        let previous = self.last_ui.load(Ordering::Relaxed);
        if previous != 0 && now_ms.saturating_sub(previous - 1) < 200 {
            return false;
        }
        self.last_ui
            .compare_exchange(
                previous,
                now_ms.saturating_add(1),
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .is_ok()
    }
    pub fn status_text(
        &self,
        enabled: bool,
        hook_installed: bool,
        fault: &str,
        now_ms: u64,
    ) -> (String, String) {
        let (accepted, bypassed, emitted) = self.counts();
        let counts = format!("Accepted {accepted} · Output {emitted} · Bypassed {bypassed}");
        if !fault.is_empty() {
            return (
                "Paused — needs attention".into(),
                format!("{fault}\n{counts}"),
            );
        }
        if !enabled {
            return (
                "Paused".into(),
                format!("Ordinary wheel input passes through.\n{counts}"),
            );
        }
        if !hook_installed {
            return (
                "Unavailable".into(),
                format!("Mouse hook is not installed.\n{counts}"),
            );
        }
        let Ok(t) = self.target.read() else {
            return (
                "Unavailable".into(),
                format!("Target information unavailable.\n{counts}"),
            );
        };
        let app = if t.app.is_empty() {
            "No app selected"
        } else {
            &t.app
        };
        if t.reason != TargetReason::Ready {
            return (
                "Unavailable".into(),
                format!("{app}: {}.\n{counts}", t.reason.text()),
            );
        }
        let event = self.last_event.load(Ordering::Acquire);
        let reason = InputReason::from_raw(event as u8);
        let recent = event != 0 && now_ms.saturating_sub(event >> 8) <= 1500;
        let title = if recent {
            match reason {
                InputReason::Emitted => "Smoothing",
                InputReason::Accepted => "Input accepted",
                InputReason::None | InputReason::Tagged => "Ready",
                _ => "Passing through",
            }
        } else {
            "Ready"
        };
        let observation = if reason == InputReason::None {
            t.reason.text()
        } else {
            reason.text()
        };
        (title.into(), format!("{app}: {observation}.\n{counts}"))
    }
}
