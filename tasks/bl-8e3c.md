+++
title = "the wire moved to PROTOCOL 13 and this build still speaks 8: re-vendor, or every connection is refused at the preface"
created = 1788582208
updated = 1788582208
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