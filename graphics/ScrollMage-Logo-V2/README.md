# Scroll Mage logo, second direction

Version 0.6.1, October 17, 2026.

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

The sidebar uses a 64-pixel logo control, with supporting copy and navigation repositioned beneath it. Historical scroll-only font/artwork files remain for provenance; they are not used to draw the current sidebar logo. This is a custom logo, not a complete installable font alphabet.

## Rebuild and verification

Run examples/export_lockup.rs, then tools/package_lockup.py. The PNG/GIF outputs encode the same pixels as the native app renderer. Motion data for the SVG also comes from that renderer's wheel function. tools/rebuild_logo_direction.py reconstructs the outline JSON from the preserved direction image; Pillow and numpy are authoring tools, not runtime requirements.

Tests sample 8001 points through the animation cycle for bounds, elastic stretch, loop continuity, and reduced-motion stability. Native-frame comparison verifies that all changed pixels lie in the new final l and that the removed reference opening does not remain as a second hole. The 80-frame packaging check verifies that the rest of the artwork is fixed. The editable SVG contains paths only, with no installed-font or raster dependency. Its audit warns about anchor complexity and a few reference-derived angles; geometry prioritizes fidelity to the supplied artwork.

Headless renders and tests establish geometry and code integration. They do not establish visible timer delivery in a running desktop session. The app was not opened or controlled during this logo pass.
