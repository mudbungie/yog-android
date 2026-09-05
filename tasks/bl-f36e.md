+++
title = "the ball pane reads and cannot act: close, assign, release, create, update"
created = 1788584656
updated = 1788584662
priority = 3
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
The read half landed in bl-d587 (DESIGN §13.9): balls, workspace-balls and board are painted, one control apiece, each where its subject is. These five are what CHANGE the store, and until they land an operator can watch a board from the phone and cannot touch it.

The desktop twin settled the two questions this half turns on, and both transfer (lernie DESIGN §4.35, bl-f7ae):

- **The `--as` name is the WORKSPACE's, so this seat needs no identity.** yog spells the field as the ball's bound workspace name, never the operator's login name, and the binding of a ball to a workspace is that equality. A seat that invented an operator name would break the binding it was making. On the phone the name is the workspace the row came from — for workspace-balls that is the focused workspace, and for a board row it is the row's own claimant-to-be.
- **None of the five is fanned.** On the desktop that mattered because a window holds many channels; this seat holds one, so the fan question does not arise here — but the rule it produced does: an act is addressed at the row it hangs on, never at an aim taken separately.
- **`close` is armed and the other three are not.** The test is whether the act is undone by doing the other thing: a filing is undone by releasing or closing it, an amendment by writing the old words back, a release by an assign. `close` folds main into the worktree, squashes and removes it, and no verb reverses it — so it takes the arming this app already has one of (the trail's truncation, DESIGN §13.8): two taps on one control, spelled in the label, cleared by leaving the screen.

The exact frames, from the vendored corpus (protocol 13):

    {"op":"assign","project":P,"id":I,"name":N}
    {"op":"release","project":P,"id":I,"name":N}
    {"op":"close","project":P,"id":I,"name":N}
    {"op":"create","project":P,"name":N,"title":T[,"body":B]}
    {"op":"update","project":P,"id":I,"name":N[,"title":T][,"body":B][,"note":X]}

Two of the five are DOORS rather than rows: create's body and each of update's three may be ABSENT, and absence is a value — an empty string asks the engine to blank a field nobody touched. The encoder must skip an absent key, which is the same rule effort's string-or-null already keeps in this codec.

All five answer the captured-run reply this codec already reads: a refusal arrives in the engine's own words on the banner, and a success says nothing — the pane's own read after the act is what shows it landed.

What this ball should NOT try to build: the scheduling fields. That field is an array of objects (priority, tag, parent, needs) and each is a picker this pane does not have; the desktop refused the same and recorded it by count and reason. Say so in the codec's own decision rather than half-spelling the shape.

Closing this ball: grow the codec for exactly these five (tests/conformance/requests.rs rows move from Refuses to spelled), paint the controls with act: tags on the pane's own rows, extend the make screens walk where a new control appears, and delete the five remaining ball-pane lines from parity.toml.