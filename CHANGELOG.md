# Changelog

## 0.6.10 candidate - Unreleased

- Fix settings saves that exceeded the loader's 64 KiB limit; oversized saves now preserve the last valid file. Reads are bounded before parsing.
- Use exclusively created temporary settings files and remove them on failure.
- Validate sign-in registration against this executable, registry value type, Unicode path and Windows command-length limits; close registry handles on all exits.
- Cancel scroll tails when Shift changes, and brake on auxiliary mouse buttons.
- Pause output when the hook thread exits unexpectedly. Treat UI message-loop errors as failures and clean up owned windows.
- Add pinned Rust tooling, a Windows CI workflow, repeatable validation, dependency-advisory checks, candidate packaging and adversarial package verification.
- Add locked-destination recovery and concurrent-save integrity checks, plus an isolated packaged-app probe for Unicode portable paths, corrupt settings, failed saves and clean shutdown. Package rejection checks now require a valid baseline first.
- Include mouse acceptance and rollback instructions in the candidate ZIP; correct stale build versions, packaged license paths and the static-runtime description.
- Full comparison and remaining production gates: `source/docs/PRODUCTION-AUDIT-2026-10-21.md`. The published 0.6.9 release remains unchanged.

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
