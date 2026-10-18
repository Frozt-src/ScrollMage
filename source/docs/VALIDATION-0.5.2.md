# Version 0.5.2 brand composition validation

2026-10-17. Sidebar mouse/scroll/MAGE composition. The plum app accent and home-only white/off-white Scroll Test are retained.

The shared native renderer successfully decoded the embedded PNGs, rendered transparent and sidebar-background compositions, and exported 80 frames at 20 samples per second. PNG previews were visually inspected at 216 by 207, 160 by 154, and 128 by 154. The mouse is above the left of scroll; purple MAGE is below and right-aligned. The native image and COM stream resources were released without an exporter error.

Asset checks confirm transparent background, containment within the 460 by 440 canvas, a four-second GIF, and fixed mouse/MAGE artwork across all 80 source frames. The GIF encoder produced 42 frames by merging unchanged frames. The elastic pill still moves.

Formatting, clippy with warnings denied, release build, and the existing 32 focused tests are the application checks for this pass. The original mouse and MAGE uploads are preserved with matching hashes. Settings, proprietary licensing, all third-party notices, scrolling engine, user graphics, and main content layout are preserved.

No desktop automation or focus changes were performed. Live window playback, actual desktop layout, animation-preference changes, and multi-monitor scaling were not rerun. Offline previews use the exact renderer called by the sidebar, but do not establish live Windows message/timer delivery. The prior desktop checks are in VALIDATION-0.5.md.

Adobe connector initialization failed with an internal error. Purple lettering was created with the image editor; no Adobe edit or upload completed. The supplied MAGE PNG is wordmark artwork, not an installable font. The main interface accents remain plum red; vivid purple is specific to the MAGE artwork.
