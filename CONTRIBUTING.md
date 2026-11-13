# Contributing to Scroll Mage

Thanks for helping. Bug reports, compatibility notes and pull requests are all welcome.

Scroll Mage is a scrolling extension for desktop mice. Touchpad support is out of scope, so please don't send changes that intercept or reshape touchpad scrolling.

## Reporting a problem

Open an issue and include:

- Scroll Mage version (shown in the sidebar and on the About page) and Windows build.
- Mouse model, and the app you were scrolling in.
- The selected preset, any app profile or exclusion involved, and the text shown on the **Activity** page.
- The exact steps, what you expected and what happened.

Scrolling feel depends on the receiving app, so the Activity text matters: it tells us whether input was bypassed, or was accepted and then rendered in steps by the app.

Please report security problems privately. See [SECURITY.md](SECURITY.md).

## Building and testing

You need 64-bit Windows, the Rust toolchain pinned in `rust-toolchain.toml`, the Microsoft C++ build tools and a Windows SDK. [BUILD.md](BUILD.md) has the details.

Before opening a pull request, run this from the repository root in PowerShell 7:

```powershell
./tools/Validate.ps1
```

It checks formatting, runs Clippy with warnings denied, and runs the logic and hidden native UI tests. CI runs the same script, then checks dependency advisories and builds and verifies a package.

`./tools/Validate.ps1 -DesktopInput` adds a test that briefly takes focus and moves the pointer. Run it only on a desktop you are not using for anything else.

## Pull requests

- Keep each change focused, and describe what you tested by hand. Automated tests cannot judge how scrolling feels.
- Add or update a test when you change the engine, settings or layout logic.
- Code that runs inside the low-level mouse hook must stay short and must never block: no file or registry access, and no waiting on locks. Use `try_lock` and pass the event through when a lock is busy.
- Dependencies are pinned to exact versions. If you add or update one, update `source/Cargo.lock`, add its license files under `source/vendor-licenses/`, and explain why it is needed.
- Add a line to [CHANGELOG.md](CHANGELOG.md) for anything a user would notice.

## Licensing

By contributing you agree that your code and documentation are released under the [MIT License](LICENSE), and any artwork under [CC BY 4.0](LICENSE-ARTWORK.md).
