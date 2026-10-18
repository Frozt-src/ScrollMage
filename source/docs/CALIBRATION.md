# Calibration contract

Status: NOT RUN. No real Mac input/render traces or physical Windows mouse/trackpad trial were available during this upgrade. No near-1:1 or faster-than-another-app claim is made.

Use the same document geometry, viewport, zoom, scaling, device, scroll settings, and gesture sequence on each platform. Record the OS/app versions, refresh rate, device model, natural-direction setting, and whether the OS provides momentum. Test a Magic Mouse and a trackpad separately; do not pool their coefficients.

Capture input records with timestamps in seconds from a monotonic clock, fractional dx/dy, precision flag, phase, momentum phase, and direction-inversion flag. Capture visible-position samples separately with their presentation timestamps, viewport dimensions, content bounds, and offsets. An input-event timestamp alone is not proof of input-to-visible latency. Record the input and presentation clock alignment and units.

Suggested JSON Lines records:

```json
{"kind":"metadata","platform":"macOS","os_version":"record actual version","device":"record actual model","refresh_hz":120,"units":"logical pixels","os_momentum":true}
{"kind":"input","t":0.010,"dx":0.25,"dy":4.5,"phase":"changed","momentum_phase":"none","precise":true,"inverted":true}
{"kind":"position","t":0.0167,"x":0.25,"y":4.5,"viewport_width":600,"viewport_height":480,"min_y":0,"max_y":2400}
```

The rows above are illustrative, not measured data. Get actual event fields from NSEvent in an owned test window; retain separate observed scroll-view/render positions. A real Mac is needed to compile/run any AppKit capture implementation. Do not infer Apple's physics from GTK coefficients.

Collect ten short, medium, and fast gestures per device, both axes and diagonals, reversals, retouch during inertia, and upper/lower boundary collisions. Repeat under light and busy UI workloads. Preserve raw traces and clearly label warmup or invalid trials.

Fit wheel distance and omega against wheel input plus visible-position traces using the exact Spring update in src/motion.rs. Fit exponential friction only to a continuous post-release segment with known units and no active boundary. Fit elastic response separately at boundaries. Use held-out trials to compare RMSE in logical pixels, end travel, 50/90/98% response times, reversal delay, and first-visible-motion latency. Do not optimize total distance alone.

Keep fractional state in replay. For the system-wide adapter, compare the integer wheel stream as well as observed content movement; receiving apps may filter or ignore small deltas. The present implementation cannot force another app to expose or synchronize its rendering. A future direct-render adapter needs separate presentation instrumentation and supported native gesture-phase input.
