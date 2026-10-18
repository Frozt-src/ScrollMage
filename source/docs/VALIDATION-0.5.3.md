# Version 0.5.3 compact frame validation

2026-10-17. Default outer/client window 1140 by 768 logical pixels, tighter card spacing, and native Windows small-radius corners.

PASS: formatting, clippy across all targets with warnings denied, release build, and 33 focused tests. Layout tests verify the default four-card layout with 0 through 4 exclusions fits without vertical scrolling, the test card remains at least 380 pixels tall, its right edge aligns with the body, footer actions remain within the frame, compact windows still stack, and up to 128 exclusion cards remain reachable without overlap.

PASS: an invisible native test window with the same standard resizable style and client-area caption technique measured 1140 by 768 for both outer and client dimensions. The shared frame-style helper returned HRESULT 0 for DWMWCP_ROUNDSMALL, and DwmGetWindowAttribute returned HRESULT 0 and preference 3. The test window stayed invisible and desktop focus was unchanged. The probe was destroyed after readback; no global input hooks were installed by it.

This confirms that the current Windows installation accepts the small-corner preference. It does not measure rendered corner pixels or guarantee an exact 4-pixel radius. Windows owns the actual radius; maximized/snapped windows can remain square. Systems that reject this Windows 11 preference retain a usable normal frame. No custom window region is installed.

No full app desktop automation or screenshots were taken in this pass. The final compact UI, physical resizing, multiple monitor/DPI combinations, and rendered corner pixels have not been visually checked on the desktop. The prior live UI checks remain in VALIDATION-0.5.md. New measurements come from examples/verify_frame.rs and the actual layout functions.

The plum accent, white/off-white Scroll Test, supplied mouse and purple MAGE artwork, elastic wheel pill, proprietary license, third-party notices, settings, and scrolling engine are preserved. The complete pre-change source, executable, and settings are archived outside the active project.

Reference: https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwm_window_corner_preference
