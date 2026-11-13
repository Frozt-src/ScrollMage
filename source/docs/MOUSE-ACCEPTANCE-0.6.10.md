# Mouse acceptance for the 0.6.10 candidate

Scope: Windows 11 x64, global mouse-wheel smoothing with per-app exclusions. Scroll Mage supports desktop mice only; it does not smooth precision-touchpad scrolling. Automated tests do not establish physical feel or visible compatibility.

## Prepare

1. Quit the existing app using **Quit Scroll Mage** in its tray menu. Closing the window only hides it. Do not run two copies together.
2. Extract the complete candidate ZIP into a separate writable folder. Keep your existing installation and settings as a rollback copy. Run the extracted `ScrollMage.exe`; the UI should say **Version 0.6.10**.
3. Open Activity and verify smoothing is enabled. Fresh candidate defaults smooth browsers; if your existing settings exclude a browser, remove it from Excluded apps before the browser tests. Existing user settings are not bundled.

## Record these results

| Check | Expected result | Your result |
| --- | --- | --- |
| Normal wheel in a browser, File Explorer and your usual editor | Activity reports accepted/emitted input for an eligible app; visible behavior is usable without jumps or missing input. Some apps accumulate small deltas into line steps. | |
| Slow notches in Super smooth | Precise scrolling without an unwanted long coast. | |
| Quick flick in Super smooth | Several rapid notches produce fast scrolling that continues and gradually slows. This does not physically unlock the mouse wheel. | |
| Brake a coast separately by reversal, primary click, auxiliary mouse button and Alt+P | Movement stops promptly; pause stays paused until resumed. | |
| Move the pointer to another app or another scroll pane during a coast | Old momentum does not scroll the new target. | |
| Add an app to Excluded apps and restart Scroll Mage | That app passes through normally; the exclusion survives restart and no old coast continues into it. Remove it and confirm smoothing resumes where supported. | |
| Shift + wheel / native sideways wheel if present | Horizontal scrolling follows the selected option; changing Shift during a coast does not turn an old tail into a new app command. | |
| Touchpad, if this PC has one | Two-finger scrolling looks and feels the same with Scroll Mage running as with it quit. If it differs, record the touchpad and driver: a driver that imitates a notched wheel cannot be told apart from a mouse. | |
| Ctrl/Alt + wheel and elevated apps | Modifier gestures remain native. An elevated target may be bypassed with an explanation rather than silently claiming delivery. | |
| Sleep/resume and an extended normal session | No stuck coast, lost input, persistent high CPU or growing memory/handle trend. Report the time and Activity text if input stops being accepted. | |
| Maximize hover, resize, mixed DPI and text scaling | Snap flyout appears; text and controls remain visible and reachable. | |
| Keyboard, Narrator and high contrast | Controls can be identified and operated; focus and state are understandable. High-contrast compatibility is still an open engineering/acceptance item, not certified by this checklist. | |
| Tray recovery and sign-in | Tray entry survives an Explorer restart when you next perform one. If you choose to enable sign-in, verify the candidate opens after sign-in; after moving the portable folder, the stale path should no longer appear enabled. | |

Record the mouse model, Windows build, app versions, preset, exclusions and exact failure steps with each issue. A screenshot of Activity helps distinguish bypass from accepted input that the receiving app renders in steps.

## Optional automated packaged check

After quitting Scroll Mage, run from the repository root in PowerShell 7:

```powershell
./tools/Test-Candidate.ps1 -Executable ./dist/ScrollMage-0.6.10-candidate-windows-x64/ScrollMage/ScrollMage.exe
```

The check launches only a copied candidate, paused, from a separate Unicode folder with spaces. It does not change your personal settings or sign-in registration. Evidence is saved in `source/target/candidate-smoke.json`. It does not replace the physical checks above.

## Roll back

Quit the candidate from its tray menu before reopening your previous copy. Keep its original settings with that copy. If you enabled candidate sign-in registration, disable it there before moving/removing the folder, then enable it only in the version you intend to use. This is a portable manual update; no automatic updater or installer rollback is claimed.
