+++
title = "the wire moved to PROTOCOL 13 and this build still speaks 8: re-vendor, or every connection is refused at the preface"
created = 1788582208
updated = 1788582742
claimant = "Animations-S"
priority = 1
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
`src/hello.rs` pins `PROTOCOL = 8` (since bl-cc54) and the engine's `src/wire/hello.rs` is at **13**. The preface is fail-closed with no negotiation (REMOTE §3, quoted in `src/hello.rs`), so against a current engine this app does not degrade — it cannot open a connection at all, and the operator sees the mismatch sentence for every read and every act.

Measured by diffing the vendored `corpus/shapes.json` (protocol 8) against the engine's (13):

- **Six new shapes**, each owed a decision row: `reply/acknowledged`, `reply/login`, `request/login`, `request/login-tail`, `request/pin`, `request/unpin`.
- **Five changed signatures**, three of them in shapes this client READS:
  - `reply/ops` gained `failed`, `exit_label` and `standing` (REMOTE §9.17). `standing` is the engine's own five-word fold — `clean` / `detached` / `live` / `retired` / `acked` — and §9.17 is explicit that a client rendering the trail should read the words rather than classify `exit` itself. The trail surface built under bl-35bd paints the row's own facts and takes no classification, precisely so this re-vendor is where the alarm reading lands.
  - `reply/attention` gained `says` — the queue row's own sentence about why it fires.
  - `reply/transcript` gained `auth_row`, `wound` and `wound_reason`.
  - `reply/config` gained a typed `settings` array; `reply/steps` lost `auth_failed`.

The re-vendor is the same act bl-a433 and bl-cc54 performed at 5 and 8: regenerate `corpus/` from the engine (`make corpus` there), bump `src/hello.rs`'s constant with the moves recorded in its prose, add a decision row per new shape, and consume what the read shapes gained.

---

Two consumption hazards the re-vendor must decide, found while reading the engine's side (both are protocol 10's, REMOTE §14.1 + the engine's src/boundary/attend.rs):

1. **`attention` became follow-class at 10 — the same ask and the same reply shape, answered as a SEQUENCE by an intake that can hold.** The wire intake this seat dials holds it: the first frame at connect, a further frame whenever the answer changes, and a terminator when the hold ends — thirty seconds, the follow lane's own bound. This seat's read is one-shot (`seat::pass::answer` reads frames to EOF and takes the last), so against a protocol-10-or-later engine the standing pass would block for the whole hold on every cycle. The engine's own note says a seat built against 9 'would read the first frame and then wait on a terminator up to a hold away, which is a hang no sentence explains' — and that the strict-equality handshake is what converts that hang into the upgrade sentence. So the bump is not just a re-vendor: it is a decision about how this seat holds (or does not hold) the lane, and REMOTE §14.2's ladder is where the phone's half of it is written.
2. **`follow` is in the same class and predates it** (§5.5). Whatever answer #1 takes has to cover the live tail read too, which today is asked at a 500 ms rest.

Neither is visible from the corpus: frame COUNT is not a field signature, which is exactly why §14.1 spent a version on a wire spelling that did not move.
