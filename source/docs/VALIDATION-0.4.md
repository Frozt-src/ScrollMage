# Version 0.4 branding validation

2026-10-17. Scroll Mage display branding and native animated wordmark.

PASS: 28 non-desktop tests, formatting, lint, release build, native Windows font loading, FreeType font loading, SVG XML, PNG counter transparency, and the four-second encoded GIF duration. The three new logo tests exercise the actual motion function used by the app.

The exact Windows vector renderer used by the native interface produced the static and animated previews. Live window playback was not tested in this pass; no desktop focus was taken. Full desktop interaction verification from the previous UI pass remains incomplete. The full alphabet, final Mage face, and final mouse icon are outside this pass because only the scroll lettering reference was supplied.

See BRANDING.md for asset coverage and integration details. The previous UI validation is retained as VALIDATION-0.3.md. Packaging results are under Outputs/ScrollMage-Verification.
