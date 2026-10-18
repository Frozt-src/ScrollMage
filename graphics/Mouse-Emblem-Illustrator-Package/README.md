# Mouse emblem — Illustrator recreation

Rebuilt by hand in Adobe Illustrator 30.8.2 from the supplied mouse image. The contours are editable cubic Bezier paths; the source's stray pixels and distressed edge artifacts are omitted. This is a clean vector interpretation of the supplied silhouette.

## Files

- `Mouse-Emblem.ai`: editable Illustrator master, saved with PDF compatibility.
- `Mouse-Emblem-Light.svg` and `.png`: light artwork with a subtle shade difference on the far ear and feet. Transparent background. PNG is 2560 x 2560.
- `Mouse-Emblem-Ink.svg` and `.png`: single-color dark artwork. Transparent background. PNG is 2560 x 2560.
- `Mouse-Emblem-Preview.png`: charcoal presentation preview.
- `Mouse-Emblem-Face-Detail.png` and `Mouse-Emblem-Tail-Detail.png`: enlarged Illustrator renders.
- `Reference.png`: unchanged copy of the supplied image.
- `Illustrator-Verification.txt` and `Export-Verification.json`: saved-document and export checks.

## Editing hierarchy

1. **MOUSE / editable master**
   - Far anatomy: rear ear, front foot and hind foot as separate paths.
   - Tail: one closed contour forming the curled ribbon.
   - Body: near ear, body and pointed near feet, with a compound-path eye aperture.
2. **BACKGROUND / optional charcoal**: locked; hide for transparency.
3. **REFERENCE / embedded, hidden, locked**: aligned to the 1280 x 1280 artboard.

The eye is an actual hole in the compound path, so it stays transparent on any background. Production SVGs contain vector paths only; the embedded reference remains in the AI master. The tail deliberately tapers toward its rounded tip.

## Validation

The AI file was closed after saving, reopened from disk in Illustrator and read back. All six artwork paths are closed, with 52 anchors in total. Native Illustrator PNG exports were visually inspected at full view and in enlarged face/tail views. Pixel alpha checks confirm transparent background and eye aperture. The original upload and the copied reference have the same SHA-256 hash.
