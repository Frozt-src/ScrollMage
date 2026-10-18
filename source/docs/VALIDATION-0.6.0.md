# Scroll Mage 0.6.0 validation

October 17, 2026. Headless verification for the settings, preset, exclusions, and minimize/restore changes.

## Automated evidence

- 36 focused tests cover engine conservation, reversal, fractional residuals, momentum and elasticity, layout, branding, settings persistence, and migration. New tests prove that preset identity survives manual tuning and that smoother presets use smaller steps without adding wheel travel or changing app choices.
- One hidden native-control integration test runs at 100%, 125%, 150%, 175%, and 200% scale. It creates native controls without a visible main window, input hook, or tray icon. It checks all six button/keyboard step pairs, lower-bound clamping, preset commit notifications, deliberate Apply persistence, sidebar pause placement, bounded 128-entry exclusions and access to the final entry, minimized-layout preservation, and light final-pixel coverage at the Scroll Test edge. It verifies that its own window never becomes the foreground window.
- Reinstating the old truncated preview dimensions makes that test fail at 125% scaling with a dark edge pixel of 37383b. Reinstating the old minimized-layout behavior makes it fail with layout 199 by 34 rather than 1140 by 768. The corrected implementation passes both checks.
- Native font readback resolves the body font to Segoe UI. The existing logo artwork is embedded unchanged. SF Pro is not distributed or selected.
- Formatting, clippy with warnings denied, and the release build are checked before installation. Recorded build output and installation hashes accompany the staged delivery.

## Limits

Headless renders exercise actual control painting into memory bitmaps. They do not establish physical-wheel feel, display ghosting or tearing, multi-monitor transitions, a measured match to macOS, real shell tray interaction, or successful global hotkey capture on a desktop where another app may own Alt+P. The registration failure path keeps the in-app and tray pause controls available. Desktop automation is reserved for closing the prior running app if required to replace its executable.

Outer corners remain the Windows-controlled small-radius preference, as documented in VALIDATION-0.5.3.md. Inner cards are rendered with a four logical pixel radius. The requested SF Pro change was not made under Apple's published standard font license. Sources and preset rationale are in README.md.
