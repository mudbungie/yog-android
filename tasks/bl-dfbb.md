+++
title = "the controls row gains an effort selector and a priority checkbox, gated by the capability the provider row states"
created = 1788321310
updated = 1788321310
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
**GATED on yog bl-23bd (yog's board): the two tuning gestures and the widened providers rows, at PROTOCOL 6.** Until that lands on an engine this app dials, there is nothing to send and no fact to gate on. First act here is the re-vendor: copy `corpus/` from the yog checkout at 6, raise `src/hello.rs::PROTOCOL` to match, and let the §14 cache's version stamp discard stored options (automatic).

**The controls (DESIGN §13.2 — "a new one is an entry here rather than a new place to look").**
- An **effort** selector (options `low / medium / high / off`, a fixed vocabulary — no wire read backs it) and a **priority** checkbox join the row under the composer, after the model selector. Tap is the act: a pick encodes the new op immediately (`{"op":"effort","workspace":…,"role":"worker","level":…}` / `{"op":"priority",…,"on":…}` — spellings from the vendored corpus, verbatim); a refusal lands in the same banner as every other.
- **Shown only when the selected provider row states the capability**: the widened `reply/providers` row carries `effort: bool` and `priority: bool` — the gate rides the wire row exactly as `blocked` greying does, never a client-side derivation (§8: a client re-deriving world state is inventing it). No provider picked this run → the controls are absent, same as the model selector's disabled state; what a control displays is what this device set, and it resets when focus leaves the workspace (join `picked_in`'s reset).
- Per-model rejection is NOT modeled: the engine's banner carries the provider's own refusal.

**Mechanics and traps (surveyed).**
- `src/shell/controls.rs` is tarpaulin-EXCLUDED — every show/hide decision and option list must live in covered code (`src/codec/pick.rs` row decode gains the two fields; a pure gating helper beside it), the `crate::rows` pattern.
- The row's width math splits post-`stops` space in two; four-plus controls at `mark::TOUCH` height in a phone width is a real layout decision — consider the checkbox as an icon-width toggle and the effort selector sharing the selector width class. §13.2's touch floor binds every target.
- `src/seat/model.rs` is at 289 and `src/seat/pass.rs` at 257 — two new `Cmd` variants + handle methods push model.rs over the 300 wall: plan the split along a real seam first, don't shave.
- Conformance: decision-table rows for `request/effort`, `request/priority`, and the widened `reply/providers` (`tests/conformance/requests.rs`, `replies.rs`) — a shape with no row is a red test.
- Doc amendments with the change: §13.2 bullet for the two controls, §13.4 parity-ledger row (in-wire once bl-23bd lands), and fix the stale "wire is at PROTOCOL 2" prose (DESIGN.md §2) against `hello.rs`'s changelog.

**Meaning, for the copy in the doc:** effort = how much reasoning the worker's model calls request (engine-side it is litany role config, switched at the next step — mid-conversation for free); priority = ask the provider's priority/fast-token lane; unchecked = the default lane. Cost-saver lanes (flex/batch) are out of scope.