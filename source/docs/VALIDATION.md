# Scroll Mage 0.6.3 logo validation

October 17, 2026. The second logo direction is integrated as cached native outline paths, with a newly sized elastic wheel cutout in its final l.

The existing 36 focused checks plus two new logo tests and the hidden native-control integration test pass, for 39 checks. Formatting and clippy with warnings denied are checked before the release build. Headless UI rendering covers 100%, 125%, 150%, 175%, and 200% scale. Native logo exports include 80 source frames over four seconds; only pixels inside the final l change. The supplied static slot is removed from the vector layer and replaced by the animated counter. Reduced-motion, nonfinite time, seam continuity, and full-cycle geometry bounds pass.

Editable SVG masters contain no live text, installed font references, or embedded raster images. The static SVG audit reports 84/100, with warnings for outline complexity and four reference-derived near-cardinal edges. PNG and GIF previews are encoded from the native app renderer. The supplied reference and prior artwork are preserved. Application settings, proprietary license, and third-party notices are unchanged.

No desktop app control or live animation acceptance was performed in this pass. The active process may retain the prior executable until it exits. The logo's native code and assets are verified headlessly; this does not claim live desktop timer delivery. See BRANDING.md for construction, artifacts, and provenance.

## Previous settings verification

## Automated evidence

- 36 focused tests cover engine conservation, reversal, fractional residuals, momentum and elasticity, layout, branding, settings persistence, and migration. New tests prove that preset identity survives manual tuning and that smoother presets use smaller steps without adding wheel travel or changing app choices.
- One hidden native-control integration test runs at 100%, 125%, 150%, 175%, and 200% scale. It creates native controls without a visible main window, input hook, or tray icon. It checks all six button/keyboard step pairs, lower-bound clamping, preset commit notifications, deliberate Apply persistence, sidebar pause placement, bounded 128-entry exclusions and access to the final entry, minimized-layout preservation, and light final-pixel coverage at the Scroll Test edge. It verifies that its own window never becomes the foreground window.
- Reinstating the old truncated preview dimensions makes that test fail at 125% scaling with a dark edge pixel of 37383b. Reinstating the old minimized-layout behavior makes it fail with layout 199 by 34 rather than 1140 by 768. The corrected implementation passes both checks.
- Native font readback resolves the body font to Segoe UI. The existing logo artwork is embedded unchanged. SF Pro is not distributed or selected.
- Formatting, clippy with warnings denied, and the release build are checked before installation. Recorded build output and installation hashes accompany the staged delivery.

## Limits

Headless renders exercise actual control painting into memory bitmaps. They do not establish physical-wheel feel, display ghosting or tearing, multi-monitor transitions, a measured match to macOS, real shell tray interaction, or successful global hotkey capture on a desktop where another app may own Alt+P. The registration failure path keeps the in-app and tray pause controls available. Desktop automation is reserved for closing the prior running app if required to replace its executable.

Outer corners remain the Windows-controlled small-radius preference, as documented in VALIDATION-0.5.3.md. Inner cards are rendered with a four logical pixel radius. The requested SF Pro change was not made under Apple's published standard font license. Sources and preset rationale are in README.md.

## Version 0.6.3 caption checks

The hidden native-control integration test now checks the caption logo at five DPI scales: its bounds fit within the caption and stay clear of window controls, its native paint contains purple MAGE lettering, its hit test passes through, and the frame beneath its center remains draggable. All 39 focused and hidden checks pass before installation. Headless rendering confirms placement; no live desktop timer or physical dragging check is claimed.

## Version 0.6.3 single-logo layout

Only the animated title-bar logo remains. The sidebar logo control and its repaint path are removed. Supporting sidebar text starts at y=76 and navigation at y=136, closing the former logo space. The title-bar logo keeps its existing scale, animation policy, and pass-through hit testing. Verification remains headless.


## 0.6.4 control polish, October 17, 2026

Headless native HWND tests reproduce the old Add behavior, then verify Add/Enter and Remove update both the active settings policy and the atomically saved file. Quoted full paths, names with spaces, case-insensitive duplicates, invalid extensions, failed Add/Remove writes, unrelated unsaved numeric drafts, and repeated queued removal clicks are covered. Removal is deferred until native button dispatch finishes; a list generation prevents a stale click from removing the next app. Native checkbox BM_CLICK state is checked without activating a window.

The native ComboLBox reports alpha 217 with LWA_ALPHA and a rounded window region at five DPI scales. Its window remains hidden during testing. This verifies configured transparency and clipping, not live desktop composition, actual background blur, or visual appearance over another app. The design uses translucent pale tint rather than claiming acrylic blur.

Pure toggle tests cover retargeting without jumps, settlement in both directions, refresh-rate independence, and snapping for disabled animation. Hidden control renders cover raised and pressed arrows, toggle midpoint, menu content, and the complete page. Full layouts are rendered at 100, 125, 150, 175, and 200 percent DPI without desktop capture. Hidden or minimized windows snap switches and stop their animation timer; Windows animation preferences are checked at runtime.

Validation commands: cargo fmt --all -- --check; cargo clippy --locked --offline --all-targets -- -D warnings; the eight focused integration test targets in README; and the filtered hidden_native_controls binary test. The separate desktop-input fixture is NOT RUN. No input hooks, foreground activation, global input, tray changes, or running-app restarts are performed by the validation workflow.

Official reference: [SetLayeredWindowAttributes](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setlayeredwindowattributes).


## 0.6.5 field clipping fix, October 18, 2026

A new hidden-native regression first failed against 0.6.4: the full-width Scroll settings helper overlapped the closed profile combo, while the Excluded apps heading overlapped the entry frame and Add button. Opaque sibling label surfaces therefore covered parts of the rounded controls. The regression checks native window rectangles and the parent-painted frame at four window sizes (940x680, 1100x760, 1140x768, 1280x920) and five display scales (100, 125, 150, 175, 200 percent).

The helper now ends before the profile pill. The right entry row has six logical pixels of clearance below the heading, a 32-pixel control height, and eight pixels before the list. The exclusion card grows by eight logical pixels; the existing 25 percent form-width reduction, three-row scroll viewport, and full-card Scroll Test are preserved. The native edit remains inset within its painted border. No scroll-distance or engine tuning changes are included.

The overlap regression passes after the fix. Hidden native composites and the previous exclusion, motion, and layout suites are rerun before packaging. Actual desktop capture and global input remain NOT RUN.


## 0.6.6 capsule bounds and dropdown blur, October 18, 2026

The user's follow-up screenshot disproved the earlier claim of complete clipping resolution. The 0.6.5 overlap fix remains valid, but its tests did not cover stroke overflow or native collapsed-item owner drawing. A new guarded bitmap test failed at all five scales: the capsule painted past its real HWND client height, and an ODS_COMBOBOXEDIT draw used popup-row styling over the closed control. A per-scale child z-order/bounds audit is emitted to target/headless-ui/pill-layer-audit.txt.

The capsule now uses physical client pixels with an inset stroke. WM_PAINT, WM_PRINTCLIENT and the collapsed owner-draw path share its painter. The guard test checks all four edges at 100, 125, 150, 175 and 200 percent DPI, and compares the ordinary and native selected-item repaints.

The dropdown renders only Scroll Mage's own surfaces into memory, including native field contents and ancestor clipping. It caches a three-pass separable blur for the current popup position, mixes 38/255 blurred content with 217/255 tint, and keeps labels opaque. Native list keyboard, selection and dismissal handling remain in place. No desktop pixels are captured. The background refreshes when the popup opens or moves; it is not a live view of other applications. No new dependencies or third-party code were added.

Pixel tests distinguish actual blur from a flat translucent tint, verify tint proportions, and cover a one-pixel image. Hidden native tests verify that changing a field behind the popup changes the generated backdrop, that repeated paints reuse the cache, and that the popup retains its rounded region. Native menu content and complete app composites are rendered without activating a window. The previous exclusion and engine suites remain in the focused checks. Desktop interaction and live compositor appearance remain NOT RUN.

The app-owned rendering approach avoids treating a successful legacy blur API call as proof of visible blur: Microsoft documents that [DwmEnableBlurBehindWindow no longer creates blur starting with Windows 8](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmenableblurbehindwindow).
