//! Wheel routing, and why a target is ineligible or a notch passes through.

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetReason {
    Ready,
    NoTarget,
    Excluded,
    HigherIntegrity,
    UnknownProcess,
    UnknownIntegrity,
    RoutingUnavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputReason {
    Accepted,
    Precision,
    Modifiers,
    ExternalInjected,
    Disabled,
    TargetUnavailable,
    TargetChanged,
    HorizontalDisabled,
    Busy,
}
