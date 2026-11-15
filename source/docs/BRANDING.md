# Scroll Mage logo, second direction

Version 0.6.3, October 17, 2026.

The current mark follows the supplied horizontal design. The left-facing mouse and its tail form a continuous S; rounded c, r, o and l lettering continues to the right. Purple MAGE sits below the right side. The final l is rebuilt as a solid native vector stem, with one live elastic wheel cutout. The fixed opening in the supplied image is excluded from the outline data, so there is no second stationary pill underneath the animation.

## Construction and files

The native canvas is 1320 by 440 units. Two cached GDI+ paths hold the black and purple outline layers. The final stem is x=1161, y=18, width=49, height=256. The white or transparent wheel is centered at x=1185.5 and stays between y=34 and y=258. Its resting dimensions are 28 by 106, beginning at y=100. Motion stretches the opening and narrows it slightly; it never leaves the stem. The animation repeats every four seconds. Windows reduced-motion preference, hidden-window shutdown, and minimized-window shutdown remain in force.

The user-supplied reference is preserved byte for byte as assets/brand/logo-direction-v2.png. Its SHA-256 is a28631bcf4550d8b2f80bd08272802b37950011733c0fab4557cafcdf34de30b. Authoring reconstructs thresholded contours with at most 1.2 source pixels of simplification. The native logo uses embedded outline JSON, not a raster image, system font, web view, or GIF decoder. Purple is #8818FF; black is #000000. The original supplied art and prior mouse Illustrator package are preserved.

- ScrollMage-Master.svg: editable outline-only static logo, including the centered wheel counter. Open this SVG in Illustrator to edit its paths. No native .ai file is claimed.
- ScrollMage-Animated.svg: transparent animated SVG using samples from the native motion function. It includes a reduced-motion still variant.
- lockup-transparent.png: 1320 by 440 transparent PNG from the native renderer.
- lockup-elastic.gif: four-second opaque preview on the app sidebar surface, 990 by 330.
- lockup-motion-sheet.png: six native-rendered animation samples.
- lockup-sidebar*.png: previews at sidebar sizes, including 168 by 64 and 160 by 64.
- lockup-validation.json: export checks and source provenance.

The larger sidebar logo is removed; the title bar contains the only logo. Historical scroll-only font/artwork files remain for provenance; they are not used to draw the current title-bar logo. The scroll wordmark exporter and font builder that produced them were removed in 0.7.0, so those files are no longer regenerated. This is a custom logo, not a complete installable font alphabet.

## Rebuild and verification

Run examples/export_lockup.rs, then tools/package_lockup.py. The PNG/GIF outputs encode the same pixels as the native app renderer. Motion data for the SVG also comes from that renderer's wheel function. tools/rebuild_logo_direction.py reconstructs the outline JSON from the preserved direction image; Pillow and numpy are authoring tools, not runtime requirements.

Tests sample 8001 points through the animation cycle for bounds, elastic stretch, loop continuity, and reduced-motion stability. Native-frame comparison verifies that all changed pixels lie in the new final l and that the removed reference opening does not remain as a second hole. The 80-frame packaging check verifies that the rest of the artwork is fixed. The editable SVG contains paths only, with no installed-font or raster dependency. Its audit warns about anchor complexity and a few reference-derived angles; geometry prioritizes fidelity to the supplied artwork.

Headless renders and tests establish geometry and code integration. They do not establish visible timer delivery in a running desktop session. The app was not opened or controlled during this logo pass.

## Title bar placement

Version 0.6.2 added the native animated logo at x=20, y=4, width=120, height=40 logical pixels, within the 48-pixel integrated title bar. It replaces the drawn Scroll Mage caption text. The caption control paints against the frame background and passes hit testing through, preserving title-bar dragging and resize edges. The remaining title logo uses the existing animation timer and reduced-motion policy. The accessible window title stays Scroll Mage.

## Version 0.6.3 single-logo layout

Only the animated title-bar logo remains. The sidebar logo control and its repaint path are removed. Supporting sidebar text starts at y=76 and navigation at y=136, closing the former logo space. The title-bar logo keeps its existing scale, animation policy, and pass-through hit testing. Verification remains headless.


## 0.6.7 application icons, October 18, 2026

The icon uses the existing Illustrator mouse outlines, reversed to face left like the current title mark, in white on a gunmetal continuous squircle. The master source is assets/icons/ScrollMage-Icon.svg; the tray cut is ScrollMage-Tray.svg. The small cut enlarges the mark and omits the two far feet to preserve legibility. Corners are transparent. The existing animated title-bar lockup remains in place.

ScrollMage.ico contains 16, 20, 24, 28, 32, 40, 48, 64, 96, 128 and 256 pixel images. ScrollMage-tray.ico contains the first eight sizes. Small frames use 32-bit DIB with an AND mask; the 256 frame is PNG compressed. The executable embeds group resources 101 and 102 through app.rc and the installed Windows SDK rc.exe. No new libraries or runtime files are required. The generator tools/build_app_icons.py uses the existing authoring dependencies Pillow and numpy.

The UI owns separate large, small and tray HICON handles, loads exact metric sizes, refreshes them for DPI changes and taskbar recreation, and destroys replaced handles. Shared LoadImage handles are avoided because their cache may return a different requested size. The tray keeps the current status tooltip and menu.

Native hidden tests load both embedded resources at eight small sizes, verify transparent corners and visible white artwork, verify actual bitmap dimensions, and read back the window's big/small icon assignments at five DPI scales. Source files and icons are delivered under graphics/ScrollMage-AppIcon as well as source/assets/icons. The desktop shortcut targets ScrollMage.exe and has an explicit icon file. The running app is left untouched until the user reopens it.

Reference: [Windows icon resources and sizes](https://learn.microsoft.com/en-us/windows/win32/menurc/about-icons), [LoadImage ownership and sharing](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-loadimagew).
