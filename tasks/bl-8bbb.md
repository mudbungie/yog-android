+++
title = "the composer is not a normal text field: typing with the cursor mid-text appends at the end, nothing is selectable, paste is unproven — make it a native Android EditText overlaid on the composer's rect"
created = 1789002185
updated = 1789002266
claimant = "Cantaloups-A10"
priority = 1
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
tags = ["usability-r3"]
+++
Operator report 2026-09-09: 'if you type a letter while the cursor is not at the end of the send box, it still goes to the end. Also can't select text. Just make it a normal text field.' Today the composer is an egui TextEdit fed by the GameActivity IME text mirror, which appends every commit at the end and offers no selection handles, no paste menu, no autocorrect. Ruling: the composer becomes a NATIVE Android EditText (a platform view in the activity's view hierarchy, overlaid at the composer's rect that egui lays out — position and size synced each frame through the existing JNI bridge; the field's text and cursor are the one home of the draft, read back into app state on change; send = the field's contents; egui paints nothing under it). This gives cursor placement, selection handles, copy/paste menu, autocorrect and the IME's own behaviour for free and closes bl-0df9 (paste). Keep the theme: the EditText styled from STYLE.md tokens (ground, ink, brand focus ring), the same height rules as the composer band (bl-0691 item 6). The kittest/screens walk must still type into it: drive it through adb  / the parity walk's existing typing door. Prove on the emulator: type mid-text, select with handles, paste, send.