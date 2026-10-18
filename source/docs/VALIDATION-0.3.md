# Validation

Date: 2026-10-17. Version: 0.3.0. Platform: Windows x64 MSVC.

## Automated checks

- PASS: cargo fmt --all -- --check
- PASS: cargo clippy --offline --all-targets -- -D warnings
- PASS: 25 tests: 10 engine, 5 motion, 4 settings, 4 spring upgrade, 2 adaptive layout.
- PASS: cargo build --release --offline
- The pre-existing native input integration test was not rerun in this UI turn. It opens and activates a desktop test window; desktop testing was stopped by the user. This is not included in the 25-test count.

The layout checks cover 940x680, 1100x760, 1280x920, and 1600x1000 logical dimensions, ensuring card separation, reachable content, and visible footer actions. They are geometry checks rather than physical window-resize screenshots.

## Live review

An isolated review copy used a copy of the installed settings. Observed: the main settings layout, blue wave icon, cards, native control names, checkbox controls, live preview surface, and footer actions. The pause action changed the status heading, status switch, and primary action. The distance stepper changed 120 to 121 in the review copy. Those test edits were not applied to the installed settings.

The review found and corrected dark-theme label backgrounds and overlapping helper text. After the label fix, a new screenshot showed the light card design. Further source polish addressed caption repainting, list scrollbar visibility, keyboard steppers, navigation selection state, and preview viewport bounds.

Final live verification was INTERRUPTED by the user pressing Escape. No more Computer Use calls were made. Full Preview/About navigation, add/remove editing, saving through the new interface, resize/DPI behavior, and final polish appearance remain unverified interactively. The final package passes compilation, lint, and the 25 non-desktop tests.

## Color measurements

Computed sRGB text contrast ratios:

- Primary text on white: 18.21:1.
- Supporting text on white: 5.14:1.
- Supporting text on canvas: 4.80:1.
- White primary action text on blue: 4.97:1.
- Selected navigation text on its pale blue background: 5.11:1.

These measurements cover those text pairs, not a complete accessibility certification. No new CPU, memory, scrolling latency, or comparative performance claim is made in this UI release.

The packaging audit report is stored separately under Outputs/ScrollPilot-Verification. It verifies active-file naming, installed executable equality, packaged source equality, and preserved installed settings. Historical snapshots remain outside the active source and application folders.
