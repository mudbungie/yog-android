+++
title = "teleop rung 1b, the sighted pair: a camera still and a location fix, each behind its runtime permission"
created = 1788398990
updated = 1788398990
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
bl-eac2 / DESIGN §16.1. Two built-in tools:

- `camera` — one still off the existing camera2 half (bl-d815's dev.yog.Camera; add a JPEG capture beside the Y-plane path). Answers a PATH, never image bytes — the screenshot precedent (DESIGN §6): a capture is text and encoding an image would add a shape to the boundary. CAMERA permission machinery already landed (per-tap ask, four-word vocabulary). Platform trap: camera access from a backgrounded process is refused by the OS — foreground-only at this rung; the pocketed-foot rung states the foregroundServiceType=camera cost if ever wanted.
- `location` — one fix, lat/lon/accuracy as text. ACCESS_FINE_LOCATION runtime permission via the same hook. Background location is a SEPARATE settings-trip permission and is NOT asked at this rung — while the foot serves foreground-only the fix is honest; the pocketed rung (see the foreground-service ball) revisits.

Refusals in band naming the grant. Probe both on device; host-test the table half. Serialize with the other teleop balls (src/tools.rs, android/).