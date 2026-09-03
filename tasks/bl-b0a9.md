+++
title = "teleop rung 1b, the sighted pair: a camera still and a location fix, each behind its runtime permission"
created = 1788398990
updated = 1788400301
claimant = "Sightline"
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
bl-eac2 / DESIGN §16.1. Two built-in tools:

- `camera` — one still off the existing camera2 half (bl-d815's dev.yog.Camera; add a JPEG capture beside the Y-plane path). Answers a PATH, never image bytes — the screenshot precedent (DESIGN §6): a capture is text and encoding an image would add a shape to the boundary. CAMERA permission machinery already landed (per-tap ask, four-word vocabulary). Platform trap: camera access from a backgrounded process is refused by the OS — foreground-only at this rung; the pocketed-foot rung states the foregroundServiceType=camera cost if ever wanted.
- `location` — one fix, lat/lon/accuracy as text. ACCESS_FINE_LOCATION runtime permission via the same hook. Background location is a SEPARATE settings-trip permission and is NOT asked at this rung — while the foot serves foreground-only the fix is honest; the pocketed rung (see the foreground-service ball) revisits.

Refusals in band naming the grant. Probe both on device; host-test the table half. Serialize with the other teleop balls (src/tools.rs, android/).

---

What was proven here, and what only a real device can prove.

Proven without a device:

- Host tests over the whole pure half — the two advertised elements and their
  schemas (the lens enum is the same two words the argument reading accepts),
  the containment rule per tool (each description states its own permission,
  its foreground fact and, for the fix, that the age is the thing to read),
  the lens mis-call, where a still lands by default and that the default does
  not move between calls, and every arm of the dispatch through the
  no-Android refusal. 100% coverage floor held; `make check` green.
- The dex readback bl-f34f started, done again: `dev.yog.Sighted.camera` reads
  back from the shipped dex at `(Ljava/lang/String;Ljava/lang/String;)`
  `Ljava/lang/String;` and `location` at `()Ljava/lang/String;`, which is
  exactly what the door builds from the argument count. The paper four still
  read back unchanged after their bridge was moved onto the shared door.
- `make screens` fully green, and one beat richer: the four runtime
  permissions this corpus asks for are read back out of `dumpsys package` as
  declared, accepted and held. That is the granted end of the chain — the
  emulator installs with -g — and it is the one place a missing manifest line
  would be loud, since an undeclared permission is never refused at install,
  only never granted.

Left for a real device, and none of it is hand-waved away:

1. A photograph. That the three-frame burst comes back metered rather than
   black, that the sensor-orientation choice looks upright in the file, and
   that the byte count and dimensions in the sentence describe the picture a
   person sees. An emulated camera scene cannot answer the first two honestly.
2. A fix from a real receiver: an accuracy figure that means something, the
   new-fix line, and — the arm that matters — the last-known line after a wait
   where nothing was delivered. Also the Android 12 answer of "approximate" to
   a fine ask, which is why both grants are declared and either is accepted.
3. The permission dialogs themselves: raised once while the app is in front,
   and the settings sentence afterwards. Each tool asks on its own request
   code, so the fourth thing to see is that answering one never disturbs the
   enrollment scanner's own ask.
4. The two background refusals, which no emulator install grants around:
   nothing on screen means no camera at all, and no new fix.
5. The scanner-collision refusal: an invocation arriving while the scan screen
   holds the camera.

Every one of those needs an invocation to reach the platform, which is
bl-05b6's ball and unchanged by this one: there is still no engine, foot leaf
and /invoke path pointed at a device in this repo.
