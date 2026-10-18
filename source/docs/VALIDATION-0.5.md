# Version 0.5 interface validation

2026-10-17. Expanded Scroll Test, individual exclusion cards, status inside Options, and a matching custom title bar.

## Automated checks

PASS: formatting, clippy across all targets with warnings denied, release build, and 32 focused tests: branding 3, engine 10, layout/frame 5, motion 5, settings 5, spring 4. Layout cases cover compact and large windows, 0 through 128 exclusion cards, no overlap, test growth, and frame hit regions. The settings migration test exercises actual load/save, verifies legacy off choices become visible exclusions, removes hidden custom tuning, avoids duplicate names, omits empty profiles on save, and preserves original file bytes until saving.

## Live Windows checks

The candidate used a copied settings file under E:/Media/REA-L/Outputs/ScrollMage-UI-Preview. The user's C:/Projects/ScrollMage/settings.json was not written.

PASS: visual review at normal and maximized sizes; main header logo absent; top-aligned cards; expanded Scroll Test; status and switch at the bottom of Options; individual browser and game exclusion cards; no profiles editor in the accessibility tree.

PASS: Add creates a separate named card. Removing the temporary unsaved test card restores the original exclusion list. No real exclusion was removed and no test entry was saved.

PASS: wheel input over the expanded test surface moved rows and showed Wheel -480, Events 1.

PASS: custom maximize changes its accessible name to Restore and fits the desktop work area; Restore returns the previous window size. Dragging the lower-right edge produces a compact stacked settings layout. The dedicated Scroll Test page fills the available content area at the compact size.

PASS: custom Minimize produced a confirmed minimized-window state from the desktop tool. Activating the app restored the compact Scroll Test page.

INTERRUPTED: custom close-to-tray check. The user pressed Escape to stop Computer Use. No further desktop input was issued. The final rounded-corner clipping polish was compile/lint checked after the visual checks; it was not separately replayed on the desktop.

NOT RUN: physical mouse hardware trials, multi-monitor DPI movement, Windows animation preference changes, native injected-input integration fixture, and a full accessibility audit. The custom maximize button does not implement the Windows 11 hover Snap Layout flyout.

## Preservation

The original app and source are backed up outside the project. Proprietary licensing, vendor license files, graphics supplied by the user, logo assets, settings, and unrelated source remain intact. The wheel engine, motion integrator, and input-policy modules are unchanged. Package metadata is 0.5.0; the executable remains ScrollMage.exe.
