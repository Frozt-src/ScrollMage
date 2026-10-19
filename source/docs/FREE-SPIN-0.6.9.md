# Super smooth software free-spin, 0.6.9

Historical local validation record, captured before publication. The published ZIP and its current checksum are listed in the GitHub 0.6.9 release; the review artifacts below are retained as original evidence.

Verified October 18, 2026. The user's requested behavior is a fast, sustained glide after one quick wheel flick, inspired by Logitech free-spin.

## Behavior and implementation

[Logitech's setup guide](https://hub.sync.logitech.com/mx-master-3s/post/mx-master-3s---setup-guide-logIbt80SWDODOk) describes a wheel that switches from ratcheted motion to physical free-spinning as scrolling speeds up. Scroll Mage emulates the sustained scrolling in software. It cannot remove physical detents or sense a finger braking the wheel.

The earlier Super smooth setting only changed a finite spring follower's response. Each notch still produced a finite distance; there was no independent source of post-flick momentum. The new behavior adds a bounded exponential coast exclusively to Super smooth:

- Consecutive same-direction notches no more than 80 ms apart, or a packet containing at least two notches, launch coasting. One ordinary notch remains precise because that event alone cannot identify a physical flick.
- Input rate sets coast speed, capped at 18,000 wheel units/second. Additional flicks reinforce the coast without accumulating unlimited velocity.
- Speed decays at 1.2/second and stops at 30 wheel units/second. These are original fixed software tuning values, not measurements of Logitech's wheel or proprietary constants.
- Coast displacement is integrated analytically and translates both spring coordinates. The existing finite impulse follower and fractional output remain intact; tested frame rates do not change total flick distance by more than one wheel unit.
- Reversing discards the old direction before accepting a precise reverse notch. Left/middle/right button presses, pause, target changes, bypassed input and failed eligibility checks retain their existing cancellation path.
- A frame gap above 250 ms cancels the coast instead of delivering a large catch-up burst.
- Default, macOS style and explicit custom app profiles keep their previous finite response. A fixed Super smooth app profile enables the new behavior. Numeric controls for both fixed global presets remain locked and selection still saves/applies immediately.
- The friction/elastic controls continue to affect Scroll Test dragging only. They do not tune free-spin or introduce elastic edges in arbitrary apps.

The UI identifies Super smooth with "Fixed. Flick to coast." Activity and README explain braking. Existing precision/injected-input bypass, exclusions, integrity checks and tagged-loop prevention remain enforced.

## Verification

**73 tests passed:** 61 integration logic tests, 11 hidden native UI tests, and one live Windows receiver fixture. Formatting, Clippy with lint warnings denied, and the locked offline AMD64 static-CRT release build passed.

The new tests first failed because quick flicks produced only 360 units with no later output. Eight free-spin tests now cover sustained traversal, precise slow notches, a grouped input packet, reverse braking, cancellation, stalls, frame-rate consistency, speed bounds, axis independence and global/per-app mode selection. The hidden preset test also verifies that the correct mode and explanatory hint activate immediately and disappear on returning to Default.

A deterministic trace of three 120-unit inputs at 0/20/40 ms produced:

| Measurement | Result |
| --- | --- |
| First sampled second after the last input | 5,778 wheel units |
| Output during 1.5-2.5 seconds | 914 wheel units |
| Total output | 8,012 wheel units |
| Last nonzero output | 4,768 ms after the first input |
| Idle by 7 seconds | Passed |

The controlled Windows receiver got 2,699 units in the early interval after a constructed three-notch flick and another 3,623 units in the following 1.2-second interval, with no additional source input. Click and pause both canceled delivery. The fixture also retained its original 120-unit vertical/horizontal conservation, tagged-loop prevention and idle-worker checks. Maximum observed hook duration was 565 microseconds. These timings describe one bounded run, not a general performance guarantee.

**Physical mouse input: NOT RUN.** The hook entry used constructed events; output used actual Windows SendInput and a live receiver. This establishes delivery through Windows, not the perceived feel or visible frame rate in the user's apps. Fine-resolution and externally injected input still bypass smoothing, and individual apps may accumulate small deltas into larger steps.

Commands, from `source`, with `CARGO_TARGET_DIR` set to its `target` folder:

```powershell
cargo fmt --all -- --check
cargo clippy --offline --locked --all-targets -- -D warnings
cargo test --offline --locked --release --target x86_64-pc-windows-msvc --test free_spin --test layout --test settings --test branding --test engine --test motion --test spring_upgrade --test logo_lockup --test control_motion --test frost --test diagnostics
cargo test --offline --locked --release --target x86_64-pc-windows-msvc --bin scrollpilot hidden_native_ -- --nocapture --test-threads=1
cargo test --offline --locked --release --target x86_64-pc-windows-msvc --bin scrollpilot owned_window_receives_smoothed_tagged_input_and_worker_stops_when_idle -- --nocapture --test-threads=1
```

Local evidence: `source/evidence/0.6.9-native-verification.json`, `0.6.9-pe-review.json`, and `0.6.9-manifest.xml`. The executable's static PE resource inventory confirmed PE32+ AMD64 and its 0.6.9.0 manifest identity. Native test renders remain under `source/target/headless-ui/`.

## Review build

- Executable: `dist/ScrollMage-0.6.9-review-windows-x64/ScrollMage/ScrollMage.exe`
- ZIP: `dist/ScrollMage-0.6.9-review-windows-x64.zip`
- Executable SHA-256: `c0e819ffcb292fc542b03500e0f000b8b364b41b78f7170e244e32f6c0ad1959`
- ZIP SHA-256: `bbc84b2f6f08f61065c90a94b6c89b2092a9abd0d9d4075b6cd20d88001f0444`

All 38 hashes listed in the ZIP were verified. No personal settings, source files or PDBs are included. The local staged folder received a separate settings copy after packaging, with Super smooth selected and the original Diablo IV exclusion retained. That local settings copy is intentionally outside the ZIP hash list. Quit the existing tray instance before opening the review executable, so the single-instance guard does not bring the old copy forward.

The original root executable and settings, the published 0.6.7 ZIP, and the previous 0.6.8 review ZIP are preserved. The running app was not replaced or restarted, and no GitHub release was published. Source changes remain local on `codex/profiles-and-scroll-diagnostics`.
