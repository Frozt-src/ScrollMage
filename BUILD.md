# Building Scroll Mage

Scroll Mage is an open-source Windows desktop application. Code is under the MIT License in LICENSE, artwork under CC BY 4.0 in LICENSE-ARTWORK.md, and third-party components under the licenses listed in THIRD_PARTY_NOTICES.md.

## Requirements

- 64-bit Windows.
- Rust with the `x86_64-pc-windows-msvc` target.
- Microsoft C++ build tools and a Windows SDK with the resource compiler.

The 0.6.9 distribution was built using Rust 1.98.1. The application icons, tray icons, and logo paths are checked in and embedded during compilation. Regenerating artwork is not required to build the app.

## Release build

From the project root, run these commands in PowerShell:

```powershell
Set-Location source
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'
$env:RUSTFLAGS = '-C target-feature=+crt-static'
cargo build --locked --release --target x86_64-pc-windows-msvc
```

The result is `source/target/x86_64-pc-windows-msvc/release/scrollpilot.exe`. Use the distribution filename `ScrollMage.exe`. The crate and compiled output keep their existing internal name.

For a distributable build, remove local paths from compiler diagnostics by adding `--remap-path-prefix=<local-root>=/build` to `RUSTFLAGS` for each local root used during compilation. This changes embedded diagnostic paths without changing the source files.

## Validation

For the current local 0.7.0 candidate, run `./tools/Validate.ps1` from the repository root using PowerShell 7. Add `-Offline` to use cached dependencies, and `-DesktopInput` only on an interactive desktop where the owned-window receiver may briefly take focus. The script selects every integration test file automatically and runs hidden native tests separately. See [the production audit](source/docs/PRODUCTION-AUDIT-2026-10-21.md) for current evidence and pending release gates.

The root `rust-toolchain.toml` pins Rust 1.98.1. An already installed toolchain alias may be selected through process-scoped `RUSTUP_TOOLCHAIN` if its `rustc --version` matches that version. Do not change global Rust configuration to build this project.

From `source`, run:

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --release --target x86_64-pc-windows-msvc --test layout --test settings --test branding --test engine --test free_spin --test motion --test spring_upgrade --test logo_lockup --test control_motion --test frost --test diagnostics
cargo test --locked --release --target x86_64-pc-windows-msvc --bin scrollpilot hidden_native_ -- --nocapture --test-threads=1
```

The 0.7.0 candidate suites cover settings recovery and concurrent replacement as well as scrolling, layout and diagnostics. Keep the `hidden_native_` filter for unattended runs. The remaining native fixture uses live desktop input and verifies free-spin delivery plus click/pause braking. The published release remains 0.6.9.

## Distribution

Run `python ./tools/audit_dependencies.py` to check locked public crate versions against OSV (Python 3.11+, network required). Unknown/error results fail; the report is saved under `source/target`.

Run `./tools/Package.ps1` to build a separate unsigned candidate from the lockfile and verify its allowlisted contents, dependency inventory and checksums. It refuses to overwrite existing candidate artifacts. `./tools/Test-PackageVerifier.ps1 -Path <candidate.zip>` verifies rejection of tampering, private settings and traversal entries. Signing and physical-device acceptance remain separate release gates.

With existing Scroll Mage instances closed, run `./tools/Test-Candidate.ps1 -Executable <candidate/ScrollMage.exe>` on an interactive Windows desktop. It launches only a copied candidate, paused and hidden, from a separate Unicode folder with spaces. It checks startup, save location, corrupt-settings preservation/recovery, blocked-save rollback, idle resource deltas and clean shutdown. Fixtures and a JSON report remain under `source/target`; it does not change personal settings or sign-in registration. A three-second idle sample is not a long soak or physical scrolling test.

`.github/workflows/ci.yml` runs validation, advisory checks, candidate packaging and verifier checks on Windows with read-only repository permission. Desktop input tests are intentionally excluded from hosted CI. The workflow must be pushed and pass on a clean runner before claiming CI enforcement; repository branch protection is not configured by adding a YAML file.

Downloadable binaries belong in GitHub Releases. The 0.6.9 release ZIP contains the executable, icon, README, banner image, startup instructions, a portable paused launcher, third-party license files, and per-file checksums. Publish its ZIP checksum beside the archive; release notes and CHANGELOG.md describe the changes.

The package excludes source code and personal settings. Preserve LICENSE, LICENSE-ARTWORK.md, THIRD_PARTY_NOTICES.md, and vendor-licenses with redistributed copies. The current build is unsigned.
