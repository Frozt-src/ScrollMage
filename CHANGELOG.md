# Changelog

## 0.6.10 candidate - 2026-10-21 (prerelease)

- Fix settings saves that exceeded the loader's 64 KiB limit; oversized saves now preserve the last valid file. Reads are bounded before parsing.
- Use exclusively created temporary settings files and remove them on failure.
- Validate sign-in registration against this executable, registry value type, Unicode path and Windows command-length limits; close registry handles on all exits.
- Cancel scroll tails when Shift changes, and brake on auxiliary mouse buttons.
- Pause output when the hook thread exits unexpectedly. Treat UI message-loop errors as failures and clean up owned windows.
- Add pinned Rust tooling, a Windows CI workflow, repeatable validation, dependency-advisory checks, candidate packaging and adversarial package verification.
- Add locked-destination recovery and concurrent-save integrity checks, plus an isolated packaged-app probe for Unicode portable paths, corrupt settings, failed saves and clean shutdown. Package rejection checks now require a valid baseline first.
- Include mouse acceptance and rollback instructions in the candidate ZIP; correct stale build versions, packaged license paths and the static-runtime description.
- Define Scroll Mage as a desktop-mouse scrolling extension. Touchpads are not supported: fine-grained touchpad scrolling is left to Windows, and Activity now names it.
- Release the code under the MIT License and the artwork under CC BY 4.0; add contributing, security and conduct guides plus issue and pull-request templates.
- Keep heading and small-label fonts correct after the window moves to a display with different scaling.
- Scroll the app's own pages and the exclusion list with high-resolution wheels that send less than one notch at a time.
- Fix removal of exclusion rows so that it no longer changes the control list while Windows is still reading it.
- Ask Windows to let the running instance come to the front when the app is opened a second time.
- Show the package version in the app and in the executable's file properties (product name, version and copyright).
- Replace the spring-and-coast scrolling engine with one momentum model: each notch starts at full speed and eases out exponentially, and Super smooth's free-spin is the same model with lower friction while a flick glides. Scroll distances stay exact at any frame rate. "Spring response" is now called **Response** and keeps its saved values.
- Make the presets feel distinct. **macOS style** eases out about twice as long as Default and a fast spin travels about 2.5 times as far; **Super smooth** has the softest ease-out as well as free-spin glide.
- Smooth browsers by default. Fresh settings no longer exclude Firefox, Chrome and Edge; existing exclusion lists are kept.
- Keep a scroll in motion when a wheel notch arrives while a frame is being sent. Previously that notch passed through unsmoothed and cancelled the current motion, most often during fast spins.
- Fix locked tuning fields drawing each new value over the old one. Fixed presets now show their values on a read-only background with inactive steppers, the sidebar uses Windows icons, the save button turns into a quiet "Settings saved" note when there is nothing to apply, and each preset describes how it feels.
- Redesign the Activity page: a status card with a coloured dot and the current app, live tiles for Accepted, Output and Bypassed counts that stack on narrow windows, and a shorter "How it works" card. Screen readers still read the status and every count.
- Smooth sooner after the pointer moves to another window. The first wheel notch there still passes through, but it now triggers an immediate look-up of the new app instead of waiting up to a quarter of a second.
- Re-install the mouse hook when the pointer moves but the hook has seen no mouse input, which is how a hook that Windows removed silently shows itself.
- Deliver a wheel notch that was queued during a busy frame even when the engine cannot take it.
- Full comparison and remaining production gates: `source/docs/PRODUCTION-AUDIT-2026-10-21.md`. Version 0.6.9 remains the previous stable release; this candidate is for acceptance testing.

## 0.6.9 - 2026-10-18

### Added

- **Super smooth free-spin:** a quick wheel flick starts fast scrolling that gradually slows. Slow individual notches remain precise. Reverse the wheel, click the left/middle/right mouse button, or pause to stop the coast.
- **App profiles:** give an EXE custom distance/response, a fixed preset, or pass-through behavior. Existing custom and bypass profiles survive settings migration.
- **Activity diagnostics:** see accepted, emitted, and bypassed input counts, plus explanations for exclusions, precision input, modifiers, unavailable targets, and output errors.
- **Custom Default indicator:** modified Default tuning stays clearly identified after saving and switching presets.

### Changed

- Preset selection saves and activates immediately. **Default** remains editable and remembers saved tuning; **macOS style** and **Super smooth** keep their fixed numeric values locked.
- Super smooth includes bounded software momentum, independent of the Scroll Test drag-friction control. It applies to both the global preset and fixed per-app Super smooth profiles.
- The settings window supports narrower widths with compact navigation.

### Fixed

- Target selection now follows Windows wheel-routing preferences, including eligible inactive windows under the pointer.
- Target changes, bypassed input, pause, mouse-button presses, and long worker stalls cancel pending momentum.
- Failed preset/profile saves preserve the previous active and saved settings. Invalid or duplicate app profiles are rejected.
- Native maximize-button hit testing and forwarding support Windows Snap integration.
- Preset selector row heights update when moving between monitors with different scaling.

### Validation and limits

- 73 tests cover scrolling logic, settings, hidden native UI, and a controlled Windows input receiver, including free-spin delivery and click/pause braking.
- The controlled receiver uses constructed source events and actual Windows output. Physical mouse feel and the visible Windows Snap flyout still need live acceptance.
- Apps control how they display incoming wheel events; some accumulate small deltas into larger steps. Fine-resolution and externally injected input still pass through.
- Free-spin is simulated in software and does not change physical wheel detents or detect a finger touching the wheel. Friction and elasticity settings affect dragging in Scroll Test only.

### Download

- Portable Windows x64 ZIP: `ScrollMage-0.6.9-windows-x64.zip`.
- Extract the entire folder, quit any older tray instance, and run `ScrollMage.exe`. Select **Super smooth** to try free-spin.
- Browsers are excluded by default. Remove the browser from **Excluded apps** to enable Scroll Mage there.
- Includes startup instructions, third-party licenses, per-file checksums, and a separate ZIP checksum. No personal settings are bundled.

## 0.6.7 - 2026-10-18

- Initial GitHub release.

[0.6.9 release](https://github.com/Frozt-src/ScrollMage/releases/tag/v0.6.9) | [Compare with 0.6.7](https://github.com/Frozt-src/ScrollMage/compare/v0.6.7...v0.6.9)
