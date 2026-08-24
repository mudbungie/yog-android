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
replies, dispatches gestures) — and, by the bl-ae9d ruling, a **tool host**
in direction: remote administration of the phone is a stated goal, with
tools invoked from the laptop running ON the phone and vice versa. The seat
lands first; hosting is the second act, not a maybe (§5).

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
- **Bootstrap rides existing trust, never the new device's own connection**
  (REMOTE §1.4, widened by bl-ae9d — §5). The app carries no enrollment,
  pairing, or account protocol reachable over its own unauthenticated
  connection, ever. What §1.4's "an act the operator performs on the boxes"
  now includes: an already-trusted device performing that act on the
  operator's behalf.

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

**O1 (ruled, bl-8d03): egui via `android-activity`'s NativeActivity backend,
no Gradle.** Spiked on real glass (a current-generation Pixel, Android 16-era
SDK 35) with a typing-heavy screen; the kill criterion — the soft keyboard —
did not kill it: the keyboard appears on field focus, IME-committed text
lands, Enter submits and dismisses, and the chat shape (a composer anchored
above the keyboard) works. The thin-Kotlin candidate loses on its own costs
(a second language, an FFI boundary, a Gradle toolchain) now that nothing
remains only Kotlin could buy. **No egui fork is needed**: every problem the
spike hit was dodged in app-side glue, none in egui itself.

Four findings the shell module must carry, each learned the hard way:

1. **Vulkan is not usable on Imagination-GPU Pixels.** The vendor driver
   segfaults inside `vkCreateGraphicsPipelines` while egui-wgpu builds its
   pipeline (tombstoned, reproducible). Force wgpu's GLES backend
   (`WGPU_BACKEND=gles` or the explicit `InstanceDescriptor`); revisit only
   with a device matrix in hand.
2. **SDK 35 is forced edge-to-edge, and the IME inset must be fetched by
   JNI.** `windowSoftInputMode=adjustResize` is ignored, and NativeActivity's
   `content_rect()` never tracks the keyboard — the working answer is
   `decorView.getRootWindowInsets().getInsets(WindowInsets.Type.ime()).bottom`
   over JNI (~60 lines). Two traps inside those lines:
   `AndroidApp::activity_as_ptr()` IS the Activity jobject (the
   `ndk_context` context is the Application — its `getWindow()` lookup
   throws), and every failed JNI lookup must `exception_clear()` or the next
   call is a CheckJNI abort.
3. **All insets are the shell's job**: content otherwise draws under the
   status bar, and a flush-bottom widget sits in the gesture-nav zone where
   taps never reach the app.
4. **The upstream era moved**: eframe 0.36 replaced `App::update(ctx)` with
   `App::ui(&mut Ui)`; yog sits on 0.29. The client tracks current eframe and
   does not wait for yog to catch up.

Named residual, deliberately narrow: **composition/preedit** (glide typing,
autocorrect-heavy input, selection handles) was not exercised by the
automated pass — a human pass on the installed spike covers it, and a failure
there reopens only the fork question, not this ruling. The shell stays a thin
paint-and-input layer over the tested core, excluded from coverage with its
reasoning in `tarpaulin.toml` when it lands.

## 4. Module map

One row per module, the same discipline as yog DESIGN §12: anything projected
≥200 lines is pre-split at design time.

| module | role | status |
|---|---|---|
| `src/frame.rs` (+ `frame/tests.rs`) | REMOTE §3 framing, bytes only | landed (bl-c747) |
| `src/codec/*` | strict decode of `Reply`, encode of `Act`/`Ask` | future |
| `src/transport/*` | rustls mTLS dial, one connection per gesture until upstream rules otherwise | future |
| `src/seat/*` | the view model: snapshots in, gestures out | future |
| shell | egui on android-activity/NativeActivity: insets, GLES, IME glue (O1, bl-8d03) | ruled, unbuilt |

## 5. The trust model and new-device bootstrap (bl-ae9d)

**The device mesh is a very-high-trust execution environment**, by explicit
operator ruling 2026-08-23. Every certificate is operator-grade within its
registrations (REMOTE §2's own words: "v1 has one human, and every
certificate is operator-grade"), and the consequence is stated rather than
implied: **one trusted device can bootstrap another.** A desktop client can
mint a new leaf and hand it to a phone; a laptop can drive tools on the
phone; the phone can drive tools on the laptop. Workspace registration
(REMOTE §1.5) remains the only partition inside the mesh.

**Three delivery channels for a new device's key material, all supported:**

1. **adb push** — the operator (or an agent the operator hands the cable to)
   places the leaf into the app's storage over a debug bridge. Zero new
   code; the channel is the physically attached, developer-authorized
   bridge.
2. **remote exec** — an already-registered client advertises tools that can
   write files on the new device (the remote-administration goal, pointed at
   provisioning). The trust carrying the cert is the existing authenticated
   tool route, not anything the new device asserts.
3. **QR** — a trusted client mints and *displays* the credential; the new
   device scans it with its camera. The trust is the operator's own eyes and
   hands: whoever can photograph that screen was shown it.

What all three share, and the line that must never move: **the new device
itself never enrolls over its own unauthenticated connection.** REMOTE §1.4
stands — there is no pairing protocol in the wire, no token exchange a
stranger on the network could initiate. Bootstrap is always an act performed
*through existing trust* (a cable, an authenticated route, a screen), and
the first thing the new device does with its material is an ordinary mTLS
dial like any other client.

**Upstream dependency, named:** minting from a seat requires the CA holder —
the engine — to expose a mint act. REMOTE §3 rules that a capability a
client needs is added to the boundary, never to the wire, so that is a
boundary `Action` in yog, tracked as a yog ball. Until it lands, minting
stays where it is today: `yog wire-certs` on the engine's own box, with adb
or remote exec carrying the result.

**What the phone durably holds** stays exactly §1's list: its key material,
and nothing else. A lost phone costs one certificate distrust at the CA
(REMOTE §4), never a history — the logs live on the engine.
