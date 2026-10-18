# Scroll Mage brand composition

Version 0.5.2, 2026-10-17.

The sidebar combines the supplied right-facing mouse silhouette, the existing rounded lowercase scroll wordmark, and the supplied angular MAGE lettering. The mouse sits above the left side, facing across the word. Purple MAGE is smaller and right-aligned underneath. This is lettering artwork, not a new installable Mage font or a complete alphabet.

## Composition

The shared canvas is 460 by 440 units. The mouse occupies x=0, y=0, width=280, height=200. The scroll wordmark starts at y=174. MAGE occupies x=215, y=324, width=245, height=111.3, with its right edge at 460. The artwork is cropped to its alpha bounds when drawn, with transparent letter counters and mouse eye preserved.

The sidebar control is 154 logical pixels high and uses the available rail width. Version 0.5.2 exported checks cover 160 by 154 and 128 by 154. In the compact 0.5.3 frame, the default rail keeps the logo at 160 by 154 and the narrow rail uses 144 by 154. Supporting copy and navigation are positioned beneath the complete composition. The main content header remains free of a logo.

## Assets and provenance

- mage-original.png is a byte-for-byte copy of the supplied yellow MAGE artwork.
- mouse-original.png is a byte-for-byte copy of the supplied mouse PNG.
- mage-purple.png is the purple lettering version created with the image editor, using #8D24E8 as the target purple. The generated raster retains small tonal and edge variations. Adobe's connector initialization returned an internal error, so no Adobe editing or upload completed in this pass.
- lockup-transparent.png is the complete static composition with transparency.
- lockup-sidebar.png and lockup-sidebar-160.png / lockup-sidebar-128.png show the native renderer on the sidebar surface.
- lockup-elastic.gif is a four-second native-rendered animation preview.
- lockup-validation.json records dimensions, animation checks, and font scope.

The original graphics folder and Illustrator mouse master remain unchanged. The existing ScrollMage-Scroll.ttf is still the limited scroll lettering font; MAGE is supplied as artwork. The temporary tray icon remains separate from the sidebar composition.

## Native integration

src/brand_lockup.rs loads the two PNG assets once from bytes embedded in the executable. Their GDI+ images and source memory streams are retained together, then released before the renderer shuts down. No font installation, browser engine, GIF playback, or external asset path is required at runtime.

The existing vector scroll renderer and elastic pill function are unchanged. The exporter and app share the Lockup renderer. Mouse and MAGE remain fixed; only the final-l pill animates. Hidden/minimized timer shutdown and the Windows reduced-motion preference remain in place.

## Rebuild the previews

Run the release export_lockup example with a raw export folder, then run tools/package_lockup.py with that folder and assets/brand. The script encodes the unchanged native RGBA outputs as PNG/GIF and verifies transparency, a four-second loop, moving frames, and fixed mouse/MAGE artwork.

The asset checks do not confirm live desktop timer delivery or window painting. See VALIDATION.md for the current test and desktop boundaries.

Native memory-stream reference: https://learn.microsoft.com/en-us/windows/win32/api/shlwapi/nf-shlwapi-shcreatememstream
