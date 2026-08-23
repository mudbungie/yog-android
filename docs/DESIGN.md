# yog-android — Design

Status: founding stub (bl-c747). This file is the client's architecture
authority. The **server side's** authority is yog's `docs/REMOTE.md`, which
governs the wire, the trust model, and what a client may be — where this file
and REMOTE collide, REMOTE wins until REMOTE is amended. Amendments here
replace prose and cite the ball id; the path to a ruling is not narrated.

## 1. What this is

The **phone seat** from REMOTE §1's canonical scene: a home server runs the
yog engine and keeps every log; this client talks to conversations from a
phone. In REMOTE's nouns it is a **client** (a machine holding an
operator-issued certificate) acting as a **seat** (asks queries, paints
replies, dispatches gestures). It is not a tool host in v1.

What follows from that, and is invariant:

- **The engine owns the world.** This client holds no durable model state.
  Per-seat UI state (REMOTE §7) lives server-side in the seat's own
  `ui.json`/`pane.json` documents; what the phone durably holds is its key
  material and nothing else. Cache is cache: reconstructible, deletable,
  never authoritative.
- **The wire adds nothing to the boundary** (REMOTE §3). The protocol is
  `Act(Action) | Ask(Query)` in, `Reply` out, in the server codec's JSON
  serialization. A capability this client needs and the boundary lacks is
  added to the boundary, upstream in yog — never invented here.
- **The client is always the asker** (REMOTE §3, the routing ruling). No
  listening socket on the phone, ever; the seat polls.
- **Bootstrap is out-of-channel** (REMOTE §1.4). The operator provisions the
  phone's leaf certificate by an act performed on the boxes; this app carries
  no enrollment, pairing, or account protocol, ever.

## 2. The wire, mirrored

The framing is REMOTE §3's, byte-for-byte with the server's `src/wire/frame.rs`:
a big-endian `u32` length, then that many bytes of JSON; a request is one
frame; an answer is N ≥ 1 reply frames then a zero-length terminator frame;
16 MiB bound, refused on the header. Landed here as `src/frame.rs` (bytes
only — the JSON layer is the codec's, not the framing's).

The channel is mTLS, both ends authenticating with certificates. The client
side will be rustls with `default-features = false` and `ring` — `deny.toml`
already bans `aws-lc-sys`, `openssl-sys` and `native-tls` so the first
transport ball cannot get this wrong.

The reply codec is hand-written strict decode, mirroring the server's
hand-codec discipline (`src/boundary/codec.rs` there). `serde_json` enters as
the parse substrate when that ball lands, matching the server's own framing
dependency; serde *derive* stays a non-dependency, as upstream.

## 3. The stack ruling

**Rust, one crate** (this repo), same contained-Rust standard as yog. The
core — framing, codec, transport, the seat's view model — is plain host-
testable Rust under the 100% floor. That much is ruled.

**O1 (open): the Android shell mechanism.** Two candidates, deliberately not
yet chosen, because the table does not depend on the answer:

- egui via `android-activity`/eframe: one language, shares yog's paint
  idioms; its soft-keyboard/IME story on Android is the known risk, and the
  seat is a typing-heavy surface.
- a thin Kotlin activity over the Rust core through FFI: native input and
  lifecycle; costs a second language and an FFI boundary in a repo whose
  rules forbid casual `unsafe`.

Whichever lands, the shell is a thin paint-and-input layer over the tested
core, excluded from coverage the way yog excludes `src/shell/*` — the
exclusion is added with its reasoning in `tarpaulin.toml` when the shell
exists, not before. Deciding O1 is its own ball with a spike behind it, and
the loser's argument gets recorded here.

## 4. Module map

One row per module, the same discipline as yog DESIGN §12: anything projected
≥200 lines is pre-split at design time.

| module | role | status |
|---|---|---|
| `src/frame.rs` (+ `frame/tests.rs`) | REMOTE §3 framing, bytes only | landed (bl-c747) |
| `src/codec/*` | strict decode of `Reply`, encode of `Act`/`Ask` | future |
| `src/transport/*` | rustls mTLS dial, one connection per gesture until upstream rules otherwise | future |
| `src/seat/*` | the view model: snapshots in, gestures out | future |
| shell (O1) | paint and input only | future |
