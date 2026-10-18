# Scroll Mage

![Scroll Mage: smooth scrolling for Windows](assets/scroll-mage-banner.png)

**Make your mouse wheel feel right.**

I built Scroll Mage because I wanted smoother scrolling on Windows with settings I could actually dial in. Pick a preset, adjust the feel, and leave specific apps alone when their scrolling already works for you.

It lives in the tray, stays out of your way, and gives you a quick way to pause whenever you need it.

## What it does

- Smooths regular mouse wheel input in supported Windows apps.
- Lets you adjust scroll distance, spring response, and acceleration.
- Includes **Default**, **macOS style**, and **Super smooth** presets.
- Supports horizontal scrolling with **Shift + mouse wheel** and an optional reverse direction.
- Lets you exclude individual apps by their EXE name or full path.
- Gives you a built-in **Scroll Test** area for wheel input, dragging, and arrow keys.
- Can start when you sign in to Windows, if you enable that option.

## Get it running

1. Open **Scroll Mage** from the desktop shortcut or run `ScrollMage.exe`.
2. Choose a preset in **Scroll settings**. Start with **Default** if you want a baseline, or **Super smooth** for a gentler response.
3. Adjust the values and click **Apply settings** to activate your changes.
4. Try the result in **Scroll Test**, then in the apps you use.
5. If an app feels better with its own scrolling, add it to **Excluded apps**.

For exclusions, enter something like `game.exe` or paste the full EXE path, then click **Add** or press **Enter**. Adding or removing an exclusion saves it immediately. Other tuning changes and the sign-in option use **Apply settings**.

Use `Start Paused.lnk` when you want to open the app with smoothing paused. Normal launches start with smoothing enabled unless a startup problem prevents it.

## Keep control of it

| Action | What happens |
| --- | --- |
| **Alt + P** | Pause or resume smoothing. |
| **Pause smoothing / Enable smoothing** | Change the current smoothing state from the app. |
| **Close the window** | Keep Scroll Mage running in the tray. |
| **Open the app again** | Bring the existing window back. |
| **Quit from the tray menu** | Exit Scroll Mage completely. |

Run one scrolling utility at a time so they do not compete for the same input.

## Dial in the feel

**Distance** controls the scroll amount per wheel notch. **Spring response** controls how quickly the motion catches up. **Acceleration window** and **Maximum acceleration** control how repeated wheel input builds speed. A maximum acceleration of **1** turns acceleration off.

The **macOS style** preset is inspired by that feel. It is Scroll Mage's own tuning, not Apple's scrolling engine.

**Test momentum friction** and **Test elastic response** affect dragging inside Scroll Test. They do not add momentum or elastic edges to every Windows app. Each app still controls how it displays incoming scrolling, so the result can vary.

Browsers are excluded by default. Remove their entries if you want Scroll Mage to handle their wheel scrolling too.

## Local and straightforward

Scroll Mage runs locally. Your settings stay in `settings.json` beside the executable. There is no account, activation system, or built-in updater.

The app is built for 64-bit Windows. The current build is **0.6.7**.

## License

Scroll Mage is proprietary and closed source. All rights reserved.

See [LICENSE](LICENSE) for the project terms and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for the licenses and notices that apply to bundled third-party components.
