+++
title = "the foot comes back after a reboot without being opened: a host a service can start"
created = 1788403054
updated = 1788659889
claimant = "Animations-AE"
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
**Filed from bl-8bd0** (DESIGN §18.3), which stated the limit rather than
hiding it: the pocketed foot is armed by `MainActivity.onResume`, so after a
reboot this device answers no tool call until somebody opens the app once. A
phone that reboots in a pocket is exactly the case the rung is for.

## Why the obvious fix is not the fix

A `BOOT_COMPLETED` receiver is LAWFUL — Android 15's restriction bars
`dataSync`, `camera`, `mediaPlayback`, `phoneCall`, `mediaProjection` and
`microphone` from a boot-started foreground service, and `specialUse` (the type
§18.3 chose, and chose partly for this) is on none of those lists.

It would also be useless as things stand. `dev.yog.Pocket` cannot CREATE a
lane: a service may start a process with no Activity in it, and this app's tool
bridges resolve their classes through handles android-activity fills on the way
to `android_main` (`src/shell/jvm.rs`). A host built from a service would be a
foot whose platform tools all refuse — an advertised set that is a decoy, which
is the shape §16.1's whole corpus refuses. `onStartCommand` returns
`START_NOT_STICKY` for the same reason, so a system-killed process does not
come back either.

## What this ball is actually about

Making a tool host startable with no Activity — which is a question about the
bridges, not about the service:

- what `ndk_context`'s globals hold in a service-started process, and whether
  the application object a service HAS is enough for
  `Bridge::open`'s class-loader walk;
- which tools genuinely need an Activity (§16.1 already says: `open` is
  platform-refused from the background, `camera` is OS-refused, a new location
  fix wants the separate background grant) versus which only need a Context;
- whether a foot whose advertised set is honest about that difference is a
  better answer than one that waits to be opened. §6's containment rule says
  the honesty goes in the descriptions a model reads — but §16.1's standing
  rule is that **the advertisement is static and whole**, so a set that varied
  by how the process started would be the two-tables defect.

That tension is the design work. It is bigger than bl-8bd0's rung and was
deliberately not answered inside it.

Cites: bl-8bd0, DESIGN §18.1/§18.3, §16.1, §6.