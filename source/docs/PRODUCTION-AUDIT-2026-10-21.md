# Scroll Mage production-readiness audit

Verified October 21, 2026. **Verdict: improved release candidate, not yet production-approved and not feature-equivalent to SmoothScroll.**

Audited local `main` baseline the `v0.6.9` tag (published 0.6.9), then made original fixes on a local branch as 0.6.10 candidate. No published release or running app was replaced. No Scroll Mage process was found during the process-name check. The root executable, personal settings and published 0.6.9 ZIP retain their original hashes.

## Method and reference boundary

SmoothScroll master resolved to [`94a6680e852b6dbd3e6ea079fc826fb3befa601b`](https://github.com/quangtruong2003/SmoothScroll/commit/94a6680e852b6dbd3e6ea079fc826fb3befa601b), with the inspected Tauri config declaring version 1.36.0. Source was read for settings, Windows hook/emitter, effective settings resolution, game mode, backup/monitor UI, updater, CI and release workflow. This establishes source-level features; it is not a runtime compatibility or performance benchmark of that application.

The reference uses [FSL-1.1-Apache-2.0](https://github.com/quangtruong2003/SmoothScroll/blob/94a6680e852b6dbd3e6ea079fc826fb3befa601b/LICENSE), with restrictions on competing commercial use. No source or assets were imported; the fixes here were implemented against Scroll Mage's code and documented platform contracts.

There is no single recognized “industry gold standard” implementation to copy. This audit uses explicit Windows contracts, bounded input work, transactional persistence, dependency checks, least-privilege CI, artifact integrity and honest acceptance evidence. A competing app's settings or architecture are not proof of correctness.

## Confirmed issues and fixes

| Finding | Before | Candidate change and evidence |
| --- | --- | --- |
| P1: Shift semantics could change during a tail | The engine captured an axis when input arrived, but emission did not compare the current Shift state. A previously accepted tail could reach an app under a new modifier meaning. | Capture Shift with accepted input, clear on a new gesture's Shift transition, and require the captured state before each output. The controlled Windows fixture failed the new mismatch assertion before the emission fix, then passed. This checks the actual emission predicate; it does not simulate physical keyboard timing. |
| P1: Hook-thread loss did not disable output | `hook_live` became false when the thread exited, while the worker could remain enabled and finish its tail. | Emission requires `hook_live`; an unexpected installed-hook thread exit pauses and records an actionable fault. The controlled fixture ends the hook thread and verifies both hook-live and enablement become false. Windows can still silently remove a hook without ending its thread; this fix does not claim to detect that case. |
| P2: A valid save could break the next load | Each list could validate while serialized settings exceeded the loader's 65,536-byte limit. Save would replace the old file; the next launch rejected it. | Serialize and check the same size budget before creating/replacing any file. Loading reads at most 65,537 bytes before rejecting an oversized input. The regression failed against the old save and now proves the original file remains reloadable. |
| P2: Failed saves left predictable temporary files | The PID-only temporary file was created/truncated; replacement/write failures did not remove it. | Exclusive creation with a per-process sequence; an owned guard closes and removes only its own file on every return path. The failed-replacement regression initially found leftover files and now passes. Existing unowned files are not deleted. |
| P2: Stale sign-in registration looked enabled | The UI checked only whether a Run value had data, regardless of executable path or registry type. An old ZIP location could still be launched on sign-in. | Read bounded typed UTF-16 data and compare the complete quoted command to this executable. Preserve Unicode paths, reject embedded NULs and overlong commands before writing, and close keys with an owned guard. The pure command regression covers current/stale paths, wrong types, suffixes, termination, Unicode and length; no user registry values were changed during testing. |
| P2: UI message-loop failure looked successful | `GetMessageW == -1` exited the loop and returned `Ok(())`; owned windows could outlive their boxed state until process exit. | Preserve the error, destroy any remaining owned main window, pause and clean up, then return an error. The ordinary shutdown/UI suite compiles and passes; an actual OS-generated GetMessage failure was not injected. |
| P2: Auxiliary buttons did not brake coasting | Only left/middle/right button-down messages interrupted a gesture. | X-button down also cancels the engine. Existing primary-button braking remains verified; a physical auxiliary-button check is still required. |
| Release-process gap: manual-only validation and packaging | No tracked GitHub CI workflow; release creation depended on session-specific commands. | Add pinned Rust tooling, `tools/Validate.ps1`, dependency checks, `tools/Package.ps1`, strict ZIP verification and three adversarial package checks. Windows workflow uses read-only contents permission, pinned action SHAs, no stored checkout credentials, timeouts, and explicit candidate uploads. It does not publish releases. Local equivalents pass; hosted CI has not run. |

## Feature comparison

Owner-confirmed production scope (October 21, 2026): Windows x64 mouse-wheel smoothing across applications, with per-app exclusions to disable it where needed. The owner uses a mouse, not a trackpad. Trackpad smoothing, monitor-specific assignments and full SmoothScroll feature parity are outside this release target. Existing optional per-app tuning does not have to be expanded or removed to meet this scope. This scope decision does not change saved settings or the candidate's default exclusions.

“Present” means implemented in inspected source, with focused local tests where described above. It does not imply every Windows application renders smooth movement.

| Capability in the reference | Scroll Mage status | Remaining work |
| --- | --- | --- |
| Native wheel interception, easing, fractional output | Present: exact critically damped followers; different algorithm | Visible feel/performance acceptance; no need to replace this solely to copy another scheduler |
| Distance, response and acceleration | Present globally | Reference also offers easing families, animation/tail controls and configurable maximum velocity; these controls are absent |
| Natural/reverse direction | Present: one global direction option | Separate horizontal inversion and direction synchronization are absent |
| Horizontal/Shift-wheel behavior | Partial: native horizontal output and fixed Shift conversion | Configurable Shift preservation/conversion and compatibility transport are absent |
| Discrete/raw/whole-notch app modes | Partial: pass-through/exclusions | No transformed whole-notch mode or per-app transport selector |
| Per-app profiles | Partial: EXE-specific distance/response or fixed preset | No reusable named profile library or complete per-app acceleration/direction/modifier overrides |
| Monitor profiles and app-by-monitor assignments | Missing | Monitor identity, precedence, UI editor and migration tests |
| Known-game foreground bypass | Missing | Manual exclusions are not equivalent to the reference's game-mode policy |
| Touchpad/source classification | Missing/intentional bypass | Fine-resolution and external injected input pass through; classification requires real devices and loop-prevention tests before broadening interception |
| Native Windows/hosted-app policies | Partial/unverified | No dedicated hosted-app identity resolver or native-smooth auto-disable/force-enable policy; test hosted Store apps and shared frame hosts explicitly |
| Ctrl/Alt modifier semantics and smooth zoom | Partial: raw pass-through | No configurable modifier policy or smooth zoom/inversion/sensitivity |
| Edge auto-scroll | Missing | Requires an explicit opt-in design and cancellation tests |
| Tray toggle, close to tray, global hotkey | Present | Hotkey is fixed Alt+P; user-configurable accelerator and related options are absent |
| Sign-in startup | Present; stale-path check fixed | Real sign-out/sign-in and moved-portable-folder acceptance are still needed |
| Theme/language controls | Missing | Hard-coded light palette and English UI; system/high-contrast behavior needs work and acceptance |
| Reduced-motion control | Partial | Branding/control animation respects the OS signal; wheel smoothing has no user-selectable follow/always/never policy |
| Backup/import/reset workflow | Missing UI | Atomic local JSON persistence exists, but is not a user-facing validated import/export or reset feature |
| Statistics/onboarding | Partial | Activity session counters exist; reference-style usage statistics and guided onboarding do not |
| Installer, uninstall and automatic updates | Missing | Portable ZIP and manual updates only. A secure updater needs signed metadata/payload validation and rollback testing; do not add an unsigned download-and-run shortcut |
| Platforms outside Windows x64 | Missing | Windows x64 is the declared scope; cross-platform work is not a Windows production blocker |
| Input/output diagnostics | Present | Counters report observed events, not visible frame rate or a continuous hook-health guarantee |
| Long software free-spin after a flick | Present in Super smooth | Physical device/app feel remains unverified; the reference is not assumed to have an identical free-spin mode |

Reference evidence: [settings and policy fields](https://github.com/quangtruong2003/SmoothScroll/blob/94a6680e852b6dbd3e6ea079fc826fb3befa601b/crates/core/src/settings.rs), [effective app/monitor resolution](https://github.com/quangtruong2003/SmoothScroll/blob/94a6680e852b6dbd3e6ea079fc826fb3befa601b/src-tauri/src/hook_wiring.rs), [Windows source handling](https://github.com/quangtruong2003/SmoothScroll/blob/94a6680e852b6dbd3e6ea079fc826fb3befa601b/crates/platform/src/windows/mouse_hook.rs), [emission planning](https://github.com/quangtruong2003/SmoothScroll/blob/94a6680e852b6dbd3e6ea079fc826fb3befa601b/crates/platform/src/windows/wheel_emitter.rs), [known-game policy](https://github.com/quangtruong2003/SmoothScroll/blob/94a6680e852b6dbd3e6ea079fc826fb3befa601b/src-tauri/src/game_mode.rs), [backup UI](https://github.com/quangtruong2003/SmoothScroll/blob/94a6680e852b6dbd3e6ea079fc826fb3befa601b/src/components/settings/BackupSection.tsx), [monitor UI](https://github.com/quangtruong2003/SmoothScroll/blob/94a6680e852b6dbd3e6ea079fc826fb3befa601b/src/components/settings/MonitorProfiles.tsx), [updater](https://github.com/quangtruong2003/SmoothScroll/blob/94a6680e852b6dbd3e6ea079fc826fb3befa601b/src/lib/updater.ts), [Windows release workflow](https://github.com/quangtruong2003/SmoothScroll/blob/94a6680e852b6dbd3e6ea079fc826fb3befa601b/.github/workflows/release-windows.yml).

## Pipeline and interface assessment

Scroll Mage's runtime path is **settings validation → effective EXE tuning → target eligibility → dedicated low-level hook → bounded two-axis engine → timer-driven worker → tagged SendInput → observed output → Activity**. Persistent changes are saved before publishing new settings to the runtime. Heavy process identity checks stay outside the hook; contention passes input through. Engine work is constant-size; the worker sleeps when idle. The existing architecture has useful production properties.

The interface is native Win32, with separate UI, settings, engine and platform modules. There is no web front end, remote back end, database or network service in this application, so adding React, Tauri, a server, web authentication or a database is not a readiness requirement. The equivalent UI concerns are native accessibility semantics, keyboard use, focus visibility, high contrast, DPI, cancellation and truthful state. The roughly 2,000-line UI coordinator and extensive unsafe FFI remain maintainability/review costs; no claim of memory-safety proof or complete accessibility certification is made.

The hook uses nonblocking lock acquisition and moves animation to a worker, consistent with [Microsoft's low-level mouse-hook guidance](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelmouseproc). Microsoft documents [GetMessage's error/quit distinction](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessagew), [wheel modifiers/small deltas](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-mousewheel), [SendInput/UIPI and current keyboard state](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput), [registry type/termination concerns](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regqueryvalueexw), and the [Run command-length limit](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys). These contracts, not reference code, guided the fixes.

Build tooling follows [Cargo's test selection/filtering contracts](https://doc.rust-lang.org/cargo/commands/cargo-test.html) and [GitHub's workflow security guidance](https://docs.github.com/en/actions/reference/security/secure-use): minimum permissions, immutable action references, no automatic deployment on unreviewed changes. Candidate packaging builds from the lockfile, uses static CRT, remaps source/cache paths, generates dependency-license inventory, copies an explicit distributable set, hashes every member, verifies the ZIP and excludes personal state. Its checksum is an integrity check, not an authenticated publisher signature or a reproducible-build proof.

## Verified checks

| Check | Result |
| --- | --- |
| Logic suites | PASS: 65 tests, including settings size, cleanup, locked-destination recovery and concurrent-replacement integrity |
| Hidden native UI/command suite | PASS: 12 tests, including startup-command validation |
| Controlled Windows receiver | Earlier audit PASS: 1 fixture, including actual SendInput, self-tag handling, idle sleep, free-spin, click/pause braking, Shift mismatch rejection and unexpected hook-thread exit. Not rerun during the follow-up because the user's existing app was running. |
| Current follow-up Rust tests | **77 passed** (65 logic + 12 hidden native); the earlier desktop receiver result is separate |
| Formatting / all-target Clippy with lint warnings denied | PASS; Cargo reported a cache hard-link fallback and copied cache files |
| Static-CRT AMD64 candidate build | PASS with installed Rust 1.98.1 (48a229cea) |
| Dependency advisories | PASS: OSV query completed for all 13 Cargo.lock registry packages, zero listed advisories; not a source-security audit or a guarantee of no vulnerabilities |
| ZIP integrity and allowlist | PASS: 40 member hashes, including the new mouse acceptance checklist; no personal settings/source/PDBs |
| Adversarial package checks | PASS: changed README, injected settings.json and traversal entry rejected |
| Workflow static check | PASS: YAML parses, action references use full commit SHAs, contents permission is read-only |
| Hosted Windows CI / branch-protection enforcement | Final hosted run fully PASS at `bda7d8cb193e01eb5cc6e8aaac2b694d201369b8`, including pinned Node 24 actions, 77 tests, packaged runtime and artifact retention. Main is confirmed unprotected (GitHub API); no policy change made. |
| PE manifest/resources | PASS: complete REA static inventory; AMD64 candidate identity 0.6.10.0, resources present; shell loading is a separate observation |

The pinned `rust-toolchain.toml` requests 1.98.1. This machine has the compiler under the installed `stable` alias, not a separately installed `1.98.1` alias. Local runs used process-scoped `RUSTUP_TOOLCHAIN=stable` after verifying the exact compiler version. No additional local toolchain was installed. The fresh hosted-run toolchain installation passed in the linked CI runs.

The OSV checker sends only public crate names/versions to the [documented querybatch API](https://google.github.io/osv.dev/post-v1-querybatch/), handles pagination, and fails on network/errors/incomplete results. Detailed evidence is in ignored local `source/target/dependency-audit.json` and `source/evidence/production-audit-*` files. A clean advisory result cannot establish runtime correctness.

## Release gates, in order

1. **Run and enforce CI on GitHub.** Obtain a fully green hosted run and configure required checks on main before release integration. The audit branch is pushed, and a local Git archive checkout in a path containing spaces passed validation and candidate packaging without ignored local configuration. Hosted execution is recorded below. No auto-release workflow is added here.
2. **Physical mouse/app matrix.** Test the owner's mouse on Windows 11 and representative mouse wheels for any broader device claims. Cover browsers with exclusions removed, an editor, File Explorer, a hosted Store app, horizontal/Shift behavior, elevated targets, target changes, sleep/resume and a long idle/scroll soak. Verify global smoothing, Super smooth flick/coast/braking, and that an excluded app receives normal input without a remaining smoothed tail. Record visible behavior, event totals, CPU, memory/handle trend and hook latency distribution. Test the actual packaged executable, not only the constructed fixture. Trackpad smoothing is outside the approved target; retain the existing fine-resolution/injected-input pass-through policy.
3. **Accessibility and native shell acceptance.** Keyboard-only navigation, Narrator/UI Automation names and state, focus visibility, high contrast, text scaling and mixed-DPI monitors; visible Snap flyout, maximize/restore, tray recovery after Explorer restart, and sign-in after moving the portable folder. [Microsoft accessibility expectations](https://learn.microsoft.com/en-us/windows/apps/design/accessibility/accessibility-overview) require evidence beyond painted screenshots and standard HWND classes.
4. **Publisher trust.** The existing 0.6.9 executable is verified unsigned. Provision an owner-controlled signing identity, sign/timestamp the release and verify the signature using the [documented signing tools](https://learn.microsoft.com/en-us/windows/win32/seccrypto/signtool). Certificates, trust services and secrets were not created or inspected here. Signing does not by itself guarantee SmartScreen reputation or compatibility.
5. **Recovery/compatibility scope.** Define supported apps and known pass-through limitations; test blocked writes/corrupt settings/recovery and a failed update/rollback. Add a user-facing backup/import workflow before broad settings migration. Hosted-app/source/transport additions need focused evidence before enabling them globally.
6. **Keep optional parity separate from release acceptance.** Automatic game detection, configurable hotkeys, reusable tuning profiles, monitor assignments, app-specific delivery modes, trackpad smoothing, zoom/edge gestures, themes/localization, an installer and a signed updater are optional backlog for the approved mouse-first product. Manual per-app exclusions are the chosen bypass mechanism. These missing reference features do not block this scope; the CI, mouse/app compatibility, accessibility, publisher-trust and recovery gates above still require evidence. Full reference parity is not claimed.

## Follow-up readiness checks

October 21 follow-up: formatting, Clippy, dependency advisories, package integrity and hidden native suites were rerun. Added two settings checks: a locked Windows destination reports failure while preserving the last good file and recovers after release; simultaneous writers leave a complete file from a successful writer. Windows can reject competing replacements with access denied; this test verifies integrity rather than promising every concurrent save succeeds. The app normally serializes settings edits on the UI thread.

`tools/Test-Candidate.ps1` adds a paused packaged-binary probe from a separate Unicode folder with spaces. It checks executable-relative save location, corrupt-file preservation and explicit recovery, failed preset-save rollback, a short idle resource sample and clean Quit. Its local run stopped before launching a copy because the user's root `ScrollMage.exe` was already running (PID 79824 at the check). That instance, its settings and sign-in registration were left untouched. The probe is included in hosted CI; its execution result must be recorded before claiming packaged runtime acceptance. The short idle sample is not a long-running resource soak.

GitHub Actions is enabled. The official Rust distribution endpoint returned HTTP 200 for the pinned 1.98.1 manifest checksum. [The first clean hosted run](https://github.com/Frozt-src/ScrollMage/actions/runs/39676821292) passed all steps at commit `fdb3766dc73dd125815029d019ddf46cc40b5420`: 77 Rust tests, advisory check, candidate build/verification, three packaged runtime scenarios and three adversarial package checks. Downloaded evidence is under ignored `source/target/hosted-readiness-first`.

The hosted packaged probe passed fresh startup/save in a Unicode path with spaces, preservation of corrupt bytes until an explicit recovery edit, blocked-save rollback and clean Quit. Its three-second paused idle sample recorded 0 ms added CPU and zero handle/GDI/USER deltas; this is a short sample, not a long-session performance claim. The local packaged executable has not been launched because the user's existing app is active; hosted results apply to the hosted-built executable, whose hash is recorded in its report.

The first hosted run exposed Node 20 deprecation warnings for the old pinned actions. Official releases and action metadata were inspected; checkout v7.0.1 and upload-artifact v7.0.2 use Node 24 and support the existing inputs. They remain pinned to full verified commit SHAs with read-only repository permission and disabled checkout credential persistence. [The final hosted run](https://github.com/Frozt-src/ScrollMage/actions/runs/39677503336) passed every step in 4m29s at `bda7d8cb193e01eb5cc6e8aaac2b694d201369b8`, including the updated actions and packaged acceptance document. Its three packaged scenarios passed; the short paused sample again recorded zero added CPU/handle/GDI/USER values. Downloaded final evidence is under ignored `source/target/hosted-readiness-final`. No main merge or release publication was performed; remote main remains the `v0.6.9` tag. Branch protection is a separate repository policy decision.

## Candidate and publication status

Candidate ZIP: `dist/ScrollMage-0.6.10-candidate-windows-x64.zip`.

- ZIP SHA-256: `2c08fc10470603f063edbe84f12277d92a049dda1c41cff0a0fc1353ec3964e9`.
- EXE SHA-256: `a24503a8b71aa1fff4a9fbdbb34d05233f6bb42691285efe4c8c53a6248564a5`.
- No personal settings are bundled. The candidate starts with normal defaults, including browser exclusions.
- Old 0.6.9 share ZIP SHA-256 remains `ca2adde128f90615e8ae86bdede62a617baa8dbeac0858413e15b9383fb488bf`.
- Audit changes and follow-up documentation/packaging are kept on the audit branch. Main and the published 0.6.9 release are unchanged. Do not label this candidate production-ready based solely on automated checks.
- The final CI outcome is recorded after the checked commit; the later release-preparation changes affect documentation and packaged release notes only.
- [Mouse acceptance and rollback checklist](MOUSE-ACCEPTANCE-0.7.0.md) is also included in the ZIP as `MOUSE ACCEPTANCE.md`.

## Authorized candidate publication

After the audit, the owner authorized merging the audit branch into main and publishing the updated candidate ZIP with its changelog. Release preparation uses tag `v0.6.10-candidate.1` and keeps the GitHub release marked prerelease. The dated candidate changelog is included in the refreshed ZIP. Historical statements above about unchanged main/publication describe the audit checkpoint, before this authorized integration. Physical acceptance, signing and required-check protection remain open; this publication is for testing.
