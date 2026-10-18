# Upgrade evidence and implementation map

Verified 2026-10-17. The user-supplied brief is design input; its numerical suggestions are not treated as measured Apple defaults.

| Brief item | Implementation and evidence | Limit |
|---|---|---|
| Accumulated spring target | engine.rs + motion.rs, exact critical damping, default omega 30 | Two constant-size states replace the impulse queue; no original-runtime speed comparison |
| Fractional precision | f64 state and carried residual, integer output boundary | Receiving apps still choose how wheel units move content |
| Reversal | Drop old axis debt and velocity before retargeting | Other axis continues independently |
| Momentum | Exact exponential integral, default k=4, cutoff 5 logical pixels/s | Owned preview drag only; OS-provided momentum never starts local inertia |
| Elastic edges | Nonlinear rubber-band, 12% viewport cap, critical return omega=22 | Owned preview only; no arbitrary-app bounds access |
| Gesture interruption | Preview retouch/cancel; global clicks/fine input interrupt pending wheel motion | No claim of system-wide finger-phase detection |
| Profiles | Per-executable distance/response/off, validation and case-insensitive matching | Browsers bypass by default; not an exhaustive compatibility list |
| Latching | Preview drag uses mouse capture; global wheel cancels on region changes | Cancel on crossing child windows rather than redirecting motion to an old control |
| Lightweight interception | Dedicated hook thread, try-lock fail-open, worker output, tagged input | No Raw Input device classification in this version |
| Windows preferences | Preview reads scroll-line and scroll-character counts | System-wide output remains wheel units rather than assumed pixels |
| Frame scheduling | Active-only 8.33 ms waitable timer; preview active-only WM_TIMER | Timer pacing, not presentation synchronization |
| Native touchpad/composition | Deferred | Window-local precision APIs are documented as prerelease; not a global interception interface |
| Mac calibration | Trace requirements documented | NOT RUN: no measured Mac traces provided |

The integration test uncovered a stale latch check: a worker could evaluate cursor safety before acquiring the engine lock and discard newly accepted input. Safety is now evaluated under that lock. Hook contention intentionally passes real input through; constructed test calls retry briefly because they have no physical fallback event.

## Primary sources checked

- [Microsoft WM_MOUSEWHEEL](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-mousewheel): 120-unit normalization and support for smaller wheel deltas.
- [Microsoft SendInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput): input stream delivery, inserted-event count, and integrity restrictions.
- [Apple handling trackpad events](https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/EventOverview/HandlingTouchEvents/HandlingTouchEvents.html): gesture and momentum phases and event routing.
- [Apple precision flag](https://developer.apple.com/documentation/appkit/nsevent/hasprecisescrollingdeltas): precision interface. Page requires JavaScript for full prose in this environment.
- [WebKit kinetic source](https://github.com/WebKit/WebKit/blob/main/Source/WebCore/platform/ScrollAnimationKinetic.cpp): GTK-derived scroller has deceleration friction 4. This does not establish macOS coefficients. Our formulas are independently implemented from the supplied equations; no WebKit source is copied.
- [Microsoft precision touchpad registration](https://learn.microsoft.com/en-us/windows/win32/input-precisiontouchpad/registertouchpadcapable): window/thread registration, prerelease notice.
- [Microsoft DwmFlush](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmflush): only the caller's queued DirectX changes are covered. Calling it would not prove synchronization with another application's content, so it is not used as a substitute for presentation integration.

The brief's release-specific Apple API claim was not corroborated by an accessible authoritative source in this run. It is not used as an implementation dependency or compatibility claim.
