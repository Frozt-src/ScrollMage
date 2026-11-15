# Scroll Mage 0.6.8 review validation

Historical local review record, captured before publication. These source changes are included in 0.6.9; see the repository changelog for the published release.

Verified October 18, 2026, on a local branch based on the `v0.6.7` tag. Source changes are uncommitted; no GitHub release was published.

## Implemented behavior

- Default is editable and keeps its saved tuning when switching away and back. A Custom Default indicator remains visible for modified tuning, including after saving and restoration. Unsaved edits show a pending indicator and require Apply changes.
- macOS style and Super smooth are fixed, locked presets. Selecting a preset saves and activates it immediately. Failed saves retain the previous active preset and restore its selection; unrelated global option drafts are preserved.
- App profiles provides custom distance/response, either fixed preset, or pass-through per EXE. Save and Remove take effect immediately. Legacy overrides survive migration; invalid/duplicate entries are rejected without replacing saved settings.
- Activity and the status footer distinguish ready, recent accepted/output events, pass-through reasons, pause, target unavailability and errors. Tagged self-output does not inflate bypass counters. These observations do not prove continuous hook health.
- Native target resolution respects Windows wheel routing. Mouse-position routing supports eligible inactive windows under the pointer. Target/root/child, routing, modifiers and eligibility are checked before output; unchanged polling preserves the smoothing tail.
- The maximize control exposes the native Snap hit target and forwards nonclient interaction. Compact navigation supports a 500 logical-pixel minimum width. Both preset selectors refresh their row heights after a live DPI transition.
- Friction and elasticity remain explicitly labeled as Scroll Test drag effects. No universal elastic-edge behavior is claimed.

See [the implementation comparison](SCROLLING-RESEARCH-2026-10-18.md) for pinned SmoothScroll source references, Microsoft contracts, input classification and horizontal-transport boundaries. No external implementation code was copied.

## Passed checks

| Check | Result |
| --- | --- |
| Integration logic suites | 53 passed: branding 3, control motion 2, diagnostics 5, engine 10, frost 2, layout 8, logo lockup 2, motion 6, settings 11, spring upgrade 4 |
| Hidden native UI fixtures | 11 passed, including preset save/rollback, editable Default restoration and persistent Custom indication, app profile persistence/errors, new-page layout/drawing, Snap hit testing and live DPI changes |
| Controlled Windows input receiver | 1 passed with actual SendInput output and a live receiving hook; hook entry used a constructed event |
| Formatting | Passed |
| Clippy, all targets, lint warnings denied | Passed; Cargo reported incremental-cache hard-link fallback warnings and copied cache files instead |
| Static CRT AMD64 release build | Passed with locked offline dependencies and remapped local paths |
| Native renders | Inspected App profiles and Activity, including 500 logical-pixel width; fixtures cover 100/125/150/175/200 percent scaling |
| Read-only review | Settings/runtime and whole change reviewed; the scoped selector review found a DPI-height defect, then approved its fix and regression coverage; Custom Default indication approved separately |
| Review ZIP | All 38 listed SHA-256 hashes verified; no personal settings, source files or PDBs present |
| Packaged executable | REA static PE resource inventory complete; PE32+, AMD64, manifest identity 0.6.8.0, PerMonitorV2 DPI awareness |

Total: **65 passing tests**. A controlled receiver is evidence of Windows input delivery, not physical-device feel or universal app compatibility.

Relevant commands from the project root:

```powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'source/target'
cargo fmt --manifest-path source/Cargo.toml --all -- --check
cargo clippy --manifest-path source/Cargo.toml --offline --locked --all-targets -- -D warnings
cargo test --manifest-path source/Cargo.toml --offline --locked --release --target x86_64-pc-windows-msvc --test layout --test settings --test branding --test engine --test motion --test spring_upgrade --test logo_lockup --test control_motion --test frost --test diagnostics
cargo test --manifest-path source/Cargo.toml --offline --locked --release --target x86_64-pc-windows-msvc --bin scrollpilot hidden_native_ -- --nocapture --test-threads=1
cargo test --manifest-path source/Cargo.toml --offline --locked --release --target x86_64-pc-windows-msvc --bin scrollpilot owned_window_receives_smoothed_tagged_input_and_worker_stops_when_idle -- --nocapture --test-threads=1
$env:RUSTFLAGS = '-C target-feature=+crt-static --remap-path-prefix=C:\Projects\ScrollMage=/build --remap-path-prefix=%USERPROFILE%\.cargo=/cargo'
cargo build --manifest-path source/Cargo.toml --offline --locked --release --target x86_64-pc-windows-msvc
git diff --check
```

The live receiver requires desktop access. Its evidence reports 120 vertical and 120 horizontal wheel units, 48 output events, 49 received events, no additional idle ticks, maximum observed hook duration 66 microseconds, and passed self-injection loop prevention. Those timings describe this bounded fixture, not a performance guarantee. Compiler: Rust 1.98.1 (48a229cea, 2026-09-01).

Ignored local evidence: `source/evidence/0.6.8-native-verification.json`, `0.6.8-pe-review.json`, `0.6.8-manifest.xml`; native renders: `source/target/headless-ui/`.

## Review artifacts

- Local executable: `dist/ScrollMage-0.6.8-review-windows-x64/ScrollMage/ScrollMage.exe`
- ZIP: `dist/ScrollMage-0.6.8-review-windows-x64.zip`
- Executable SHA-256: `8a0766af8e66097353cbb0d04917a4dc5b366bbede9501c62cbc40ff608ec697`
- ZIP SHA-256: `3f9bee0ef2cea08ef146c9f1c9a079230f41d5fecf25a933473d43d2867e6891`

The staged local folder contains a copy of the existing settings, with only Diablo IV excluded, to support a comparison with the user's current setup. That copy was added after ZIP creation and is intentionally absent from the ZIP/hash list. The ZIP starts with the app's defaults, which include browser exclusions. Quit the existing Scroll Mage instance from its tray menu before launching the review copy; otherwise the single-instance guard brings the existing copy forward.

Original files preserved during this implementation/package step:

| File | SHA-256 |
| --- | --- |
| Root `ScrollMage.exe` | `3715f743297e37b4750b43c35442dabe422b0b64107623fa048471371e2db594` |
| Root `settings.json` | `cc8742a9d59b0852d34e4e00f92c59d1bf8c811926f2dd99a232829e3e8e36aa` |
| Published 0.6.7 ZIP | `87100e64553bf1e9cc1f3b749993a72d761bf459d8413efb9f141473b045b03f` |

## Live acceptance still required

- **NOT RUN:** physical mouse or trackpad A/B in the user's apps. Fine-resolution and external injected input still intentionally pass through; a named app may accumulate small deltas into line-sized updates.
- **NOT RUN:** visible Windows 11 hover Snap flyout and shell mouse interaction. Native hit-test/forwarding and keyboard control paths are covered by hidden tests; that does not establish the visible shell behavior.
- **NOT RUN:** arbitrary-app elasticity or alternate horizontal compatibility transport. These are not implemented or advertised.

The review build was staged without replacing or restarting the user's existing running app. Its current enabled state is separate from the new executable's behavior.
