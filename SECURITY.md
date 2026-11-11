# Security policy

## Supported versions

Security fixes are made on the latest release only.

## Reporting a vulnerability

Please do not open a public issue for a security problem. Use GitHub's private reporting instead: on the repository page, open **Security**, then **Report a vulnerability**. Include the Scroll Mage version, your Windows build and the steps to reproduce.

Please allow time for a fix to be released before sharing details publicly.

## What Scroll Mage does on your computer

Knowing the design helps you judge a report:

- It installs a low-level mouse hook (`WH_MOUSE_LL`) to read wheel events and mouse-button presses, and sends replacement wheel events with `SendInput`. It does not log keystrokes; it only checks whether Ctrl, Alt, Shift or the right mouse button is held.
- It runs without elevation (`asInvoker`) and does not smooth scrolling in apps that run at a higher integrity level than itself.
- It contains no networking code. There is no telemetry, account or updater.
- Settings are stored in `settings.json` beside the executable. Starting at sign-in, if you enable it, adds one value under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.

Release builds are currently unsigned, so Windows SmartScreen may warn before the first run. Check a download against the SHA-256 checksum published with the release.
