+++
title = "parity: export the seat's egui tree to the accessibility layer, act:<op> tags, coverage assertion over the bl-243b dumps"
created = 1788329844
updated = 1788397747
claimant = "Droidtags"
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
RULING (operator delegated the runtime choice 2026-09-01; decided by the coordinator): the preferred route is APPROVED — enable eframe's accesskit feature in the SHIPPED android build. The lockfile growth is sanctioned for this purpose. Rationale: the real tree is the one inventory (a self-reported dump is a second representation of the same fact and drifts from what users get), and a live accessibility tree is an operability win in itself — TalkBack reads it, matching the house everything-operable ruling. The debug-gated self-dump is now strictly a FALLBACK with a named exit: take it only if the android AccessKit adapter proves genuinely immature (record the specific failure in this ball), and file the exit ball to return to accesskit when upstream matures.

---

bl-243b has landed; what it leaves you (DESIGN §15).

Step 0's instrument gap is now CONFIRMED IN EVIDENCE, not just by survey: `make screens` captures a uiautomator dump beside every screenshot, and all six come back byte-identical — one android.view.View, no text. The dumps are in the run's own output, so the emptiness is a fact a future run re-checks rather than a claim in prose.

The walk therefore gates on a narrow FALLBACK, and it is deliberately not the inventory this ball wants. src/shell/app/probe.rs logs one line per change carrying exactly two things: the screen's name (written at the dispatch arm that chose it) and the mark's rect in device pixels — the mark is the only way into the configuration surface and carries no text, so a harness cannot otherwise find it. It carries no labels and no act: tags on purpose: logcat is device-wide and readable over the debug bridge, so a bar title or a row label written there publishes world state to the whole device. An act: inventory down this channel would breach that; the accesskit route does not, which is a second argument for the ruling already recorded here.

The two do not collide. When accesskit lands, the walk's assertions move to the real tree and the probe either shrinks to the screen name or goes; nothing in bl-243b assumes a control inventory exists.

What you inherit ready-made: the walk drives six screens with NO engine — a leaf minted per run makes the device a seat (Seat::open dials nothing), and the paint-first cache seeded from the vendored corpus supplies the rows, with the stored focus selecting which screen opens. That is the inventory instrument this ball says not to duplicate: add rows to the walk table, not a second harness.

---

FINDING — the accesskit route was walked end to end and failed on device, so the RULING's fallback condition is met. Recorded here as that ruling asks.

The preferred route is not one feature toggle, and the missing part is documented nowhere upstream: eframe's accesskit feature enables egui-winit's, which pulls accesskit_winit — but accesskit_winit compiles its android adapter ONLY under its own accesskit_android feature, an implicit optional-dependency feature that is in no default set and that neither eframe nor egui-winit forwards. Without naming accesskit_winit directly, eframe constructs the null adapter (whose update_if_active body is empty) and every frame's tree is built and dropped. Measured: with the feature on and the dependency unnamed, accesskit_android is not in the android graph at all.

Naming it works, and adds no Gradle work — the crate ships a prebuilt DEX and installs its own View.AccessibilityDelegate at runtime.

Then the app ABORTS. Raising its first accessibility event, accesskit_android event.rs:64 unwraps the JNI call getParent().requestSendAccessibilityEvent(..); under GameActivity's surface view that call returns a JavaException, and the unwrap is a SIGABRT. Measured, not inferred: a full emulator walk with the feature in place painted the roster, captured the dump, and the next step found a dead app — SIGABRT, backtrace through accesskit_android::event::send_completed_event, panic 'called Result::unwrap() on an Err value: JavaException'. The same three lines with the same unwrap stand in the newest accesskit_android release, so waiting a version does not help.

The dependency came back out for a reason bigger than the harness: an accessibility client is not only uiautomator, it is TalkBack. Shipping it would abort the app for exactly the users accessibility exists for, which is worse than exporting no tree at all.

So the ruled fallback is taken — PARITY §6's own words, a debug-gated in-process inventory — and the exit ball is filed as bl-a6f3, carrying the failure and the exact restore steps. §5's assertion is unchanged: only where the inventory bytes come from differs, and the driver reads both shapes with one scanner, so the changeover is a deletion rather than a second gate.
