# Windows scrolling implementation comparison

Verified October 18, 2026. This compares inspected primary source and documented platform contracts. Repository claims about feel and app coverage were not independently benchmarked. No external implementation code was copied into Scroll Mage.

## What the Windows references actually do

1. [SmoothScroll for Windows, Python implementation](https://github.com/re1von/Smoothscroll-for-windows/blob/3367bd7264f0d8ce9478ab5a527eba54d94f550b/smoothscroll/smoothscroll.py) eases queued wheel impulses and carries fractional remainder between updates. Its [Windows adapter](https://github.com/re1von/Smoothscroll-for-windows/blob/3367bd7264f0d8ce9478ab5a527eba54d94f550b/smoothscroll/utils/winapi.py) intercepts wheel events, resolves the window under the pointer, and emits smaller wheel events. It does not gain universal pixel-scroll control by using Python or its GUI. Scroll Mage already has equivalent essential stages: wheel interception, a continuous follower, fractional state, integer wheel emission and tagged loop prevention.

2. [SmoothScroll's Rust Windows emitter](https://github.com/quangtruong2003/SmoothScroll/blob/94a6680e852b6dbd3e6ea079fc826fb3befa601b/crates/platform/src/windows/wheel_emitter.rs) uses native input injection for vertical output and supports an alternate horizontal-message transport for applications whose horizontal semantics differ. Its [mouse hook](https://github.com/quangtruong2003/SmoothScroll/blob/94a6680e852b6dbd3e6ea079fc826fb3befa601b/crates/platform/src/windows/mouse_hook.rs) classifies input sources and distinguishes its own injected marker. These are extra compatibility layers, not a missing Tauri/React prerequisite. The project uses the [FSL license](https://github.com/quangtruong2003/SmoothScroll/blob/94a6680e852b6dbd3e6ea079fc826fb3befa601b/LICENSE); this review uses architectural observations only.

3. [Scroil's own supported-app and restrictions documentation](https://github.com/EricxWood/Scroil) describes app classification and per-app controls, and explicitly excludes some older desktop applications and File Explorer from supported pixel-precise smoothing. This is the project's stated coverage boundary, not independently verified compatibility. It is corroborating evidence against promising universal smooth rendering from a global hook alone. Its native wheel interceptor was also inspected; advertised polling rates are not measured visible frame rates.

Browser-only JavaScript projects called SmoothScroll operate on a page's own DOM and scroll offsets. They do not establish an equivalent Windows system-wide implementation and were not used as native compatibility evidence.

## Missing steps found in Scroll Mage

| Layer | Earlier behavior | Change in 0.6.8 review build |
| --- | --- | --- |
| Preset activation | Selection populated editable fields but required Apply | Fixed macOS style/Super smooth values save and activate on selection; Default remains editable and its saved tuning survives switching |
| Per-app tuning | Loading discarded legacy custom overrides | Load/save retains profiles; App profiles editor supports custom distance/response, fixed presets, bypass, and removal |
| Hover target | Foreground-only target silently bypassed inactive hover scrolling | Target selection respects Windows wheel routing; mouse-position routing supports the hovered root with eligibility and emission checks |
| Delivery feedback | Enabled flag displayed as smoothing-on | Activity and status distinguish ready, accepted/output, pass-through, paused, unavailable and errors; session counters track actual events |
| Windows frame | Client maximize button did not expose the Snap hit target | Native maximize hit testing/forwarding implemented; minimum width reduced to 500 logical pixels with compact navigation |

Microsoft documents the [mouse-wheel routing settings](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-systemparametersinfoa), [small-delta receiver behavior](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-mousewheel), [SendInput integrity restriction](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput), and [Snap hit-test/minimum-size requirements](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/ui/apply-snap-layout-menu). Those contracts guided original changes to the existing native implementation.

## Remaining boundaries and next evidence

- Fine-resolution and external injected wheel input still pass through. A reliable next step is device/source classification and a physical-device test, rather than treating every small or injected event as a mouse notch and risking duplicate touchpad momentum or reinjection loops.
- Horizontal scrolling still emits native horizontal wheel events. An app-specific alternate transport could improve programs expecting Shift+vertical-wheel semantics, but needs a named target and focused compatibility test before claiming support.
- A receiver can accumulate small deltas into whole-line movement. The earlier standard EDIT/LISTBOX replay preserved 120 units across 25 engine events but observed only three top-line updates. That is receiver quantization, not a missing timer or framework initialization step.
- Global wheel output cannot read arbitrary app content boundaries or control its presentation. Friction and elasticity remain explicitly scoped to Scroll Test dragging. A target-specific scroll-offset/bounds adapter would be needed to add controlled elastic boundaries elsewhere; none is claimed here.
- An installed-hook flag does not prove that Windows has never silently removed the hook. The UI reports recent accepted/output observations without claiming a hook-health guarantee.
- Hidden tests establish native control behavior and drawing. Actual shell Snap flyout, physical-wheel feel, and app-specific visible acceptance require desktop checks and are reported separately in validation.

There is no evidence that replacing Scroll Mage's native UI framework would fix these input and rendering boundaries.
