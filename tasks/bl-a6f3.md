+++
title = "return the parity inventory to the platform accessibility tree when AccessKit's android adapter stops unwrapping"
created = 1788397678
updated = 1788397678
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
The named exit from bl-fe4c's fallback (yog docs/PARITY.md §6, DESIGN §15.1).

The ruling for the parity gate preferred the real accessibility tree over a self-reported inventory, on two grounds that both still hold: the tree is what a user's screen reader gets, and an observed inventory cannot drift from what ships the way a self-report can. That route was walked end to end and it does not survive contact with a device.

WHAT HAPPENS, precisely. eframe's accesskit feature alone is a no-op on android — accesskit_winit compiles its android arm only under its own accesskit_android feature, which is an implicit optional-dependency feature in no default set and which neither eframe nor egui-winit forwards, so the fall-through is a null adapter whose update method has an empty body. Naming accesskit_winit directly with that feature does put the real adapter in the graph, and it does install its delegate with no Gradle work (the crate ships a prebuilt DEX). Then the first accessibility client to attach kills the app: raising the first event, accesskit_android's event.rs:64 unwraps the JNI call getParent().requestSendAccessibilityEvent(..), that call returns a JavaException under GameActivity's surface view, and the unwrap aborts the process. Measured on the emulator — SIGABRT, backtrace through accesskit_android::event::send_completed_event, and the screen walk's own verdict shows it: the roster paints, the dump attaches, the next step finds a dead app.

It is not a version behind: the same three lines with the same unwrap stand in the newest accesskit_android release.

The blast radius is why the dependency came back out rather than staying behind a debug flag. An accessibility client is not only the test harness — it is TalkBack. Shipping this would mean the app aborts for exactly the users accessibility exists for, which is worse than exporting no tree at all.

WHAT TO DO WHEN THIS IS ACTIONABLE. Upstream has to stop unwrapping there (or stop needing the parent for the event send). Watch accesskit_android's event.rs; when a release handles that failure instead of panicking, this ball is: restore the eframe accesskit feature and the accesskit_winit dependency with its accesskit_android feature (the manifest comment kept the exact spelling and the reasoning), delete the fallback inventory — the shell's file writer, its marker gate and the walk's pull step — and point tests/parity.rs at the dumps alone. The act: tags themselves do not move: they are the same tokens either way, and only where the inventory bytes come from differs, which is what PARITY §6 already says.

Worth filing upstream too, with the backtrace: the fix is small and the failure is total.