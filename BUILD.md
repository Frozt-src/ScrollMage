# Building Scroll Mage

Scroll Mage is a proprietary Windows desktop application. This repository is private. The LICENSE and third-party notices apply to the corresponding project files and components.

## Requirements

- 64-bit Windows.
- Rust with the `x86_64-pc-windows-msvc` target.
- Microsoft C++ build tools and a Windows SDK with the resource compiler.

The 0.6.7 distribution was built using Rust 1.98.1. The application icons, tray icons, and logo paths are checked in and embedded during compilation. Regenerating artwork is not required to build the app.

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
cargo test --locked --release --target x86_64-pc-windows-msvc --test layout --test settings --test branding --test engine --test motion --test spring_upgrade --test logo_lockup --test control_motion --test frost
cargo test --locked --release --target x86_64-pc-windows-msvc --bin scrollpilot hidden_native_ -- --nocapture --test-threads=1
```

The selected test suites cover 43 logic tests and 4 hidden native UI tests. Keep the `hidden_native_` filter for unattended runs. The remaining native fixture uses live desktop input.

## Distribution

Downloadable binaries belong in GitHub Releases. The 0.6.7 release ZIP contains the executable, icon, README, banner image, startup instructions, a portable paused launcher, and third-party license files.

The package excludes source code and personal settings. Preserve LICENSE, THIRD_PARTY_NOTICES.md, and vendor-licenses with redistributed copies. The current build is unsigned.
