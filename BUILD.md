# Building Scroll Mage

Scroll Mage is a proprietary Windows desktop application. This repository is private. The LICENSE and third-party notices apply to the corresponding project files and components.

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

From `source`, run:

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --release --target x86_64-pc-windows-msvc --test layout --test settings --test branding --test engine --test free_spin --test motion --test spring_upgrade --test logo_lockup --test control_motion --test frost --test diagnostics
cargo test --locked --release --target x86_64-pc-windows-msvc --bin scrollpilot hidden_native_ -- --nocapture --test-threads=1
```

The 0.6.9 selected test suites cover 61 logic tests and 11 hidden native UI tests. Keep the `hidden_native_` filter for unattended runs. The remaining native fixture uses live desktop input and verifies free-spin delivery plus click/pause braking. The source/review build is 0.6.9; the published 0.6.7 distribution remains unchanged.

## Distribution

Downloadable binaries belong in GitHub Releases. The 0.6.9 release ZIP contains the executable, icon, README, banner image, startup instructions, a portable paused launcher, third-party license files, and per-file checksums. Publish its ZIP checksum beside the archive; release notes and CHANGELOG.md describe the changes.

The package excludes source code and personal settings. Preserve LICENSE, THIRD_PARTY_NOTICES.md, and vendor-licenses with redistributed copies. The current build is unsigned.
