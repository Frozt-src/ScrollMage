# Scroll Mage interface design

Version 0.6.3, 2026-10-17. Native Windows scrolling controls with a quiet settings surface and room to test scrolling.

## Tokens

Canvas and integrated caption #F6F7FB; sidebar #EDEFF5; text #10132D; supporting text #636E82; plum red accent #8E3555; borders #DCE1EB. White cards and the white/off-white Scroll Test share 4-pixel corners. Individual exclusion cards use the canvas color and 4-pixel corners. Segoe UI is used for controls, Segoe UI Variable Display for display text, and Segoe UI for numbers. Copy uses ordinary punctuation without em or en dashes.

## Layout

The integrated caption is 48 logical pixels high. Cards begin at y=64. The default outer and client size are both 1140 by 768 logical pixels because the custom caption already belongs to the client area. No extra standard title-bar dimensions are added. The main header logo and status banner are removed; the title bar uses a left-facing mouse with its tail connected to the S and purple MAGE beneath the right side. The elastic wheel pill remains.

The left column contains Scroll settings and Options. Smoothing status, its helper, and its switch sit at the bottom of Options. The right column contains Excluded apps and Scroll Test. Scroll Test begins immediately below exclusions and expands to the available height, occupying the former profiles area. Its whole surface is interactive: title, helper, counters, and rows belong to one card, with no inset viewport. Reset stays pinned in its header.

The two main columns use a 16-pixel gap. Scroll settings is 320 pixels high and Options is 276 pixels high; numeric rows are 38 pixels apart. The outer content gutter is 24 pixels and no scrollbar gutter is reserved when the content fits. The default layout with up to four exclusions has no vertical scrollbar. Exclusions use a two-column grid of individual app cards with named native remove buttons. Plum selection tint is #F3E4EA, selection text #792A48, and pressed actions #722541. Scroll Test uses white #FFFFFF and off-white #F8F7F5 for its header, metrics, and alternating rows. A single-line app-name field supports Add and Enter. The section is capped at 184 logical pixels high, with two compact rows visible and a native scrollbar for larger lists, up to 128 entries. App cards are 26 pixels high, with 12-pixel text and accessible native remove buttons. Narrow windows stack all four settings cards in reading order. The sidebar has no logo; supporting copy starts at y=76 and navigation starts at y=136. Footer actions remain fixed; the settings body scrolls. Default size is 1140 by 768 logical pixels; minimum outer size is 940 by 680. Scroll Test appears only on the main Settings page. Navigation contains Settings and About.

## Behavior

- The sidebar pause button, Alt+P, tray menu, and status switch control the same pause state. Apply settings is the only footer action; Quit remains in the tray menu.
- Apply saves controls and exclusions. Duplicate executable names are rejected without regard to case; invalid names do not add a card.
- Per-app profiles have no editor or active hidden overrides in the running app. Legacy off entries migrate into visible exclusions; custom tuning falls back to global settings. Loading preserves the original file until Apply.
- The custom frame retains the Windows resizable style and removes the standard caption using WM_NCCALCSIZE. WM_NCHITTEST provides resize edges and a draggable caption. Accessible native buttons issue minimize, maximize/restore, and close-to-tray commands. Windows handles caption double-click and the system menu. The Windows 11 maximize-hover Snap Layout flyout is not implemented by the custom client button.
- Scroll Test keeps wheel, drag, arrow-key, Home/End, momentum, and elastic behavior. Bounds account for its pinned header. Hiding, minimizing, or switching pages stops test animation.
- Buttons retain native focus and accessibility names. PerMonitorV2 scaling and reduced-motion branding behavior remain in place.

Frame reference: [Microsoft custom-window-frame guidance](https://learn.microsoft.com/en-us/windows/win32/dwm/customframe). Verification and limits are recorded in VALIDATION.md.

## Window corners

The app calls DwmSetWindowAttribute with DWMWA_WINDOW_CORNER_PREFERENCE and DWMWCP_ROUNDSMALL after frame creation and when Windows settings change. This requests native small-radius corners, close to the requested 4-pixel style, without imposing an exact pixel value. Windows controls the actual radius and may use square corners for maximized or snapped windows. Unsupported systems retain their normal frame. No SetWindowRgn or layered-window clipping is used.

Reference: https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/ui/apply-rounded-corners

## Presets and control paint

A native owner-drawn combo box presents three global presets in a plum-tinted pill across from Scroll settings. Confirming an item loads its values; closed-menu arrow selection also works. Preset identity and manually adjusted numeric values persist independently. Apply settings commits them. The six numeric fields have bounded native button and keyboard stepping, with framed chevrons. The maximum acceleration hint sits below its label. Unit suffix controls are removed.

All physical pixels are initialized before buffered painting; fractional display sizes round outward. Test and button client printing uses the same custom rendering. Minimized dimensions are excluded from layout. Tray and second-launch restoration use one path that restores minimized windows, reflows existing controls, and redraws descendants without rebuilding the form.

SF Pro is not used under Apple's standard font license for a Windows product. Body, caption, helper, section, and numeric text use the Windows Segoe UI family. The current logo uses cached native vectors reconstructed from the supplied horizontal design. Its rebuilt final l contains the only animated wheel opening.

## Caption logo

The 48-pixel caption now contains a 120 by 40 logical pixel native logo at (20,4), replacing the painted app-name text. Its color surface matches the integrated caption, its aspect ratio remains 3:1, and its pointer hit-test result is transparent so the parent caption continues handling dragging and resizing. Minimize, maximize, and close keep their existing positions. The title logo uses the existing clock, reduced-motion preference, and visibility policy.

## Version 0.6.3 single-logo layout

Only the animated title-bar logo remains. The sidebar logo control and its repaint path are removed. Supporting sidebar text starts at y=76 and navigation at y=136, closing the former logo space. The title-bar logo keeps its existing scale, animation policy, and pass-through hit testing. Verification remains headless.
