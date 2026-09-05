# yog-android — Design

Status: founding stub (bl-c747). This file is the client's architecture
authority. The **server side's** authority is yog's `docs/REMOTE.md`, which
governs the wire, the trust model, and what a client may be — where this file
and REMOTE collide, REMOTE wins until REMOTE is amended. Amendments here
replace prose and cite the ball id; the path to a ruling is not narrated.

## 1. What this is

**The app is yog** (operator ruling 2026-08-30, yog bl-15bd), and it ships all
three of REMOTE §12's runnable components — the seat, the foot and the server
— **each gated behind an explicit bootstrap rather than auto-started**. The
default path is mTLS client enrollment; running the engine on the phone is
allowed but is the deliberate, non-default choice. §9 is that ruling made
structural.

The **phone seat** is still the scene REMOTE §1 opens with, and still the
default: a home server runs the yog engine and keeps every log; this client
talks to conversations from a phone. In REMOTE's nouns it is a **client** (a
machine holding an operator-issued certificate) acting as a **seat** (asks
queries, paints replies, dispatches gestures) — and, by the bl-ae9d ruling, a
**tool host** in direction: remote administration of the phone is a stated
goal, with tools invoked from the laptop running ON the phone and vice versa.
Since REMOTE §4.2 minted the foot grade, hosting is not merely a second act —
it is a component with a certificate of its own (§9).

What follows from that, and is invariant:

- **The engine owns the world.** This client holds no durable model state.
  Per-seat UI state (REMOTE §7) lives server-side in the seat's own
  `ui.json`/`pane.json` documents; what the phone durably holds is its key
  material and **one cache** (§14, bl-de96), which is cache in the strict
  sense: reconstructible, deletable, never authoritative, and replaced by the
  next answer off the wire.
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

**The corpus and the spoken version move together, and the version is the
only thing that breaks an installed seat** (bl-e837). An unknown FIELD is
tolerated — this codec is strict about the fields it spells and says nothing
about extra ones, which `codec::conv`'s own test pins — so an engine that
grows a column does not break a seat that predates it. The §3 preface is what
breaks it, fail-closed and on purpose. So a protocol bump upstream is a
**re-vendor and a rebuild here**, not a compatibility shim: re-vendor
`corpus/` from a yog checkout at the new number, raise `hello::PROTOCOL` to
match, decode whatever the moved shapes gained, and let the §14 cache's own
version stamp discard what the previous build stored.

**Every connection opens with a version preface (bl-93e3).** REMOTE §3 is the
authority and this file cites rather than restates it: each end writes one
frame, `{"protocol": <integer>}`, before it reads the peer's; a mismatch is
fail-closed and the refusal names both versions and the remedy; a peer that
states no version is refused exactly as a peer of the wrong one. Landed here
as `src/hello.rs`, the mirror of the server's `src/wire/hello.rs`, with the
same sentence word for word — one rule said two ways is two rules. This seat
writes its preface and its request in one breath and confirms the engine's on
the way to the answer, so the check costs no round trip.

**The vocabulary is judged by the conformance corpus, not by fixtures written
here (bl-93e3).** REMOTE §3 ships `corpus/` — generated from the server's own
codec, never authored — and states what a client owes it: decode every frame
in both directories, round-trip what it emits, and record every skip as a
decision. The directory is **vendored** into this repo (there is no published
artifact and no endpoint that serves it) and replayed by
`tests/conformance/`. Three consequences are structural here:

- **The gesture codec gained a decode side** (`src/codec/request.rs`), which
  nothing in the app calls at runtime — this client is always the asker. It
  exists because an encoder alone can only be proved against a fixture
  somebody here wrote, while an encoder with an inverse is proved against the
  server's own bytes. That is what catches a field dropped on the way out.
- **A skip is a row with a reason**, exhaustive over the corpus in both
  directions: a shape with no row and a row with no shape are each a red test,
  so a vocabulary that grows upstream arrives as a question rather than as
  silence. A skipped shape must still be refused **naming itself**.
- **Rule 3 reaches inside an envelope, not only across shapes.** This codec
  spells one staging rung and predicts no conversation name (§8), so a frame
  stating another rung or a real seed is refused rather than flattened into
  the shape this codec has — the same misread the rule forbids, one level
  down.

The corpus caught one live defect on its first replay: firing a conversation
answers `{"kind": "started"}`, which this client had no arm for, so the one
gesture that makes a conversation reported a failure over a conversation that
was in fact running.

**The number lives in two files and the suite asserts they agree**
(bl-8553): `src/hello.rs` and `corpus/shapes.json`. So a corpus vendored from
a yog that has moved on, and a preface bumped without re-vendoring, are each a
red test rather than a skew discovered on a handshake. **What the standing
number IS is `hello::PROTOCOL`'s own changelog and is not restated here** —
this paragraph carried the integer once and it went stale across four bumps
while every sentence around it stayed true.

Two kinds of move stand behind it, and only one of them is mechanical:

- **The ledger catches a signature.** A field gained or withdrawn moves a
  shape's `since` and the drift check refuses it at the standing version —
  which is what forced 2 (the worktree lane's `subject_cwd`/`cwd`), 3 and 4
  (the roster's `failure`, the queue's `flag`), 5 (`reply/governing`'s
  rewrite) and 6 (the tuning pair and the providers row's capability
  booleans). A field this codec did not carry would be one dropped on the way
  out, which is what the request round trip is for.
- **The ledger cannot see a meaning.** REMOTE §5.5 made the follow lane's
  frame an **append** — *"absorb every frame of a read, in order, onto an
  empty fold"* — under a wire spelling that did not change. A signature ledger
  records field paths and types, so nothing forced a bump and no fixture can
  fail. A client of that lane must read the section rather than re-vendor the
  fixtures and call it consumed (§7 — this seat reads it one shot at a time,
  where the fold is assignment).

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
5. **An app's private storage is not executable** (API 29 onward, bl-7f12).
   A file this process writes cannot be `exec`'d at all, and the one exception
   is the app's own native library directory — whose contents are placed there
   at install time and therefore cannot be generated. It is recorded here
   rather than only where it bites (§10, where it is one of the two stops on
   running an engine on this device) because it bounds **anything** this app
   might ever write and then run, and the failure it produces reads as a
   permission problem an operator could grant. There is none to grant.

**The input mechanism is settled (bl-014e, closed with the spike's full
trap ledger in its comments).** The shell is GameActivity behind a minimal
Gradle shell, with a **two-way mirror bridge**: the GameTextInput buffer is
adopted into the focused field, the field is pushed back when they drift
(`set_text_input_state` is asynchronous — guard the echo window or a focus
change blanks a field), and `set_ime_editor_info` is set per field, because
the default `inputType` is TYPE_NULL and that alone puts the IME in degraded
key-event mode. Latency is wake-driven, not polled: game-activity's own
`onTextInputEvent` already wakes the looper on every commit — winit merely
declines to make a frame of it, and a three-line `TextEvent` arm (vendored
in the spike, upstreamable as-is) closes the gap. Measured on glass: ~80 ms
tap-to-glyph dominated by the injection harness, backspace repeat at ~52 ms
per char. Two upstream defects are carried until fixed there (both, plus the
null-buffer abort, tracked as bl-2958): winit's missing wake arm, and
games-activity 4.4.0 calling `restartInput()` on every key — which destroys
the connection Gboard's delete-repeat runs against — shimmed by a ~40-line
Java OnKeyListener in the Gradle shell. An IME action key is a known residual:
GameActivity writes the action where the enter key does not read it, so an
action declared on a field cannot fire. **No field declares one** (bl-6850):
the composer is `TYPE_TEXT_FLAG_MULTI_LINE` with `TextInputAction::None`, so
Android shows a return key rather than an action key and the enter key breaks
the line, which is what §8 says it does. The flag is load-bearing in both
halves — an IME not told a field is multi-line commits no newline into the
editor buffer the mirror adopts, so before it the key was inert rather than
merely un-sending.

**The input-wake question is ruled (bl-c761): focus-gated fast repaint, no
vendored winit.** This repo is registry-only with no exception standing, so
the wake arm cannot ship here until a winit release carries it. The shell
polls the GameTextInput buffer each frame and paces itself: 16 ms repaints
while a field holds focus, 250 ms idle, 8 ms only inside a push's echo
window (a push the shell made itself generates no wake). The one trap,
recorded because it silently reverted the fix once: the focus flag is read
AT the repaint decision from egui's settled memory, never remembered from
earlier in the frame. A winit release with the wake arm dissolves this poll;
consuming it is bl-2958's exit.

The shell is a thin paint-and-input layer over the tested core (bl-c761):
`src/shell/{sys,inset,bridge,app}.rs` are `cfg(target_os = "android")`,
excluded from coverage with their reasoning in `tarpaulin.toml`, and CI's
android leg is their compile check; the UTF-16 span math
(`src/shell/span.rs`) is host-tested under the 100% floor. `unsafe` is
confined to `src/shell/sys.rs` (`rules/unsafe-outside-sys.yml`), where the
soundness arguments are written. The Gradle shell lives in `android/` — no
wrapper jar (the leak gate refuses binaries, correctly); the system-gradle
requirement is documented in the Makefile `apk` target.

## 4. Module map

One row per module, the same discipline as yog DESIGN §12: anything projected
≥200 lines is pre-split at design time.

| module | role | status |
|---|---|---|
| `src/frame.rs` | REMOTE §3 framing, bytes only | landed (bl-c747) |
| `src/hello.rs` | REMOTE §3's version preface: state, confirm, and the one fail-closed sentence | landed (bl-93e3) |
| `src/codec/request.rs` | the gesture codec's decode side — the inverse the corpus is replayed through | landed (bl-93e3) |
| `corpus/` + `tests/conformance/` | the vendored wire conformance corpus and its replay: the decision table over every shape, in both directions | landed (bl-93e3) |
| `src/codec/start.rs` | the §8.1 start family: stage a conversation, fire it, and the prepared body carried whole between them | landed (bl-b64e) |
| `src/codec/follow.rs` | REMOTE §5.5's lane, read one shot at a time: the answer in flight as much of it as has landed | landed (bl-4822) |
| `src/codec/pick.rs` | the provider/model family: the two per-workspace reads and the pick that states an assignment whole | landed (bl-0267) |
| `src/seat/options.rs` | what the selectors offer AND what the workspace is set to, held as the engine's own envelopes and painted under the workspace they were read for | landed (bl-0267, bl-e9f9) |
| `src/shell/controls.rs` | android-only: the controls row under the composer — the conversation-level acts, one row | landed (bl-0267) |
| `src/codec.rs` + `codec/{fields,ws,conv,transcript,reply}` | the chat-loop slice: encode message/workspaces/conversations/transcript, strict decode of their replies; spellings pinned to the server byte for byte | landed (bl-fe33) |
| `src/codec/row.rs` | the row acts' three spellings and the `flagged` receipt (§13.5), plus the three readings the menu is built from — one roster, one home, shared by the codec, the seat and the paint | landed (bl-f97c) |
| `src/material.rs` | the seat's key material: three answers (off / half-provisioned named in full / provisioned) | landed (bl-48d9) |
| `src/tls.rs` | rustls client config, ring named never defaulted | landed (bl-48d9) |
| `src/transport.rs` | the Seat: one connection per ask, server name off the address | landed (bl-48d9) |
| `src/transport/wire.rs` | `Wire`, the four-class failure and its two predicates: which end failed, and whether the act was written and never answered (§19.1) | landed (bl-8641, third class bl-8bd0, fourth bl-07b1, split out bl-07b1) |
| `src/test_support.rs` + `test_support/serve.rs` | tests only: openssl-minted PKI; the one-shot and scripted multi-connection mTLS answering servers | landed (bl-48d9, split bl-5a98) |
| `src/rows.rs` + `rows/{build,compacted,project,project/blocks}.rs` | the transcript's one-line row projection: the row vocabulary (class, tone, role, fold), the per-entry match and its labels, the preview/body split — pure, no paint | landed (bl-0ed6) |
| `src/rows/turns.rs` + `turns/{steps,counts}.rs` | the turn rollup: where a turn is, when its machinery folds to one aggregate line, and the census that line says | landed (bl-0ed6) |
| `src/tools.rs` + `tools/{shell,files}.rs` | what this machine can run: the built-in table, its advertisement, and the dispatch | landed (bl-d366) |
| `src/tools/bridged.rs` | the two-line answer protocol every Java bridge speaks, and the one parser that reads it — pure | landed (bl-f34f, out of `tools/ui.rs`) |
| `src/foot.rs` | REMOTE §4.2's foot set as a type: the three gestures, and no way to reach a fourth | landed (bl-2040) |
| `src/host.rs` | the tool host's handle: what the frame holds, the standing it paints, the three-state health, and whether it is still a host | landed (bl-d366, split bl-8641, `alive` bl-8bd0) |
| `src/host/serve.rs` | the loop the worker runs: advertise, ride the follow read, run, complete, re-assert the set — refuse a carried `cwd` (§6), and the redial matrix (§18.5): the wire always, this device's own predecessor after one hold's width, every other refusal never | landed (bl-0ac8, bl-8641, bl-cc54, matrix bl-8bd0) |
| `src/state.rs` | the process's one live tool host (§18.1) — the crate's only lock, so a foot outlives the activity and a relaunch cannot build a second | landed (bl-8bd0) |
| `src/pocket.rs` | the pocketed foot's whole decision (§18): which devices hold their lane, and what the shade says in every state — pure, host-tested | landed (bl-8bd0) |
| `android/…/Pocket.java` | the foreground service that holds the process: the `specialUse` grant, the standing notification it is required to carry, and the two acts that end it | landed (bl-8bd0) |
| `src/tools/ui.rs` | the interface tools: their advertised elements and argument reading — pure | landed (bl-1511, protocol out bl-f34f) |
| `src/tools/ui/bridge.rs` | android-only: the JNI into the accessibility service, class resolved through this app's own loader | landed (bl-1511) |
| `android/…/{InterfaceService,UiTree,Gestures,Screens}.java` | the platform service: read the node tree, dispatch a tap, type, press a system control, screenshot | landed (bl-1511) |
| `src/tools/paper.rs` | the paper tools (§16.1 rung 1): `device`, `clipboard_set`, `notify`, `open` — their advertised elements, the price each states, and the argument reading | landed (bl-f34f) |
| `src/tools/paper/bridge.rs` | android-only: the JNI into `dev.yog.Paper`, one signature built per argument count | landed (bl-f34f) |
| `android/…/{Paper,Device,Notify,Open}.java` | the paper tools' platform half: the door, the three device reads, the notification grant and post, the typed intent | landed (bl-f34f) |
| `src/tools/sighted.rs` | the sighted pair (§16.1 rung 1b): `camera` and `location` — their advertised elements, the price each states, the lens reading and where a still lands | landed (bl-b0a9) |
| `src/tools/bridged/door.rs` | android-only: the call every bridge makes — a class of this app's resolved once, a static reached by name, the descriptor built from the argument count | landed (bl-b0a9, out of `tools/paper/bridge.rs`) |
| `src/tools/sighted/bridge.rs` | android-only: the two static calls into `dev.yog.Sighted` | landed (bl-b0a9) |
| `android/…/{Sighted,Still,Shot,Lens,Jpeg}.java` | the still's platform half: the door, the three gates (grant, foreground, the scanner holding the same camera), the camera2 burst, which lens and how big, and the frame on disk | landed (bl-b0a9) |
| `android/…/{Fix,Position}.java` | the fix's platform half: the two grants and the device switch, a bounded wait over every live provider — and what one fix says, age always included | landed (bl-b0a9) |
| `src/tools/shade.rs` | the shade read (§16.1 rung 2): `notifications` — its advertised element, the enable and the retention ruling it states, and the cap reading | landed (bl-5cbd) |
| `src/tools/shade/bridge.rs` | android-only: the one static call into `dev.yog.Shade` | landed (bl-5cbd) |
| `android/…/{Shade,ShadeService,Notice}.java` | the listener's platform half: the door, the service the operator enables, the two refusals it tells apart, and one notification as the lines a model reads | landed (bl-5cbd) |
| `android/…/Span.java` | how long ago, in the unit a reader acts on — one ladder, shared by a fix's age and a notification's | landed (bl-5cbd, out of `Position.java`) |
| `android/…/App.java` | the two handles a tool-host thread cannot get for itself: this app's context, and whether it is in front | landed (bl-f34f) |
| `src/seat.rs` + `seat/model.rs` + `seat/tests/{reads,deposit,start,grace}.rs` | the view model's handle: the commands the frame sends and the `Snapshot` it reads back | landed (bl-5a98, split bl-dfbb) |
| `src/seat/worker.rs` | the loop that spends them: one pass, one wait, and the live tick inside it | landed (bl-dfbb, out of `model.rs`) |
| `src/seat/pass.rs` | one pass of that loop: the standing questions, and what survives a pass the engine did not answer (§13.2's grace) | landed (bl-3202, out of `model.rs`) |
| `src/seat/acts.rs` | the acts the seat posts: the message deposit, the §8.1 start pair, the turn's stop and nudge, and the worker's tuning — none of them ever sent twice (§19.2) | landed (bl-de96, out of `pass.rs`) |
| `src/seat/acts/row.rs` | the three acts addressed to a conversation ROW rather than to the focus (§13.5), and the read that settles each in doubt — including the one that says out loud that none does | landed (bl-f97c) |
| `src/seat/asks.rs` | the reads a gesture asks for — the selectors' three and the live tail. Split from `acts.rs` on the contract's own line: an ask re-asks freely (§19.1) | landed (bl-0267, bl-e9f9, bl-4822, split out bl-07b1) |
| `src/seat/posted.rs` | what became of an act — took, refused, or in doubt — and the one wording of the lost-reply contract (§19.2) | landed (bl-07b1) |
| `src/shell.rs` + `shell/span.rs` | shell root + UTF-16 span math (the host-tested sliver) | landed (bl-c761) |
| `src/shell/place.rs` | the second host-tested sliver: which side of a control its list opens on and how tall it may be, so an opened popup lands inside the tappable area — pure, and the only half of §13.2's geometry a test can reach | landed (bl-78c2) |
| `src/shell/controls/drop.rs` | android-only: the drop-down that spends it — `Popup` over a button, because `ComboBox` places its list against the display | landed (bl-78c2) |
| `src/shell/{sys,inset,bridge}.rs` + `shell/app.rs` + `app/pass.rs` | android-only glue: the confined `unsafe` + entry, the JNI inset probe, the two-way IME mirror, what the shell IS and what one frame does with it | landed (bl-c761, split bl-dd7b) |
| `src/shell/screens.rs` | android-only: the screens by focus depth over the model's snapshot — the dispatch, the roster, the foot's standing, the banner and the one list-row helper every navigation list paints through | landed (bl-5a98) |
| `src/shell/screens/rows.rs` | android-only: the conversation list and the acts its rows carry (§13.5) — the long-press menu, its three items and the composer they spend, placed by `shell::place` like every other popup | landed (bl-f97c, out of `screens.rs`) |
| `src/shell/app/probe.rs` | android-only: the render-and-see probe (§15) — the screen this pass painted and where the mark went, said to logcat once per change | landed (bl-243b) |
| `scripts/screens.sh` + `scripts/screens-seed.sh` | the headless emulator loop (§15): boot, install, walk, capture, judge — and the two seeds (a minted leaf of either grade, a corpus-fed cache) that put the device on a screen without an engine | landed (bl-243b, the grade bl-8bd0) |
| `scripts/screens-platform.sh` + `scripts/screens-background.sh` | what the platform granted and bound, and — split from it because these beats MOVE the device — the two background lanes: the scheduled fetch (§17) and the pocketed foot (§18) | landed (bl-b0a9, bl-fcc5, bl-5cbd, split bl-8bd0) |
| `src/shell/back.rs` | android-only: the platform back gesture — the read, and the leave when no depth took it | landed (bl-550e) |
| `src/shell/mark.rs` | android-only: the yog mark control — the walk said in egui's primitives, toggling the configuration surface | landed (bl-387f, drawn mark bl-ff27) |
| `src/icon.rs` + `icon/arc.rs` | the application mark's generation walk, ported from the yog crate: compass-work arcs, the flat shape list, the hue drive — pure, host-tested | landed (bl-ff27) |
| `src/icon/drawable.rs` | the same walk emitted as Android `VectorDrawable` XML — the launcher icon as a derivation, pinned byte-for-byte against the committed assets | landed (bl-0b31) |
| `android/…/res/drawable/ic_launcher_{foreground,background}.xml` + `res/mipmap-anydpi-v26/ic_launcher.xml` | the generated layers and the five lines of adaptive-icon wiring that name them | landed (bl-0b31) |
| `src/shell/chat.rs` | android-only: painting one projected row — the stripe, the toggle, the two-line speaking shape, and the live fold under them | landed (bl-0ed6, bl-4822) |
| `src/shell/composer.rs` | android-only: the composer row — the field's band and presence, and the send that is THE send | landed (bl-9196, split bl-4822) |
| `src/roster.rs` | the conversation list's two readings of the carried stamp: newest-first order, and how long ago each row says it is — pure, host-tested | landed (bl-e837) |
| `src/live.rs` | the streaming tail's one rule: the lane's fold replaces the transcript's own tail, and at rest there is none — pure, host-tested | landed (bl-e3d1) |
| `src/outbox.rs` | the local echo and every decision about it: has this message come back in a transcript read yet, and which of the three fates it stands in (§19.2) — pure, host-tested | landed (bl-66fb, the echo itself bl-07b1) |
| `src/cache.rs` | the paint-first cache (§14): the last answered pass, stored as the engine's own envelopes and re-decoded by the one decoder | landed (bl-de96) |
| `src/bootstrap.rs` | which component this device is, derived from the leaf on disk | landed (bl-7714) |
| `src/bootstrap/offer.rs` | the three bootstraps as branded choices — Lernie / Thrall / Yog — and DESIGN §5's delivery channels | landed (bl-0d3c) |
| `src/leaf.rs` | the DER walk over this device's own leaf: its client name and its REMOTE §4.2 grade | landed (bl-7714) |
| `src/shell/boot.rs` | android-only: the bootstrap gate — read the standing, start exactly that component, start nothing otherwise | landed (bl-7714) |
| `src/shell/enroll.rs` | android-only: the configuration surface — the three branded choices, and the screen behind each tap | landed (bl-0d3c, generalized bl-387f) |
| `src/shell/enroll/material.rs` | android-only: the enrollment screen — the file list, the delivery channels, the pasted envelope and the re-read | landed (bl-dd7b) |
| `src/envelope.rs` | the enroll envelope a seat mints: read, checked against the leaf's own grade and name, landed under `material`'s names | landed (bl-dd7b) |
| `src/scan.rs` | the QR decoder: a camera luminance frame in, the envelope's text out, plus the camera bridge's four-word vocabulary — pure, host-tested | landed (bl-d815) |
| `tests/fixtures/enroll-v33m-{symbol.txt,payload.json}` | one foreign-encoded symbol at REMOTE §8.4's own bar (1567 bytes, version 33, level M, 149×149) and the bytes it carries | landed (bl-d815) |
| `src/shell/jvm.rs` | android-only: the crate's ONE JNI plumbing — attach, this app's class loader, the pending-exception discipline | landed (bl-d815, out of `tools/ui/bridge.rs`) |
| `src/shell/camera.rs` | android-only: the five static calls into `dev.yog.Camera`, activity passed in | landed (bl-d815) |
| `src/shell/enroll/scan.rs` | android-only: the scan screen — ask, preview, throttle, decode, and the way back to the paste field | landed (bl-d815) |
| `android/…/{Camera,Session,Frames}.java` | the camera2 half: the permission, the device session, and the Y plane as bytes | landed (bl-d815) |
| `android/` | the minimal Gradle shell: manifest (INTERNET, CAMERA, ACCESS_NETWORK_STATE, POST_NOTIFICATIONS, ACCESS_FINE/COARSE_LOCATION, RECEIVE_BOOT_COMPLETED, FOREGROUND_SERVICE + FOREGROUND_SERVICE_SPECIAL_USE), games-activity trio, the OnKeyListener backspace shim, the permission-result hook routed on four request codes, the lifecycle hand-off to `App`, and the two lanes armed on resume | landed (bl-c761, bl-d815, bl-f34f, bl-b0a9, bl-fcc5, bl-8bd0) |

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

## 6. Tool hosting, and the one deviation from REMOTE §5.2 (bl-d366)

The phone is a **tool host** as well as a seat: it presents what it can run,
rides the `invocations` follow-class read for its next work, runs it, and
posts the capture with `complete` (REMOTE §5, §5.3). The loop is the server's
own `src/wire/host.rs` mirrored — one loop, three gestures, all of them
ordinary boundary verbs, the engine never speaking first — and it rides its
own connection beside the seat model's, on the same material and therefore
the same certificate common name, which §5's refcounted presence map already
expects of one client.

**The surface is a type, not a convention (bl-2040).** REMOTE §4.2 minted the
foot grade after this loop landed, and states it as a closed set: *"the
tool-host gestures and **nothing else**: `advertise`, `invocations` and
`complete`. No other `Query`, no other `Action`."* `src/foot.rs` is that
sentence in the type system — it owns the transport and never hands it out, so
the general encode-any-gesture door is not reachable from the host loop and a
fourth verb is a compile error rather than a refusal at the far end. It is not
*enforcement*: the engine enforces the grade with one raise at its chokepoint,
in band, naming it. What the narrowing buys is that a foot-graded phone cannot
accidentally spend a gesture its own certificate refuses.

**A seat's tool host rides the same surface**, and that is not a
contradiction: the tool-host gestures are the tool-host gestures whatever
grade the leaf carries. One code path, so the seat-with-a-host case and the
foot-only case are the same case.

**The host redials a broken channel and stops on a refusal** (bl-8641,
reversing the founding ruling). The first ruling was that a channel which
fails stops the host with the sentence that stopped it, on the argument that
reconnect policy is a statement about how a device is supervised and a thread
that silently redialled forever would hide a broken seat from the operator
holding the phone. That argument was written for a box that sits still. This
is a phone: it changes networks hourly, and one `receive: Software caused
connection abort` on a wifi transition left the host dead until the app was
restarted — the tool host of a device whose whole point is being carried
around. **What made silent redialling wrong was the silence, not the redial.**
So the standing line is a three-state fact — serving, reconnecting, stopped —
and a host climbing back says `tools: reconnecting…` with the sentence that
broke the channel, for as long as it is climbing.

The ladder doubles from one second to thirty and stays there, forever, with no
attempt count: a device that changes networks hourly has no number of failures
after which giving up is the right answer, and thirty seconds is both fast
enough to be back before the operator looks and slow enough not to be a spin.
A channel that got as far as being accepted starts the ladder over, because
the dial that just worked is not history the next one answers for. The
presentation goes with the connection that carried it, so a redial presents
again — which is why `advertised` is a fact about the channel that is up now.

**The set is re-asserted at the end of every hand-off, and a re-assertion
that WROTE is said out loud** (bl-cc54, following thrall bl-2d78 and yog
bl-66d4). Both of REMOTE §5.1's guards over the advertised set stand on this
client holding a *parked read* — a second follow-class read under one identity
is refused, and an advertisement that would change the set in force is refused
while a read is parked. This loop is serial, so for the whole runtime of a
tool the device holds no read at all and neither guard covers it: a second
connection bearing this device's certificate may replace the set in that
window. Presenting again after each completion bounds the exposure to one
tool's runtime instead of forever, and it costs an idle host nothing — no
hand-off, no gesture.

**Knowing is the other half, and it is what PROTOCOL 8 bought.** The receipt
carries `wrote` — false when the engine found the stored set identical and
compared, true when it changed the document — so a re-assertion that wrote is
this device being told the set it offers was not the set in force. The
restoration is automatic; being told is not. It is counted onto the standing
(`restored`) and painted beside the tools line in the words of
`host::RESTORED`, which name both readings an operator can act on — another
connection bearing this device's identity, or an engine that lost the set —
because the device cannot tell them apart and guessing would be worse than
saying so. **A `true` on a channel's FIRST presentation says nothing** and is
discarded: every fresh channel, a redial's included, presents into whatever
the engine happens to hold, and the ordinary first presentation writes. Only a
presentation made after work this device just did can tell a rival from a
beginning. The count does not clear on a redial — unlike `advertised` it is
not a fact about the connection that is up, it is something that happened to
this device.

**What is NOT redialled is everything the wire refused.** `crate::transport::Wire`
draws the line where the code already knows it, at the socket: a connection
that would not open, a write that did not land, a read that died are the
channel; the engine declining the advertisement, a completion posted into no
slot, an answer of the wrong kind, a reply that will not decode and a version
that cannot be spoken to are answers, and none of them changes for being asked
again. A host that redialled a refusal would earn one refusal per pass forever
and put a wall of sentences where a stop belongs. **One window is on the wrong
side of that line and stays there**: REMOTE §3 rules that a peer which hung up
mid-preface is refused exactly as a peer of the wrong version, so a channel
that dies inside the preface's own round trip stops the host rather than
redialling. Narrowing it would be a client amending REMOTE, which §1 forbids;
the window is one connect away from a dial that just succeeded, and the ladder
covers everything after it.

**The deviation, and why it is lawful.** REMOTE §5.2 derives a host's
advertisement from an operator-authored `<yog-data-root>/tools.json` naming an
argv per tool, and spawns that argv. This client's deliverable is an APK:
there is no operator-authored argv on a phone and nowhere to install one, and
a config file naming executables that do not exist on the device would be a
fiction the advertisement then published. So **the table is built into the
app** (`src/tools.rs`) — the same three advertised facts per tool
(REMOTE §5.1: name, description, and the JSON Schema verbatim), projected from
that table, and dispatch to a Rust function rather than a spawn. Nothing the
wire sees changes, which is the test of whether a deviation is lawful; the
capture stays REMOTE §5.3's three facts, so every tool answers in stdout,
stderr and an exit code and nothing downstream carries a second shape.

**What is offered is bounded by what an app uid can actually do**, established
by probe on a current device rather than assumed: a shell (`sh -c`, under this
machine's own 60-second bound, terminated and reported as the shell's own
`timeout` verdict), and file read / write / list.

**The paper tools are the same sentence answered four more times** (§16.1,
bl-f34f): `device`, `clipboard_set`, `notify` and `open` need no platform
service at all, so what bounds each is one OS rule, and each rule was read
before the tool was advertised rather than after. The clipboard WRITE is
allowed with no focus and no permission — AOSP's own
`ClipboardService.clipboardAccessAllowed` reaches its focused-window and
default-IME tests only under `OP_READ_CLIPBOARD`, while its
`OP_WRITE_CLIPBOARD` arm is *"Writing is allowed without focus"*, unchanged in
every branch from android10 to main — which is why there is a
`clipboard_set` and no clipboard read (§16.1's refused shapes). `notify` meets
the API 33+ runtime grant and `open` meets the API 29+ background-launch
refusal, and both ask BEFORE they act rather than reporting success for a call
the platform silently dropped: a background `startActivity` throws nothing and
logs one line, which is exactly the shape that would make a tool a decoy.

**The sighted pair is the same sentence answered twice more, one rung up**
(§16.1, bl-b0a9): `camera` and `location` cost a runtime grant, so what bounds
each is an OS permission the operator holds and can revoke, plus a foreground
fact the platform enforces itself. Each asks before it acts and refuses in band
naming the one act that lifts the refusal, and each states that price in the
description a model reads. `camera` answers a **path** for `screenshot`'s
reason, and `location` answers a fix's **age** beside its accuracy, because a
position with no age is the one shape of this pair that could mislead while
looking like an answer.

**The shade read costs a service the operator enables, and keeps nothing**
(§16.1 rung 2, bl-5cbd): `notifications` reads what this phone is currently
showing — app, age, title, text — through a `NotificationListenerService`
whose enable is a settings act rather than a permission an app may ask for.
That shape is the point: it answers the read want `READ_SMS` would otherwise
be asked for, at one act the operator can revoke, where the SMS pair is
hard-restricted and its send half has no undo. Three rulings ride with it and
each is written where it is met. **The grant is all-or-nothing** — a listener
sees every notification on the device or none — so there is no per-app filter
in this app, which would advertise a narrowing the OS does not enforce and be
§16.1's refused per-tool toggle screen wearing another hat. **Nothing is
retained**: the service overrides neither the posted nor the removed callback,
holds no history, writes no file and logs nothing (logcat is device-wide), so
every answer is the platform's own `getActiveNotifications` at the moment of
the call — and the cost of that, a dismissed notification being gone and an
unwatched moment being unanswerable, is stated in the description rather than
papered over with a buffer. And **it is read-only**: a bound listener may also
dismiss a notification and fire its buttons, and neither is built at this rung.

**The interface tools need a platform service, and enabling it is the
operator's act** (bl-1511, and §5's trust model unchanged): an app uid cannot
screenshot — `screencap` wants a signature-level permission — and cannot see
another app's views at all. An `AccessibilityService` carries reading, gesture
dispatch and screenshots in one place, and the app never grants itself
anything: the operator turns it on in system settings, or a trusted device
does it over the physically attached debug bridge, the same channel that
carries this seat's key material. **Sideloaded apps meet a second gate** —
Android's restricted-settings block silently reverts the enable — and lifting
it is one more act on that same cable (`appops set … ACCESS_RESTRICTED_SETTINGS
allow`), recorded here because the failure presents as a setting that will not
stick rather than as a refusal.

**An operator who upgrades rather than installs fresh meets both again**
(bl-7f12): the service is enabled per class name, and the class moved with the
rename (§9). Today that is moot — the `applicationId` moved too, so every
install is a fresh one — which is exactly why it is written down now, before it
stops being moot.

The five are `ui_read` (the node tree in front, as text — the form a model can
act on), `ui_tap` (a coordinate, or the first clickable node matching text),
`ui_type`, `ui_key` (back / home / recents / notifications / quick-settings)
and `screenshot`. **A screenshot answers with a path, never the image**: a
capture is text (REMOTE §5.3), and encoding one would be this client adding a
shape to the boundary. They are advertised whether or not the service is on,
because an advertisement is a fact about what this machine offers and being
able to act is a fact about right now — two tables would put a right-now fact
into a durable document, which is the defect §5 was amended to remove; a
disabled service refuses in band with the sentence that names the fix.

**One JNI trap, paid for once:** a thread the JVM did not create resolves
class names against the SYSTEM class loader, which knows nothing an APK
shipped — so `FindClass` from the tool-host worker throws
`ClassNotFoundException` for this app's own class. The application object's
own loader is what resolves it, and one global reference to the result
outlives the attach that found it (`src/tools/ui/bridge.rs`). The honest limits are written into the descriptions a model
reads — REMOTE §5's containment paragraph in this client's own terms: what
runs here runs as this app's user, and the design does not claim otherwise.

**The worktree lane ends here, in a refusal (bl-0ac8).** PROTOCOL 2 added
REMOTE §5.4's lane: an advertised entry may carry `"subject_cwd": true`, and
the engine then routes a bare granted name to the one client of the workspace
that both advertises it and consents, putting the conversation's resolved
working directory on the invocation as `cwd`. §5.1 puts enforcement on the
advertiser in as many words — *"it stays checkable because the box that stated
it is the box that enforces it (thrall refuses a carried cwd against an
unconsenting entry, in band, naming the key)"*.

**This device consents to nothing, and cannot.** The deviation above is why:
dispatch is a call into a Rust function, not a spawn, so there is no directory
to run *at* — `tools::tool` has no consent parameter, and every advertised
element rides without the key (which is what "absent reads false" is for). The
subject reason stands behind the mechanical one: a phone rarely holds a
conversation's worktree, and the box that does is the co-located thrall §5.4
already names as the normal install.

So `crate::host` refuses **every** invocation carrying a `cwd`, in band, with
the capture's three facts: exit code `tools::UNCONSENTED`, empty stdout, and a
sentence naming the key, the tool, the directory that was not entered, the fact
that nothing ran, and the two ways to get the work done. Its own exit code
rather than `BAD_INPUT`'s, because the arguments were read perfectly and
nothing the model wrote is wrong — the two failures have different fixers.

A blanket refusal is lawful only while nothing can consent, so that is an
**invariant with a test** (`tools::tests`), not an assumption: no advertised
tool may state `subject_cwd`. The day a dispatch here can honour a directory,
that test fails and names the check to change — one fact, and the check is its
consequence rather than a second copy of it. Refusing rather than dropping the
field, because a `cwd` silently ignored leaves both ends believing a tool ran
in the conversation's worktree when it ran wherever this app's uid happened to
be, which is the quiet miss the whole lane exists to exclude.

**The table is host-testable and is tested.** It is ordinary Rust over the
standard library, so the same code the device runs is the code the suite runs;
a tool whose behaviour only the phone could witness would be a tool nothing
verifies.

## 7. The chat screen, and where its mechanics live (bl-0ed6)

The transcript is painted the way the window paints it — collapsing
included — because a tool-heavy conversation is unreadable at a phone's width
without it, and because two clients of one engine showing the same
conversation two different ways is two readings of one fact.

**The projection is pure and the paint is thin, and that seam is the point.**
`src/rows/*` turns decoded entries into rows — the label, the collapsed
one-line preview, the expanded body, whether the row folds at all, and the
turn rollup that replaces a finished turn's machinery with a single census
line. It is ordinary Rust with no egui in it, so all of it sits under the
100% floor and the mechanics are verified without a device;
`src/shell/chat.rs` maps a row's `Tone` and `Role` to hues and draws it. A
row is a **block**, not an entry: a model message that says something and
then calls two tools is three rows.

**The knobs say what they do, and they sit away from the way out**
(bl-f165). They are *show full response* and *show intermediate steps* — the
operator's words for the two classes, not the projection's — right-aligned in
a row of their own under the bar. Left-aligned they sat a thumb's width from
the back control, which is the one gesture on that screen that throws away
where you are; the gap IS the feature. §13.2's touch floor applies to a
checkbox exactly as to a row: the floor is spent as the row's own height and
as the minimum interact size inside it, so each knob is a target rather than
a glyph. Both fit one right-aligned row down to a 320-point display
(measured: 295 points of controls in 300 of content).

**Expansion is derived, never stored** (the desktop's own rule): a row is
expanded when `(in-flight OR its class's knob) XOR the operator flipped it`.
So the override set holds *flips* rather than states, a row that appears
mid-frame is already in its configured state, and a tool call that completes
returns to it with nothing to invalidate. The two knobs are policy and the
flips are viewport ephemera — durable per-seat state belongs on the engine
(REMOTE §7) and is a later ball, so the phone keeps neither.

**The spellings are the desktop's, byte for byte** — the glyphs, the
`· N chars` size hint on a folded tool result, the compaction sentence built
client-side from the counters that crossed the wire, the 160-character
preview cap. They are asserted in this crate's own tests, so a divergence is
a red test here rather than two clients that disagree in front of an
operator.

**One deliberate difference, and its reason:** the desktop pulses an
in-flight row's colour. A phone repaints on a budget it is also spending on
the IME mirror, so the label says `running` and the hue holds still.

**The conversation list takes its ink from the same map (bl-ef9a), and that is
how a provider refusal becomes visible here at all.** yog bl-b43b named a wound
with no badge of its own: a conversation refused at the provider rung comes to
rest `stopped` — the badge set is frozen at four (REMOTE §5.1 #9), so it wears
the word an operator's own `/stop` owns — with an empty transcript and an
empty-stderr trail row. Upstream's answer was to say **which way** the rest came
about at every passive surface, and the half that reaches this seat is the
roster row's `tone`: `bad` for a refusal, `weak` for a start whose driver has
written no branch yet. The `refused` boolean itself rides `reply/agent`, and
`reply/attention` carries the signal word; both are shapes this codec does not
read, so `tone` is the whole of what the phone can see and it was being decoded
and dropped one function short of the screen. It now colours the row's label
through `shell::chat::tone_hue` — one map, two surfaces, because a hue meaning
one thing in a list and another in a transcript would be two colour
vocabularies inside one app. Upstream's own words for why a list is the place
this matters: *"the roster is the operator's one passive sighting of it — a
list where the two read identically is a list that cannot be scanned."*

**The growing answer is the speaking agent's own row, and there is exactly
one of it** (operator ruling, bl-e3d1). The engine writes the tail into the
transcript itself as a streaming entry, so a cadence read already carries
one; the follow lane below then reads the same answer several times a rest.
Painting the lane's fold *beside* the transcript put the same words on the
glass twice, and the projection labelled its row `live:` — a word that is not
a speaker, in the speaker's seat, which §13.3 does not allow. So: the tail
wears `<speaker>:`, exactly as the settled turn it becomes will, and the lane
**replaces** the transcript's own streaming entry rather than adding
anything (`crate::live`). That makes the dedupe structural instead of a
content match — when the read stops carrying a tail there is nothing to
replace and nothing to dissolve — and it gives the flight-end half of the
same defect one answer: **at rest there is no tail at all**, whatever the
response file still holds, because the row's own `flight` is the gate the
lane already obeys.

**The world is re-read at cadence; the answer being WRITTEN is followed, one
shot at a time (bl-4822, amending bl-2842).** `seat::pass::fill` still asks
`workspaces` → `conversations` → `transcript` on every pass of the worker's
own clock, and a gesture still wakes it immediately. What is new is what
happens *inside the wait*: while the focused conversation's row states a
`flight`, the worker asks `follow` on a quicker rest and publishes the tail
as it lands, so arriving text appears several times a cadence instead of
once. The turn's end drops the fold — the finished answer arrives as an
ordinary transcript row, and a fold left standing under it would be the same
words twice.

**The lane is read one shot at a time, and no connection is held.** REMOTE
§5.5 makes that lawful in its own words — *"a read starts holding nothing"*,
*"the **first** frame of any read is the whole tail so far"*, and *"Two reads
by the same seat are two reads: the second starts holding nothing, so it
replaces rather than appending"* — so every read this seat makes is a first
frame and its fold is assignment. Holding the connection would buy
write-cadence and cost three things this device cannot pay cheaply: a second
socket held open on a machine that changes networks hourly (bl-8641), a
second worker thread, since the one there is would be parked on the read and
could not answer a gesture, and the real append fold, whose meaning the
corpus cannot show (below). The day those are worth paying, the design is in
`tests/conformance/expect.rs`'s `follow` note, which states what such an
author owes.

**Two things were measured before choosing, and both narrowed it.** Reading
the *transcript* faster is what the lane exists to prevent: upstream measured
the whole-text frame at 20x amplification, quadratic in the answer's length,
and a transcript re-read is that cost with the whole conversation attached —
a follow read is bounded by the answer in flight. And smoothing the SCROLL
alone buys nothing, because the jump is the chunk rather than the mechanic:
egui's `stick_to_bottom` already follows content exactly, already unsticks
the moment a hand scrolls up and re-sticks when the handle returns to the
bottom (its own documented contract), which is precisely the behaviour this
section asks for. So the arrival rate was the thing to fix.

**The hazard the corpus cannot see, kept because it is still true.** What
changed upstream is not the lane's spelling but its meaning: a follow frame
carries *what landed since that read's previous frame*, and the rule is one
line —

> *"Absorb every frame of a read, in order, onto an empty fold. What you hold
> after the last frame you have received is what you paint."*

The frame body is byte for byte what it was, so **nothing mechanical here can
notice**: the corpus ledger records field paths and types, no version bump was
forced, and a green conformance run says nothing about it. That is why the
decision lives in a place a person reads — the `follow` note in
`tests/conformance/expect.rs` carries it, and `transport::Seat::answered`
carries the trap at the line that would spring it, because it decodes
`stream.last()` and the last frame of an append stream is the final delta
alone. `Seat::ask` hands back every frame and is the held lane's door. This
seat is not on that door: one shot per read means one frame to read, and
`answered` is exactly right for it.

The tool host's `invocations` read (§6) is follow-**class** and is not this
lane: it holds a connection open, but its answer is one frame of rows rather
than a text fold, so §5.5's rule does not reach it.

## 8. Starting a conversation (bl-b64e)

A client that can read every conversation and speak into one, and cannot make
one, is a chat app whose first screen is a list you can only watch. The
boundary already has the act and the wire adds nothing (REMOTE §3): §8.1's
start family is `prepare` — everything a new conversation needs before it is
prompted — answering a **prepared body**, then `prompt`, which carries that
body back with the goal and fires it.

**The prepared body rides through this client whole.** Every field is carried
rather than re-derived: it is the engine's own statement about what was
staged, and a client that recomputed one would be inventing world state it
does not own and would drift the first time the engine's policy moved.
`binding` and `lineage` cross as **real nulls** — the field is present and its
absence is the value — so what came off the wire goes back on it unchanged.

**One rung, and the other two are not omissions.** The bare rung is the whole
slice: a phone is not where a work directory is chosen or a ball is bound. The
richer rungs grow here when a surface on this device needs them, which is the
rule the codec has kept since it landed. (Narrowed by §16.2 under the
full-seat ruling: the ball pane, bl-d587, is the first surface named as
needing the ball rung — the growth rule stands, and it now has a customer.)

**Two gestures on the wire, one act at the glass.** The staging and the firing
are one thing to the operator — a sentence typed into the field at the bottom
of the conversation list — so the seat model runs both and the composer knows
nothing about the pair. That field shares the chat composer's widget id
deliberately: only one of the two is ever on screen, and the IME mirror
addresses exactly one field by that id (bl-014e).

**The field carries a send control (bl-9196).** Enter is not a control a
phone can be promised — the IME's action key is the keyboard's to interpret,
and a message that can be typed but not sent is a chat app that does not
chat. The composer row pairs the field with a button on both screens (one
shared helper, `shell::chat::composer`), and since bl-56d6 the field is
multiline — enter is a newline because the field is DECLARED multi-line to
the IME (§3, bl-6850: the flag is what turns the action key into a return
key, and without it the IME committed no newline at all), which is also what
a phone composer does with enter — the field grows to a cap and scrolls
inside it, and the button is the one send.

**The FIELD's own resting height is the touch floor** (bl-01a6). A `TextEdit`
at rest is one text row inside a two-point margin — nineteen points of box,
which at the bottom of a forty-four point band read as a thin line pressed
into a corner, and is not a target a thumb can hit. The operator's word for
it was *super tiny*. So the field carries padding derived rather than chosen:
half the difference between §13.2's touch floor and one line of body text,
above and below, which makes the resting field exactly the floor and centres
the hint in it instead of sitting it on a baseline. Derived, because the line
height is the platform's — a device with larger text gets a larger field, and
the floor is never the thing that gives. The text was always the transcript's
own size (a `TextEdit` resolves to `TextStyle::Body`, which is what a row's
body label uses); what was missing was the box around it.

**The row is allocated its own height, never the screen's remainder**
(bl-193c). Both callers paint bottom-up, so a row that asks for what is left
is handed the entire rest of the screen, and its two children then resolve
that rect at opposite extremes: a scroller anchors to the TOP of whatever it
is given — the cross alignment is not a thing it reads — while the
bottom-aligned button anchors to the floor. The field painted under the
transcript header and the send button a full screen beneath it, which is one
row seen twice. So the helper allocates a band first: the field's own
last-painted content height, floored at the touch target (§13.2) and capped at
the growth limit, with the cap living there and nowhere else. Last frame's
measurement is the honest input — a widget's height is not knowable before it
is laid out — and it is the CONTENT height that is cached, which depends on
the text and the width but never on the band, so the measurement converges
rather than pinning the field at whatever height it was first handed.

## 9. One app, three components, three bootstraps (bl-15bd, landed bl-7714)

**The ruling** (operator, 2026-08-30): the Android app is named **yog** and
ships all three runnable components, each gated behind an explicit bootstrap
rather than auto-started. The default bootstrap is mTLS client enrollment — the
seat or the foot dialing a host engine, material provisioned out of channel
per REMOTE §1.4. Running the yog server locally on the phone is allowed but is
the deliberate, non-default choice. The old development client is superseded:
its landed foundations are the starting material, not discarded work.

**The identity moved with it.** The launcher label was already `yog`; the
`applicationId` and Java package were `dev.yog.seat`, which names one
component out of three. They are now `dev.yog`, and the accessibility service
is `dev.yog.InterfaceService` rather than a class whose name repeated the
app's. The app has never left the box that builds it and has no upgrade path
to preserve, which is exactly why the id moved now: an install channel makes
it a one-way door.

**The component is derived, never stored.** This is the design's whole shape
and it dissolves the first-run special case rather than answering it:

- **No material — nothing runs.** That is the gate. An unbootstrapped yog is
  inert by construction, not by a check, and the first screen is the three
  offers rather than a component that started itself. It is REMOTE §12's
  *"ship inert"* posture one machine over.
- **A leaf — the leaf says which component.** REMOTE §4.2 puts the grade *on
  the certificate*: `CN=<client>, OU=foot` is a foot and a subject with no
  `OU=foot` is operator grade. So enrolling a phone as a tool host is minting
  it a foot-grade leaf, which is the friction §4.2 wants, rather than tapping
  a setting on the phone.

A stored choice would have been a second authority for one fact, and the two
would disagree the first time an operator replaced a seat's leaf with a
foot's. `src/bootstrap.rs` is the derivation and it is host-tested; reading
the grade is `src/leaf.rs`, a DER walk because this crate links no certificate
library — structural rather than a byte scan, because the **issuer** carries a
common name too and comes first.

**A foot paints no chat screen**, and that is the component working. §4.2: *"A
foot cannot ask about the world: not the workspaces, not the board, not the
trail, not a transcript."* A phone on a foot-grade leaf running the seat's
standing-question loop would earn a refusal per question, per pass, forever —
the operator would read a wall of sentences where a component boundary
belongs. So the foot arm starts the tool host and nothing else, and its screen
is what this machine offers and what it has run. The **structural** narrowing
landed with it (bl-2040, §6): `src/foot.rs` is the three gestures and there is
no path from the host loop to a fourth.

**The first-run surface is three branded, tappable choices** (bl-0d3c,
amending this section). It shipped with *no button, deliberately*, reasoning
from REMOTE §1.4 — and the operator, opening the app, found "no buttons or
anything to do either of the activities". **The reading was too strong.** §1.4
stands and says what it says:

> *"there is no pairing protocol in the wire, no token exchange a stranger on
> the network could initiate. Bootstrap is always an act performed through
> existing trust."*

What that forbids is the app **dialling unauthenticated**. It never forbade a
control. An operator who opens the app is entitled to be told what each
bootstrap is and taken to the screen that explains it, and no widget on this
surface opens a socket. So: **the buttons choose and inform; the material
still arrives out of channel.**

Each choice wears the name of the component it makes this device — **Lernie**
the seat (*operate your conversations*), **Thrall** the foot (*let
conversations use this device's tools*), **Yog** the server (*run the engine
here*) — because a screen listing "seat / tool host / server" is a taxonomy
rather than a choice. A tap opens a real screen: for the two enrollments, what
material is needed, the directory it lands in, and DESIGN §5's three delivery
channels; for the server, §10's recorded blockers.

**Component-derived-from-material is untouched, and that is the load-bearing
half.** A tap stores nothing. It opens the flow that acquires the matching
material, and the component that comes up is still read off the leaf on disk
by `standing()` at every boot. There is no chosen-mode field and there must
never be one — the paragraph above this one is the reason. What the screen
carries is navigation, no more durable than a scroll position.

**One control acts, and it acts on this app's own storage:** *check for
material* re-runs the derivation, so a leaf pushed over a cable comes up
without killing and relaunching the process. Re-reading a directory this uid
owns is not a bootstrap and reaches no network.

**The server offer states its dependency chain and starts nothing** (bl-d6c6).
A button that started an engine which refuses every act would be worse than a
sentence saying what is missing, and §12's ship-inert ruling is the precedent.

**The surface is standing, not first-run (bl-387f).** It shipped reachable
only while nothing was provisioned, and the operator hit the wall the same
day: a device enrolled as a seat had no path back, so the second act —
enrolling the tooling side — could not be reached at the glass. The way in is
now the **yog mark**, a control at the top-left of every screen, whatever
component is running; it toggles the configuration surface open and closed,
and the chooser also states its own exit — a `< back` control whenever a
component is running (bl-e192), because a toggle nobody can see is not an
affordance. Breadcrumbs were considered and rejected — a trail requires every path worked
out, and the paths are not; one standing control asks nothing of the screens
beneath it. The cold device is the same surface forced open, not a separate
first-run screen, and the chooser's standing line says what is running
rather than pretending nothing is. Nothing else moved: a tap still stores
nothing, material still arrives out of channel, and *check for material*
still exits by re-running the derivation.

**What this does not yet do: REMOTE §8.2 entries.** §8.2 rules that a client
can be a client of many servers, each named by its own directory under
`wire/workspaces/<leaf>/` with its own material, address and optional
`workspace` rename. This device reads the **flat** directory, which §8.2 says
*"remains what it has always been, the box's own root"* — so today's shape is
lawful and is the zero-entry case, not a deviation. Multi-entry is a ball of
its own, because it is N models and N host channels rather than a file format.

## 10. Running the engine on this device: the chain, walked (bl-d6c6)

The third component §9 offers is the yog **server** — holder of the world, the
balls, the conversations. This section is the honest evaluation the offer's
sentence is a summary of. **It is not landable, and the reasons are structural
rather than effort.**

**Rung 1 — the crate cross-compiles, and this is measured, not assumed.** The
engine's library and its binaries both build for `aarch64-linux-android`
against the pinned NDK, with `balls`, `brazen`, `litany`, `ureq`, `rustls` and
`ring` in the graph and no C toolchain acquired: `ring` is the crypto provider
on both ends already, for the reason this repo's own manifest gives. The
release link produces a ~13 MB PIE executable for `/system/bin/linker64`, min
API 21. So *"does the Rust cross-compile"* — the question that usually decides
this kind of question — is answered yes and is not the obstacle.

**Rung 2 — the engine must be a child process, not a library.** This app is a
`.so` loaded into a zygote-forked process, so `current_exe()` here reports the
system's app runtime, not yog. That matters because yog's spawn resolution is
**self-multiplex**: `bl`, `litany` and `bz` all resolve to yog's own
`current_exe()` under a leading namespace word. In-process, every one of those
would exec the wrong program. Launched as a **child** from the app's native
library directory the resolution is correct again, because that child's
`/proc/self/exe` is the engine.

That directory is also the only place an APK-shipped executable may be
executed: since API 29 the platform refuses to execute a file in an app's
private storage, and the native library directory is the documented exception.
So the shape, if this is ever built, is a binary shipped inside the APK's
native library set and spawned — never a linked-in engine.

**Rung 3 — the world's agent-tool shims cannot be executed, and this is the
first hard stop.** yog's world seeds `<world>/tools/{bl,litany,bz,…}`: small
`/bin/sh` re-execs of yog under a namespace, written at runtime so *an agent's
bash* resolves `bl` to the embedded balls rather than to whatever is on the
ambient `PATH`. They are generated files in the app's private storage, which is
exactly what rung 2's platform rule refuses to execute. Nothing this repo can
write changes that: the shims are yog's mechanism, and the fix — resolving the
agent's tools to the shipped binary directly rather than through a written file
— is an upstream change to yog's own resolution.

**Rung 4 — git is absent, and this is the second hard stop.** The engine founds
its world by committing, `litany new` commits every workspace, the task store
is a git repository on two branches, and the whole workspace read surface is
`git`. Android ships no git and nothing in yog's dependency graph substitutes
for one. Shipping a cross-compiled git beside the engine is conceivable and is
a project of its own, not a step in this one.

**Two smaller ones, recorded so the list is complete.** The engine's boot mints
its wire certificates by shelling to `openssl`, which is also absent — though a
phone's material is provisioned out of channel anyway (§5), so the mint is the
wrong act here regardless. And a listening engine needs to survive the
platform's background limits, which means a foreground service and a
notification, not a thread.

**What the offer therefore says.** The first-run surface states rungs 3 and 4
in the operator's own terms and starts nothing. That is the whole deliverable
of this evaluation, and it is deliberate: REMOTE §12's *"ship inert"* ruling
says a server that cannot serve refuses in band, and a button that started one
which refuses every act would be strictly worse than a sentence that says why.

## 11. The enroll envelope, and why pasting it is not a pairing protocol (bl-dd7b)

yog's `enroll` act (yog bl-f4e3) mints a client leaf on the engine's own
recipe and answers a reply carrying `{grade, name, address, ca, cert, key}`,
shredding the leaf key server-side. The operator's seat renders that as a QR;
this device reads it. The envelope is that payload with a version tag in
front, and `src/envelope.rs` is this end of it:

```text
{"yog-enroll":1,"grade":…,"name":…,"address":…,"ca":…,"cert":…,"key":…}
```

**REMOTE §8.4's payload-contract paragraph is the authority, and it says so in
those words** — *"The payload contract — this section is its authority"*. This
file records what this device does with the envelope; the envelope's own shape
is quoted from there and settled there. Two facts from it that this end has to
meet and could not derive:

- **PEM rides verbatim, at error-correction level M or lower.** A real mint —
  P-256 keys, 825-day leaves — measures **1567 bytes** of compact JSON. A
  version-40 QR in byte mode carries 2953 bytes at L, 2331 at M, 1663 at Q and
  1273 at H, so the envelope fits at L, M and Q and overflows at H.
  DER-plus-base64 was weighed and refused: it buys ~13% and costs the property
  worth keeping, a field an operator can paste into `openssl x509 -text`. **So
  the decoder's bar is 1567 bytes at level M**, which is the version-33 symbol
  §12's fixture pins (the module map's `enroll-v33m-*` row) — one number, read
  from one place, and the fixture is what proves the decoder meets it.
- **The two keys that do not travel.** `ok` and `kind` say what a *wire answer*
  is and a photograph is not one, so they are absent from the envelope by
  contract rather than by omission. `yog-enroll` is the marker a scanner
  recognizes it by, and the version it will be told about if the fields move.

`request/enroll` and `reply/enrolled` are additions to the wire vocabulary and
took no protocol bump of their own (strict decode already refuses an unknown op
in band). This client is on the enrolled side of both and sends neither; the
decision is recorded in `tests/conformance/expect.rs` as `NOT_THE_MINTER`, and
the corpus's `reply/enrolled` fixture was checked field for field against
`src/envelope.rs` when it was vendored — they agree. (The "sends neither"
half is re-classed by §16.2's full-seat ruling: bl-2ee8 builds the minting
side — this device fires the mint and displays the QR — and amends this
clause when it lands. The enrolled side above is untouched.)

**REMOTE §1.4 is untouched, and the reason is which machine acts.** The new
device performs no channel act at all: an already-trusted **operator-grade**
seat performs the mint over *its own* authenticated channel, and the material
travels out of channel — a screen, and an operator's eyes. That is §5's third
delivery channel arriving, not a token exchange a stranger could initiate.
Nothing in this module dials.

**The tag names the envelope and states its version in one field.** A payload
carrying a version but not a name would be read out of whatever JSON a camera
happened to see. The version is checked first and refuses naming both — the
fail-closed shape `src/hello.rs` gives the wire preface, one channel over.

**The grade is not taken on the envelope's word.** REMOTE §4.2 puts the grade
on the certificate and §9 derives the component from it; an envelope field
that disagreed would be a second authority for one fact, and landing it would
enroll this device as something its own leaf is not. So the stated grade and
name must AGREE with the leaf's own, and a disagreement refuses naming both —
it is a defect in whatever minted the envelope, caught at the one moment the
material can still be refused. The envelope's `grade` field is good for
exactly that check and for nothing else.

**Paste is the QR's degraded path, and it is the same sink.** A camera that
will not focus, a denied permission, an operator reading a laptop screen: the
text field has to work regardless, so it was built first. The decoder
(§12, bl-d815) is only a producer of the same string, feeding a path already
proven end to end on real glass — every refusal above is reached by one path
however the text arrived.

**The field holds a private key while it is full**, and is emptied the moment
it lands and on the way back out of the screen (`Shell::forget_envelope`).
Nothing logs it. It is the one place this app holds key material it was not
handed a file for.

**Landing over a provisioned device states what it destroys** (bl-f12d). One
device holds one leaf (§9, §13.3), so landing is not addition: `envelope::land`
overwrites the four `material::WANTED` files in place, and the private key that
was there is gone — the engine shredded its copy at the mint (above), so
nothing on either side can hand it back and the only recovery is a fresh mint.
The operator hit this live: a device running as a seat took a Thrall envelope,
derived Foot on the next boot, and every chat screen lawfully vanished, which
reads at the glass as an app that lost the way back to the conversations. The
land was correct; it was silent. So the enrollment screen paints the running
identity and that consequence **beside the control that acts** — above the
enroll and scan buttons, in the words §13.3 rules (`<name> · Lernie`, the
brand and not the grade). **Not a confirmation**: §13.2 says nothing in this
app is modal except the camera, and a dialog dismissed on the way to a button
is read less than a sentence standing under one. A cold device is told
nothing, because there is nothing to lose and a warning about an identity that
does not exist is noise.

## 12. Scanning the envelope: the decoder and the capture route (bl-d815)

§11 built the paste field first and called a decoder "only a producer of the
same string". This section is that producer, and the two decisions it took.

### 12.1 The decoder is `rxing`, and the alternatives were refused

**The ruling is the operator's** (2026-08-30) and it is a rule 6 dependency
decision, because the honest finding was that **the platform ships no decoder
this app can reach**: AOSP has no barcode API in any public surface, and there
is no intent a stranger app may rely on. The three real routes and what each
costs the *shape* of the app:

- **ML Kit barcode scanning — REFUSED.** It is not "the platform route": it is
  Google Play Services, a Gradle dependency tree this repo has never had,
  several MB of bundled model in the APK, absent on a device without Play
  Services, and it brings CameraX or a camera2 pipeline with it.
- **From scratch — REFUSED.** A Reed–Solomon decode behind a binarizer, a
  finder-pattern locate and a perspective unwarp: buildable, and then
  maintained forever under the 100% floor, for a format with three mature
  implementations.
- **`rxing`, the Rust port of ZXing — TAKEN.** It puts the decoder in the Rust
  core, which is **the one place this repo's gate can judge a dependency**:
  `cargo-deny` reads its licenses, advisories, sources and bans, and the Gradle
  side has no equivalent gate at all. It added no new license (Apache-2.0 was
  already allowed), no advisory and no C toolchain. The feature set and the
  reason each flag is load-bearing are recorded at the manifest line, which is
  where the cost lands.

`src/scan.rs` is the whole Rust surface: a frame in, `Option<String>` out. It
knows nothing about envelopes — **the sink is unchanged**, so the version
check, the grade-versus-certificate law and every refusal sentence in §11 are
reached by one path whether the operator pasted or scanned.

### 12.2 The capture route is camera2 in the Java shell, and there is no preview surface

The app is Rust/egui over `GameActivity` with a thin Java shell. Two routes
were live: the NDK camera API called from Rust, or camera2 in the Java shell
handing frames across. **The Java shell wins on the machinery already
present**: this repo has a proven JNI bridge with the class-loader trap already
solved (`src/tools/ui/bridge.rs`, now over the shared `src/shell/jvm.rs`),
while the NDK route needs either a new dependency or raw FFI — and rule 3
confines every `unsafe` to `src/shell/sys.rs`, so a camera's worth of FFI would
be pressure on the one file whose whole value is being small.

**The session has exactly one output and it is an `ImageReader`.** No
`SurfaceView`, no `SurfaceTexture` handed to wgpu, no CameraX: the Rust side
paints the preview from the very buffer it decodes, so the egui frame loop
keeps the one surface it already owns and the class of defects where the
preview works but the decoder sees something else cannot arise.

Three consequences are structural, and two of them were learned on the
emulator rather than reasoned:

- **Rust pulls; Java never calls in.** A Java→Rust callback needs a
  `#[unsafe(no_mangle)]` entry point, which is rule 3 pressure again. The frame
  loop is already a loop, so it polls: one state call and one frame call per
  pass, both through the shared plumbing.
- **A frame is packed only when the reader has taken the last one, into one of
  two buffers that alternate.** A fresh ~900 KB array per frame killed the app
  outright — `OutOfMemoryError` on the camera's `HandlerThread`, whose death is
  the process's. Steady-state allocation is now zero, and the packer catches
  even `OutOfMemoryError`, because a sentence beats a stack trace only logcat
  sees.
- **The permission is asked per tap, not per process.** The platform's own
  `checkSelfPermission` cannot tell "the dialog is up" from "the operator said
  no", so `MainActivity.onRequestPermissionsResult` records the answer and
  `dev.yog.Camera` answers one of four words. A denial closes the scan screen
  and writes its sentence into the enrollment screen's own refusal line: what
  the operator lands on is the paste field with an explanation, never a preview
  that will not fill.

## 13. The app, whole: workflows, chrome, vocabulary, parity (bl-a246)

The screens above grew one operator sighting at a time — a way in (bl-387f),
a way out (bl-e192), a send control (bl-9196) — and each fix was right and
none of them was a design. This section is the design: what an operator does
with this app end to end, the chrome every screen answers to, the words the
app speaks, and the ledger separating parity this wire can already carry from
what is an upstream ask. **The bar is ordinary**: this is a phone chat app,
and it should hold up beside any of the LLM apps on the same shelf. Novelty
here is a defect.

### 13.1 The workflows, end to end

- **W1 — first run.** Open the app cold: the configuration surface is the
  whole screen. Three branded offers; tapping **Lernie** shows what material
  is needed, where it goes, and the three delivery channels; a QR scan or a
  pasted envelope lands it; *check for material* (or the landing itself)
  re-derives, and the seat comes up on the workspace roster. No restart, no
  choice stored.
- **W2 — daily chat.** Roster → conversations → transcript, each one tap;
  attention marks and tone ink say where to look before anything is opened.
  Speak through the composer; start a conversation through the same row on
  the list screen. Reads refresh at cadence and a gesture wakes the worker
  immediately (§7).
- **W3 — this device's tools, for a seat.** **There is nothing to do**, and
  the app must say so where the operator would otherwise go looking: a
  Lernie seat already runs the tool host beside the asker, one identity on
  two connections (REMOTE §5). The roster's `tools:` line is the running
  proof. The Thrall screen carries the sentence (§13.3), because the one
  operator who opens it while a seat runs is the one about to enroll a
  second name this device does not need.
- **W4 — a dedicated tool box.** Thrall is for a device that should offer
  ONLY tools — no chat surface, no questions about the world (REMOTE §4.2).
  Enrollment is W1 with a foot-grade leaf under that device's own name.
- **W5 — re-provisioning.** The mark, from any screen, opens the same
  surface W1 used; the chooser states what is running and offers the way
  back (bl-e192). Landing new material and rechecking is the exit that
  changes what runs — and the enrollment screen says so before it is taken:
  the running identity and the fact that landing replaces it and destroys
  the old key stand beside the control that acts (§11, bl-f12d), because
  the act is one keystroke and its remedy is a trip to the engine.
- **W6 — the engine on this phone.** The Yog offer explains the recorded
  blockers and starts nothing (§10). It is the deliberate, non-default
  choice and reads like one.

### 13.2 The chrome contract

- **One standing bar**, painted before every screen: the mark (the drawn
  application mark, not a wordmark — the walk ported from the yog crate),
  then the screen's back control when it has a parent, then the screen's
  title. A screen paints no heading and no back control of its own; the bar
  is the one place depth is spelled. The mark toggles the configuration
  surface; back walks one focus depth.
- **The platform's own back control means exactly that** (bl-550e). The
  gesture-nav back is the control every Android thumb reaches first, and it
  was inert: `GameActivity.onKeyDown` answers `KEYCODE_BACK` true — spending
  the platform's default before it can run — and hands the key to the native
  glue, which enqueues it, so it arrived at this app as an ordinary
  `BrowserBack` key that nothing read (`shell/back.rs` records the chain).
  It is now read once per frame and **taken by whatever has a depth to
  walk**: the bar wherever it paints a back control, and the scan screen,
  which paints no bar because it is a camera and for which closing the camera
  IS one depth up. A press nothing took means there was no depth left, and
  that is where leaving the app belongs — performed by hand, because step one
  already spent the platform's own. No screen is enumerated: the rule is the
  same one sentence at every depth, and a new screen inherits it by painting
  a bar.
- **The composer is one shared row** at two depths (§8): a multiline field
  that grows to a cap and scrolls inside it, beside a send button that is
  THE send. The platform residual makes this the only honest shape: the
  IME's enter key is a newline on this stack (§3 — the field is declared
  multi-line and declares no action, because GameActivity writes an action
  where the enter key does not read it), which is also what every phone chat
  app does with enter anyway.
- **What is anchored to the floor claims its space first** (bl-192c). A
  bound rect is not a clip: `app::pass` bounds what a screen is GIVEN, and a
  screen whose content exceeds it simply paints past it — with the keyboard
  up and a tuning band shown, the controls, the composer and the knobs all
  went through the floor and under the gesture-nav bar. So the two screens
  that anchor controls to the floor are laid out in the floor's own order:
  the acts and the composer are painted FIRST, bottom-up from the floor, and
  the chrome (the bar, the banner, the knobs) and the list or transcript take
  what is left above them. **The transcript is what gives way**, down to
  nothing — which needs saying to egui, since a `ScrollArea` refuses to be
  shorter than its `min_scrolled_height` however little room it is given
  (the composer's own defect, bl-9cfd, one level up). Measured at 320 and 400
  points, keyboard up and down, tuning band shown: nothing paints past the
  floor. **That measurement was a throwaway rig and no harness of it was ever
  committed** (recorded here by bl-78c2, which went looking for one): the
  claim is a reading somebody took, not an assertion anything re-runs, which
  is exactly how the same class reached a third site. `shell/place.rs` is the
  first committed geometry assertion in this tree.
- **The platform's insets are the interface's edges, and they are spent
  once.** `app::pass` pads the top inset and **shrinks the rect every screen
  is painted into by the bottom one** (the taller of the keyboard and the
  gesture-nav bar, §3), so the floor is structural rather than a discipline
  each screen has to remember: a bottom-up layout anchors to it, a top-down
  scroller ends at it, and no screen spends an inset of its own. It became a
  fact rather than a habit the hard way (bl-9cfd) — the composer's field
  painted under the gesture-nav bar with its send button correctly above it,
  because a `ScrollArea` will not go below its `min_scrolled_height` however
  little room it is given, and the 20 points it took by force inherited the
  row's bottom alignment and landed in the nav bar. Two rules came out of it:
  the inset is spent in one place, and a widget that cannot fit inside the
  band it was handed is a widget deciding the layout — say the smaller
  minimum, do not pad around it.
- **An opened list is inside the tappable area, and that is asserted rather
  than remembered** (bl-78c2). The two rules above make the floor structural
  for everything a screen LAYS OUT. A popup is not laid out: egui gives it an
  `Area` of its own, positioned by `RectAlign::find_best_align` against
  `Context::content_rect` and constrained to the same — the viewport minus
  egui's **safe area**, which is a first-class notion `egui-winit` fills in on
  iOS and nowhere else. On Android it is zero, so `content_rect` is the whole
  display, gesture-nav zone included, and the selectors sit on the floor by
  design (the controls row is the last thing the bottom-up stack adds). A list
  opening downward from one therefore painted where taps never reach the app.
  The rule: **a list opens into the room the tappable area actually has** —
  below its control when it fits there, above when that side is roomier,
  capped to the room either way and scrolling inside the cap; when neither
  side has room, nothing opens, because a list an operator cannot tap is worse
  than a control that did not respond.

  Two things about how it is built are the point rather than detail. **It is
  not a `ComboBox`**: the combo exposes neither its popup's alignment nor a
  constraint rect, so there is no setter to reach for, and `controls/drop.rs`
  assembles the same `Popup::menu`-over-a-button one layer down. And **the
  rule is not in the paint**. `shell/place.rs` decides and states the band
  egui will then paint; it is pure, host-tested, and stays out of
  `tarpaulin.toml`'s exclusions. That seam exists because this class had been
  fixed twice — bl-9cfd's floor, bl-192c's floor-first order — with nothing
  but prose to hold it, and came back a third time at the one site neither
  reached. The composition *fit → list ⊆ tappable area* is asserted over a
  sweep of screen shapes, control positions and list heights: the first
  geometry assertion in this tree, and the reason there need not be a fourth
  fix.
- **The launcher icon is the mark, and it is a derivation** (bl-0b31). The
  app's face outside the app is the same walk `shell/mark.rs` paints inside
  it — emitted as an adaptive icon's two `VectorDrawable` layers by
  `icon::drawable` and pinned byte-for-byte against the committed assets by
  its own test, so a constant moved in the walk fails a test naming the file
  to regenerate rather than leaving two pictures of one mark. **XML, and
  never a PNG**: the disclosure gate refuses any binary it cannot read, which
  is correct, and an icon is one of the two things a project usually commits
  a binary for. There is no density fallback because minSdk 28 means there is
  no device that needs one. The safe-zone geometry is Android's: a 108-unit
  viewport whose central 72 is the unit square the walk works in, which puts
  the mark's furthest ink at 64.2 units — inside the 66-unit circle every
  launcher mask keeps.
- **One controls row, under the composer, inside the same floor** (bl-0267).
  The composer is what you are SAYING; the row beneath it is what you are
  saying it WITH — the worker's provider and model, and (as they land) the
  acts on a turn already running. It is added to the bottom-up layout before
  the composer, so it sits between the composer and the platform's floor and
  rides the keyboard with it; its height is the touch floor, spent both as
  the row's own height and as the minimum interact size inside it. One row
  owns every conversation-level act, so a new one is an entry here rather
  than a new place to look.
- **The tuning pair rides a second band, and only when the provider will
  take it** (REMOTE §9.4, bl-dfbb). *Effort* is how much reasoning the
  worker's model calls request (`low`/`medium`/`high`/`off`, a closed
  vocabulary no wire read backs — `off` is the absence of a level carried as
  a real null, not a fourth word); *priority* asks the provider's priority
  lane, a toggle rather than a tri-state because asking for the STANDARD lane
  is a different intent no config key expresses. Both act on the worker role
  at the next step, so they take mid-conversation like the model pick, and
  both are **shown only when the selected provider's own row states the
  capability** — the widened `reply/providers` carries `effort` and
  `priority` booleans, and reading them is the same discipline the greyed
  credential fact keeps: the engine states it, this seat never derives it
  (§8). The gate is answered in covered code (`codec::pick::tunable`), not in
  the paint.

  **The effort face carries its own name** (bl-b191). Every other control in
  the block is named where it is read: a provider selector shows a provider,
  a model selector a model, the priority toggle the word *priority*. A
  magnitude names nothing — `medium` alone is a level of something the
  operator must guess, and the guess on record guessed *context size* — so
  the face reads `effort: <level>` once one is standing, keeping the empty
  state's word instead of replacing it.

  They are a **second band** under the first rather than more controls in it,
  for a measured reason: three selectors and a toggle beside the conversation
  acts leave a model selector too narrow to read a model name in at a
  320-point width, and egui's own wrapping layout does not answer it — the
  `ComboBox` these selectors were did not declare its width to the wrap
  check, so it overflowed the column instead of moving down (measured: 418
  points in a 390-point column), and the drop-down that replaced it truncates
  at its width rather than wrapping (bl-78c2). Either way the second row is
  allocated, not wrapped into. One block under the composer, two rows when
  there is something in the second: that is still one place to look.
- **Tap is the act, and the controls load what the workspace actually has**
  (operator ruling, bl-e9f9 — this replaces the *shows only what this device
  set* rule the row shipped with). Nothing in the row holds a draft: picking
  a model IS the assignment, and an engine that refuses one says so in the
  same banner every other refusal uses. What a selector DISPLAYS is now the
  workspace's own assignment, read from the lineage tip the tuning gestures
  write to, so a seat reads its own write back and a fresh install shows what
  is set rather than a row of placeholders. The old rule was not wrong when
  it was written — no shape on the wire stated the assignment, and §8 forbids
  re-deriving one — it was a gap in the wire, and REMOTE §9.4's read closed
  it.

  **An act is optimistic and the read is truth.** A tap paints immediately,
  because the round trip is seconds; the assignments are re-read straight
  after the act, and when that lands the optimistic value goes. A refusal is
  covered by the same motion — the engine never took it, so the read never
  carries it, so the control snaps back to what IS set and the banner says
  why. Nothing needs a second mechanism for the refused case.

  **The effort word is the file's, not this codec's vocabulary.** The config
  may hold a level the four-word gesture set does not spell; it is shown as
  itself, unselectable but never dropped, because flattening it to *nothing
  set* would be this app lying about the workspace to keep its own
  vocabulary tidy.

  **An engine that predates the read says nothing.** The deployed build
  refuses the op in band by name, and that means *no preload* — silently. A
  banner there would be this app telling an operator off for running the
  engine they have; the controls simply start empty, exactly as they did
  before the read existed.

  The selection belongs to the workspace it was made in and goes when the
  focus leaves it. A provider row is greyed **by the credential fact it
  states about itself** and stays tappable: the operator may be about to sign
  it in, and a control that vanishes teaches nothing.
- **The outbox: a sent message paints at once** (bl-66fb). A deposit is a
  round trip and a chat app that shows nothing for the length of one is a
  chat app you press twice, so the composer's text appears the instant it is
  sent, in muted ink, where its row will be. Three states and three signals
  the app already has: **sent** is local (muted); **landed** is the engine's
  receipt (ordinary ink, with a rule under it — the rule is the *not yet in
  the transcript* mark); **taken** is the message coming back in a transcript
  read, at which point the echo dissolves into the row it became. A refusal
  gives the text back to the composer and the banner carries the engine's own
  sentence.
  The echo is the composer's own state and lives beside it in the shell —
  `crate::rows` is pure over what the engine has written down, and this is a
  message it has not written down yet — while the *rule* for when it stops
  being an echo is `crate::outbox`, host-tested under the floor. **The known
  weakness, named rather than hidden:** a deposit's receipt carries no id
  this codec reads, so the match is on content within the transcript's tail,
  and two identical consecutive messages are indistinguishable — the honest
  fix is upstream (a receipt naming the entry it wrote), not a cleverer
  guess here.
- **Wide content: prose wraps, a fenced block scrolls** (bl-b62b). Anything
  a person reads as text wraps at the width it actually has and never
  scrolls sideways — a horizontal scroller under a paragraph is a paragraph
  nobody finishes. The one exception is content whose lines mean something:
  a code fence, where a wrapped line is a changed line, gets a horizontal
  scroller of its OWN and the surrounding prose keeps wrapping around it.
  The trap that made this a rule: a bare label inside a horizontal layout
  does not wrap in egui — a non-wrapping row reads as *extend* — so an
  expanded row body ran off the glass and was clipped mid-word (measured:
  1017 points of label in a 400-point display). A row that lays out
  horizontally for a stripe or a toggle has to ASK for the wrap.

  **And it has to be every one of them, which is why fixing the body alone
  did not hold** (bl-e86c). egui expands the enclosing `Ui`'s `max_rect` to
  include whatever a child painted, so a single widget that overflows widens
  the whole column and every correctly-wrapped label under it then wraps at
  the widened width — measured: one extending prefix took a 390-point column
  to 495, and the body fixed by bl-b62b wrapped at 474 and still ran off the
  glass. So the invariant is *nothing in a horizontal row extends*, and it is
  machine-checked (`rules/unbounded-label-in-row.yml`): inside a horizontal
  layout, text states its wrap mode — `Label::new(..).wrap()` or
  `.truncate()` — and never rides the `ui.label`/`colored_label`/`weak`
  shorthands, whose mode is the ui's. In a VERTICAL layout those shorthands
  are correct (the ui's mode is `Wrap` there) and are what every screen's
  prose uses, so the rule does not touch them.
- **Touch targets:** every navigation row and action control stands at
  least 44 points tall, full width where it lists — `shell/mark.rs` holds
  the one constant and `screens.rs`'s row helper spends it. In-content
  affordances (a transcript row's fold toggle) read at text size; a row an
  adult thumb misses is a defect, not a style.
- **Status where it happened:** a connection error is a banner under the
  bar; an enrollment refusal paints on the enrollment screen, verbatim from
  the one place the sentence is made. No toast, no dialog — nothing in this
  app is modal except the scan screen, which is a camera.
- **A failure is not an error until it persists** (bl-3202). Swapping back
  into the app raced the network coming back: the first refresh after a
  resume failed on a name lookup and the frame painted a red banner over
  three emptied lists, for about a second. Both halves of that were one pass
  discarding what it already had, so both are answered in the model, where
  there is one clock — the frame renders what it is handed and carries no
  timer of its own. A failed pass **republishes the last answer the engine
  gave**, under the focus it was asked at (a moved focus gets the empty lists
  it honestly has — pairing one focus's rows with another's is the one thing
  a snapshot promises never to do), and its **sentence waits one pass**: the
  cadence is the clock, so a second consecutive failure is exactly "it did
  not clear within one rest". A pass that answers clears the banner at once,
  because a standing success is never in doubt. **A gesture's own answer
  never waits** — a refused deposit, a start the engine would not run — since
  the operator just acted, and silence there is a message that vanished.

### 13.3 The vocabulary rule

**Brands where an operator reads; grades only beside the certificate facts
they name.** The words are Lernie, Thrall, Yog — the components an operator
can say out loud — and each surface's first mention carries a clause saying
what it means (*Lernie, the seat — operate your conversations*; *Thrall, the
tool host — this device's hands, and nothing else*). `foot` and
`operator-grade` appear only where the leaf itself is the subject,
parenthesized after the brand, because they name what the certificate may
say, not what the operator chose. The identity line reads
`<name> · Lernie` / `<name> · Thrall`. **One device, one name, one leaf**:
the grade is what a leaf may say, never a second registration — the Thrall
screen says so while a seat runs (W3), and the engine's own enrollment
refusal for a taken name is upstream's half of the same sentence.

### 13.4 The parity ledger is a file, not this section (bl-fe4c)

This section used to carry the ledger itself: a paragraph of what a phone LLM
app has and where this seat stood, kept by hand, in three groups — in-wire and
built, in-wire and unbuilt, upstream ask. **It is now `parity.toml`**, one line
per absent op with a citation, machine-checked on every walk against the
engine's own roster (§15.5). One home, and a loud one.

**The fold was not bookkeeping — the prose had rotted and only a machine
noticed.** The ledger filed *conversation search* under upstream asks, "the
wire does not carry the fact; a ball on the server's board, not a shim here".
The engine's help table carries `search`, classed `control`, summarised by
upstream as *find the text anywhere: balls, workspaces, conversations,
transcripts*. It was an unbuilt surface wearing an upstream ask's clothes, and
the board had already corrected itself (bl-4c2b) while this paragraph went on
asserting the opposite — which is the case for the fold in one sentence: prose
stays green until a person happens to reread it. A second
sighting came out of the same fold: this seat paints an attention mark on two
screens and fires no `seen`, so the queue it shows can only grow (bl-2889).

What stays here is the shape of the ledger and nothing that can go stale in it:

- **In-wire and built** is not a list any more — it is the set of `act:` tags a
  walk observes, which is derived rather than asserted (§15.5). The balls that
  built each control remain in the log; the state of the tree is a question for
  the gate.
- **In-wire and unbuilt** is a `parity.toml` line whose citation is the ball
  that will build it. Deleting the line re-reddens the gate, which is the
  severability test. Since the full-seat re-scope (§16.2, bl-eac2) this class
  is the whole of the fenced forty-three: the chat-loop-slice group reason is
  retired and no line may cite DESIGN §2 as an absence's reason again.
- **An upstream ask** — the wire does not carry the fact at all — is the one
  class the roster cannot see, because an op that does not exist is in no help
  table. Entries — one device as a client of many engines — is the standing
  example (REMOTE §8.2, bl-d0d2).

  **Push notifications used to be the other, and are not one any more**
  (bl-fcc5). The line read: *"the engine dials nothing, so a channel is a
  REMOTE design act rather than an app feature."* The first half is still
  true and is now a ruling rather than a gap — REMOTE §14.3 refuses
  engine-initiated contact on physics before posture, and refuses the vendor
  relay on what it costs. The second half was the error: the want behind
  "push" is *attention reaches a pocketed phone*, and §17 answers it entirely
  app-side, with no wire change and nothing asked of anyone. An absence
  whose remedy is in this repository was never an upstream ask, which is the
  §13.4 lesson a second time: this paragraph asserted a dependency the board
  had already dissolved.
  **Anything an upstream ask claims must be checked against the roster before
  it is written here**, which is exactly the check that had never been run.

### 13.5 The conversation row's acts, and the menu that carries them (bl-f97c)

`src/codec/row.rs`, `src/seat/acts/row.rs`, `src/shell/screens/rows.rs`.
**The first surface in this app that is opened by a gesture rather than by a
tap**, and the first act that addresses something other than the focus. The
operator's ruling (2026-09-03) is the shape: *the surface for the conversation
acts is a long-press context menu on the conversation row*, and the desktop
seat lands the same design on the same day off the same fact — egui
synthesizes a secondary click from a touch long-press, so one design serves
both platforms and each gets its native trigger for free.

**The gesture is egui's, and it was verified before it was built on.** A touch
held past `max_click_duration` (0.8 s) marks the widget under it `LONG_TOUCHED`,
which `Response::secondary_clicked` reports alongside a real right-click. The
two triggers are exclusive rather than layered: `could_any_button_be_click`
goes false the moment a press outruns that duration, so the release of a long
press is not also a click and opening the menu never navigates into the
conversation as well. egui also wakes itself to check a held press, which is
the half a harness depends on — no event arrives while a finger rests. All of
that is read out of egui 0.36.1; **the walk is what proves it on a device**
(§15.4's `row-menu` beat), and if it ever stops holding, the parity gate goes
red naming all three ops rather than the app going quietly inert.

**The roster is three, and the fourth is a missing READ.** `interrupt` cuts the
conversation off mid-work and sends it the composer's text; `retarget` settles
it onto this workspace's config lineage; `flag` raises a human look with a
reason. `fork` is the group's fourth act and is not offered: its `from` is a
fork point, a commit of the conversation's own history or a `config/<name>`
head, and the engine's own `fork::Attempt` says *"Empty is not a value — the
composer refuses to fire without one, because a fork with no ref is a different
gesture."* Nothing this seat reads names one — the marks and the tip ride the
`agent` read (bl-146b) and the lineage names ride `lineages` (bl-3685), both
unbuilt. A free-text field where an operator types a commit sha on a phone is
not a surface; it is this app asking the operator to be the read §8 forbids it
to derive. So the item is absent rather than dead, `fork` keeps its
`parity.toml` line re-cited to bl-99fd, and the conformance table goes on
refusing its frame by name.

**The composer is the parameter, and that is the whole of the staging.** Two of
the three need text, and this app has exactly one place text is typed (§13.2's
one shared row at two depths). So an item that needs text spends what is in the
composer at the moment it fires, and is **disabled with the reason stated
beside it** when there is none — `interrupt — type the text first`. Disabled
rather than absent, and the reason spelled rather than left to the grey: a
greyed control says a thing is not live and nothing about what would make it
live (the desktop's §4.20 reading, which transfers). Only an act that takes a
parameter spends the field; a retarget that emptied it would eat a draft it
never read.

**Nothing is armed, and stating why is the point.** The desktop's §4.20 makes a
destructive act a PLACE with an arming, and none of these three is one: an
interrupt keeps everything already committed (its cut tool call is reported to
the model in band as having produced no result), a retarget discards nothing
and kills nothing, and a flag *"changes nothing else"*. That is a reading of
the three ops rather than a policy this seat adopts — and it is consistent with
the tree, where `stop` has been a bare button on the controls row since bl-48fa
and an interrupt is a stop with a message after it. **The first row act whose
product is that its subject is gone is where this app earns an arming**
(`delete-agent`, bl-f645), and it should follow §4.20 rather than invent a
second idiom. §13.2's *tap is the act; there is no apply* still governs
everything here.

**The subject is the row, not the focus** — the one structural difference from
every other act this seat posts, and the reason they have a file of their own.
A long press names its own conversation; nothing has to be opened first, and
the operator's current depth may not reach the wire. So the workspace comes
from the focus (a row is only ever painted under one) and the agent is carried
in beside it. The seat's test fires all three at a conversation it has never
focused, which is what a regression reaching for `focus.agent` would fail.

**The menu is a popup, so it obeys `shell::place`** — the same rule at a fourth
site (§13.2's last bullet). egui's own `Popup::context_menu` opens at the
pointer and falls back against `Context::content_rect`, which on Android is the
whole display, gesture-nav zone included; a menu opened from a row near the
floor would paint where taps never reach the app. It is assembled instead from
`Popup::menu`'s pieces with the side and the cap `place::fit` decided, exactly
as `controls/drop.rs` does, and **anchored to the ROW rather than to the
finger** — which is what makes it the same geometry as a selector's list and
lets one assertion cover both. The assertion gained one axis rather than a
second copy: a row is two or three lines where a selector is always one touch
target, so the sweep now runs every anchor HEIGHT as well as every position,
including a zero-height anchor (what a pointer-anchored popup would hand in)
and one taller than the band itself.

**Three fates, and one of them has no read.** Every row act is `seat::posted`'s
three-way outcome and none is idempotent, so a lost reply is never re-sent
(§19). Two can name the read that settles them — an interrupt's text appears in
the conversation's transcript, a flag's mark on the conversation's own row.
**A retarget cannot**, because what it writes is a mark the `agent` read
carries and this seat does not make that read. Its sentence says so outright
rather than naming a read that would not show it: a recovery this app has not
got must not be claimed, and §19's contract is satisfied by an honest *no read
here says which* — the gap is bl-146b's, and it is cited.

### 13.6 Search: one field at the top depth, and the screen its answer opens (bl-4c2b)

`src/codec/search.rs`, `src/seat/asks.rs`, `src/shell/screens/search.rs`.

The wire has answered `search` all along — §13.4's own rot lesson is about
this very op, filed as an upstream ask while the engine's help table classed
it `control`. Upstream's bl-764a is what made it usable from off the box: a
hit's address now crosses as **wire names** (the §5.1 project name, the §3.1
workspace leaf, the agent id) rather than engine-local paths, so a hit is an
address this seat already focuses instead of a path it would have to derive a
name from — which §8 forbids it to do.

**The field's depth states the query's scope.** `search` names no workspace
and no conversation; it is the one read this seat makes that asks the engine
*where to look*. So the field sits at the top depth, where the whole world is
already what is on the glass. A field on the conversation list would have
implied a search scoped to that workspace — a promise the wire does not carry,
and the same class of claim §8 rules out everywhere else.

**The answer is a screen, and gets one for free.** It paints its own bar
(§13.2) and therefore inherits the back rule unchanged: the platform's back
gesture walks out of a search exactly as it walks out of a conversation, with
no screen enumerated anywhere and no second mechanism. The probe names it
`search`, which is what lets a walk say it went there.

**An empty needle is no search, on both sides of the wire, and that is one
rule rather than two special cases.** Before the wire: a cleared field is
answered in `seat::asks::search` and crosses nothing, because the answer being
dropped is this seat's own copy and an operator must be able to leave a search
with the engine unreachable — a clear that needed a round trip would be a
screen a broken channel could trap someone on. After it: the engine's own
spelling of *no search* is an answer whose needle is empty (upstream's `/search`
clear), and it means the same thing. Nothing downstream tells a cleared search
from a search that was never made.

**The answer carries its own question, so "nothing matched" is a sentence.**
Upstream's bl-648a put the needle on the reply for exactly this: without it,
*was a search asked?* and *did anything match?* are the same value precisely
when a search found nothing. `Snapshot::search` is therefore `Option<Found>`
where `None` is *no search was made* and an empty `hits` under a needle is
*this question came back empty* — two different screens, and the wire already
tells them apart.

**It rides the snapshot and never the §14 cache.** The cache is the world the
engine wrote down; a search is a question this operator asked a moment ago, and
reviving one on the next boot would open the app on a search nobody just made.
The answer is held by `Standing` like the deposit counters — a gesture's
answer, not a pass's — so a pass that re-reads the world does not drop the hits
the operator is reading, and a failed search does not drop them either: losing
an answer the engine gave over one it did not is the same defect the §13.2
grace exists to prevent.

**A ball hit paints and does not tap.** There is no ball surface on this device
yet (bl-d587), and a row that navigates nowhere is worse than a row that
plainly does not. The hit is still shown, because *the engine found it* is part
of the answer, and hiding a third of a reply to keep the list uniformly
tappable would be this app editing what came back. The matched field's own word
(`name`/`summary`/`text`) rides beside every excerpt for a related reason: a
name matching a name would otherwise read as the same string twice with nothing
saying why the row is there.

**What the walk can reach, and what it cannot** (§15.6). The field paints on
the roster, which the walk visits, so `act:search` is observed and the
`parity.toml` line is deleted. The hits screen is NOT walked: reaching it means
typing a needle and an engine answering it, and this walk dials nothing by
design. That is the honest state — the gate proves the affordance is reachable,
and what the answer looks like is asserted in host tests over the codec and the
seat instead.

## 14. The paint-first cache (bl-de96)

Switching out of the app and back re-read the whole world through the wire
while the operator watched three empty lists fill. The operator's ruling is
recorded with its own caveat — *generally opposed to caches, amenable here* —
and this section is what that permission is bounded by.

**One writer, one reader, one authority.** The model's worker writes after a
pass the engine ANSWERED; the model's boot reads it once, synchronously,
before the first frame; the next cadence read replaces whatever it painted.
Nothing consults it to decide anything — it decides only what is on the glass
for the second before the first answer lands. It is never consulted again for
the life of the process.

**The focus rides with the rows, because rows are only paintable under the
focus they were asked at.** That is the invariant `Snapshot` already keeps,
and the file keeps it too: a stored depth that disagrees with the stored focus
is refused rather than half-read. It also means the app reopens where the
operator was, which is the same fact rather than a second feature.

**It stores the engine's own envelopes, and that is a deliberate deviation
from the ball's sketch.** bl-de96 asked for the decoded snapshot serialized
with a version stamp; writing that means writing a reply ENCODER, and
`tests/conformance/replies.rs` records why this client has none — *"a reply
encoder would be a second implementation of the engine's own spelling with
nothing to check it against"*. `Entry` alone is a tree of blocks and untyped
usage, and an encoder for it would drift from the decoder the first time a
field moved, silently and only in the cache. So the file holds the reply
envelopes verbatim and reading them goes through the ONE decoder every wire
answer goes through. What the sketch was protecting against — a local file
speaking with the engine's authority — is answered instead by strictness:
**any doubt discards the whole file**. Absent, unreadable, not JSON, another
layout version, another protocol version, an envelope the decoder refuses,
an envelope of the wrong kind, a depth that disagrees with the focus: one
answer to all of them, because a caller has one thing to do with all of them.

**Two version stamps, because two things can move**: the file's own layout,
and `hello::PROTOCOL`, since the envelopes inside are the wire's bytes and a
protocol move can change what they mean.

**It is written on change and nowhere near the enrollment path.** A pass whose
snapshot equals what was last stored writes nothing; a live conversation
changes every pass and a quiet one never does, and the write is proportionate
either way, because this app already paid to receive the same bytes over TLS
on the pass that produced them. The file is `<internal>/cache/seat.json`, a
sibling of `<internal>/wire/` — material and snapshots share no file and no
directory, and nothing in `crate::cache` reads or writes a key.

## 15. Render-and-see: the headless emulator loop (bl-243b)

An agent working on this app could not look at it. Every verdict was either a
host test over pure code or an operator's eyes on a phone, and the whole class
of defect between them — *the screen paints, but the way to it does not
exist* — was unreachable by either. `make screens` closes that: a headless
emulator boots, this app is installed, a named walk drives it through its
screens, and each one leaves a PNG an agent can read and a verdict a gate can
fail on. `scripts/screens.sh` is the loop; `scripts/screens-seed.sh` is what
puts the device in each state, split from it because a seed is what a screen
IS and the loop is what is DONE with it.

### 15.1 The accessibility tree still cannot be read, and now we know why

The ball behind this section asked for structural assertions read out of
`uiautomator dump`, and **that surface does not exist**: egui paints into one
opaque view, so the dump comes back as a single `android.view.View` — no label,
no button, no row, and byte-for-byte the same XML on every screen. The dump is
captured beside every screenshot anyway, so the emptiness lives in the run's
own evidence rather than in a sentence somebody has to believe.

**bl-fe4c walked the way out and it was a dead end.** The parity gate (§15.5)
wanted exactly that tree, and its ruling preferred exporting it over any
self-report. What was found, in this order, is worth keeping because each step
looks like the answer until the next one:

1. **eframe's `accesskit` feature alone does nothing here.** It enables
   `egui-winit/accesskit`, which pulls `accesskit_winit` — whose android
   adapter compiles only under that crate's own `accesskit_android` feature, an
   implicit optional-dependency feature in no default set which neither eframe
   nor egui-winit forwards. The fall-through is a `null` adapter whose update
   method has an empty body: every frame's tree is built and dropped, and the
   dump is as empty as before. eframe is Android-aware enough to `compile_error!`
   on the wrong activity backend here, and still never enables the right
   adapter — that guard is the gap's fingerprint.
2. **Naming `accesskit_winit` directly does put the real adapter in.** No
   Gradle work and no Java of ours: the crate carries a prebuilt DEX and
   installs its own `View.AccessibilityDelegate` on the GameActivity surface at
   runtime.
3. **And then the app aborts.** Raising its first accessibility event,
   `accesskit_android` (`event.rs:64`) unwraps the JNI call
   `getParent().requestSendAccessibilityEvent(..)`; under GameActivity's
   surface view that call returns a `JavaException`, and the unwrap is a
   `SIGABRT`. Measured on the emulator, in this walk's own output: the roster
   painted, the dump attached, and the next step found a dead app and a
   launcher. The same three lines with the same unwrap stand in the newest
   `accesskit_android` release, so it is not a version behind.

**So the dependency came out, and the reason is users rather than tests.** An
accessibility client is not only `uiautomator` — it is TalkBack. Shipping this
would abort the app for exactly the people accessibility exists for, which is
worse than exporting no tree at all. The manifest records the failure at the
line where the dependency would go; **bl-a6f3 is the named exit**, and it is
one feature, one dependency and one deletion when upstream stops unwrapping.

**The fallback is PARITY §6's own** — *"a debug-gated in-process inventory: the
shell serializes the act-tags it painted to a file the harness reads. No new
dependency; weaker, because it is self-reported rather than observed"* — and
the assertion in §5 is unchanged either way, because only where the inventory
bytes come from differs. `src/shell/act.rs` is the recorder; what it can
honestly claim, and the disclosure argument for the channel it writes to, are
stated there.

### 15.2 So the app states what it painted

`src/shell/app/probe.rs` writes one line to logcat when what it would say
changes — the app's answer to *"what am I showing?"*, which is the only place
that fact honestly lives. It carries **two things and no more**:

- **the screen's name**, written at the dispatch arm that chose it. Not
  derived a second time from the same state: a derivation beside a branch is a
  second authority for one fact, and the two disagree the first time a branch
  moves.
- **the mark's rectangle in device pixels.** The mark is the only way into the
  configuration surface (§13.2) and it carries no text at all, so it is the
  one control a harness cannot otherwise find. The app says where it put it;
  the harness taps there and still has no say in what the tap means.

**Nothing else may go down this channel, and the reason is disclosure.**
Logcat is device-wide and readable by anything holding the debug bridge, so a
bar title or a row label written here is world state published to the whole
device — the same editorial rule the task store keeps (AGENTS.md). A screen
name and a rectangle disclose the shape of an app whose source is public
anyway.

The marker `yog.screen` is in the MESSAGE, never the logcat tag:
`android_logger` tags a record with its module path, so a tag filter is a
harness coupled to where a file happens to live, and it would go quietly
silent the day that file moves. Both facts are frame-scoped like
`Shell::back` — taken at the end of the pass — so a screen that stops painting
stops saying it is there, and a stale rectangle can never be tapped.

### 15.3 The engine is not dialled, and does not need to be

Two seeds put the app on any screen with no server anywhere:

- **A leaf, minted per run by `openssl`.** `transport::Seat::open` builds a
  configuration and dials nothing, so a self-signed CA and a leaf under it are
  enough to make the device a seat. The address names a closed port; the wire
  failure is painted, which is a screen worth a picture in its own right.
- **The paint-first cache (§14), seeded from `corpus/`.** This is the
  "recorded endpoint": the rows are the reply envelopes vendored out of the
  server's own codec, not a second spelling invented in a harness, and the
  **focus stored beside them is what selects the screen**. Roster, conversation
  list and transcript are three seeds of one file rather than three
  navigations, which is the same fact §14 already keeps — rows are paintable
  only under the focus they were asked at.

The cache's two version stamps are read out of `src/cache.rs` and
`src/hello.rs` by the seeding script rather than restated in it. A stamp that
outruns the script makes the app discard the file, which surfaces as the wrong
screen name and reddens the walk — the failure is loud, and it is loud in the
one place that already knows.

### 15.4 What gates

**Structure, and reachability.** The screen the app says it painted, against
the screen the walk asked for — then, once, what the whole walk could reach
(§15.5). Still no golden image and no pixel diff: a picture that fails on a
font bump teaches nobody anything, and the PNGs are for eyes, not for
assertions.

The walk, and the standing assertions it exists for:

| step | seed | asserts |
| --- | --- | --- |
| `cold` | nothing provisioned | the bootstrap chooser is the first screen |
| `roster` | a leaf | a leaf alone makes this device a seat, and "main" is its roster |
| `settings` | — tap the mark | **the configuration surface is reachable from the roster** |
| `back-to-roster` | — tap the mark | the mark toggles: a way in with no way out is the same defect wearing the other face |
| `conversations` | focus at a workspace | the conversation list paints under its focus |
| `row-menu` | — hold the first row | **the long-press synthesis works on a device**, and the three conversation acts are on the glass (§13.5) |
| `transcript` | focus at a conversation, at rest | the chat screen paints under its focus, and the nudge is offered |
| `running` | the same, with the stop gates on | the two stop controls paint under the engine's own gates |
| `parity` | — every dump above | **every `control`-classed op is reachable or cited** (§15.5) |

**The seeds grew a state, not a screen** (bl-fe4c). `transcript` and `running`
are the same screen; what differs is the conversation row's own gates, which
REMOTE §3.1 and §9.4 put on the row so no client derives them. Three controls
live behind those gates — *stop*, *stop all* and the *nudge* — and a walk that
only ever saw one state could not observe the other's controls at all. The
`running` seed sets the two booleans to their other lawful value; it invents no
field and reads no spelling the codec does not already decode. The options the
controls row is made of (the providers reply, the roles the workspace is
actually set to, one provider's models) are seeded for the same reason: without
them there is no picked provider, so the model selector is disabled and the
§9.4 tuning band does not paint.

The pair in the middle is the assertion the loop was built for. It found its
first defect immediately and the defect was in the harness's own premises: the
walk was run against a stale APK, and the screen it drove had a heading no
source in this tree emits. A loop that cannot tell you which tree you are
looking at is a loop that will eventually lie to you — which is why `make
screens` builds nothing and refuses an APK that is not there, rather than
quietly rebuilding one.

**One beat is not about a screen at all** (bl-b0a9): before the walk, the run
asserts that every runtime permission the teleoperation corpus asks for is
**declared, accepted and held** on the installed app, read back out of
`dumpsys package`. A runtime permission is a chain of three — the manifest
line, an installer that took it, the grant — and the emulator installs with
`-g`, so it is the one place the granted end of that chain is observable. The
failure it exists to catch is silent everywhere else: an undeclared permission
is not refused at install, it is simply never granted, and the tool then
refuses forever on a device where the operator did everything right. It does
not invoke a tool — an invocation through the host channel needs an engine, a
foot leaf and something to fire `/invoke`, which is bl-05b6's ball — so the
refusal halves stay host tests, where they belong.

**Three more are not about a screen either, and they are the only proof a
platform SERVICE can have here** (bl-5cbd, §16.1 rung 2). A notification
listener is declared, permissioned and bound entirely outside this app's own
code, and every way of getting it wrong fails the same silent way: the enable
appears to work and nothing ever binds. So the walk asserts the state a fresh
install is in (no notification access — the state the tool refuses from), then
performs the operator's act over the debug bridge (`cmd notification
allow_listener`, which the design already names beside the settings toggle),
then reads the platform's **`Live notification listeners`** back — the
listeners it has actually CONNECTED, not the `Allowed` list, which is only the
setting written back. The distinction is the whole beat: a component that does
not exist is allowed just as readily and never appears among the live ones, so
this is what catches a missing `<service>`, a missing
`BIND_NOTIFICATION_LISTENER_SERVICE`, a missing listener intent-filter action,
or a class the dex does not carry. The third beat posts a notification and
asserts it stands in the shade while the listener is bound — the material and
the reader in place at once, which is what bl-05b6 will join with an
invocation. **What none of them proves is that this app READ it**, and no
other evidence exists to look for: the retention ruling means a shade read
leaves no trace anywhere, and one that did would be the defect rather than the
proof.

**Two harness findings that beat paid for**, in the shape of §17.5's. `cmd
notification post` takes `[-t <title>] <tag> <text>` and splits on whitespace
with no quoting of its own, so a quoted argument silently becomes two and the
row lands under a tag nobody expected — every token in that beat is one word
for that reason. And `dumpsys notification` **redacts** the content it prints
(`android.title=String [length=3]`), which is why the match is on the record's
`tag=` rather than its body; that redaction is the platform agreeing with this
rung's own ruling that a shade is not material to leave lying in a dump.

**It is not part of `make check` and will not become part of it.** It needs an
SDK, an emulator and a built APK, and a lint gate that depends on an artifact
a build step produced is a gate that cannot run on a clean box.

### 15.5 The parity gate: what the walk could REACH (bl-fe4c)

The steps above answer *did the app go where it said it would*. The last beat
answers a different question, and it is the operator requirement behind yog's
`docs/PARITY.md`: **if something is interactable in the desktop seat it must
exist here**, caught mechanically rather than noticed by hand.

**Against one roster, never against the other client.** Components meet at the
interface, never pairwise (PARITY §2): a client-vs-client diff has no authority
when the two disagree and goes quadratic at a third surface. The authority is
the engine's own help table, whose rows carry `surface` since protocol 7 —
`control` for an op every seat owes a discoverable interactable, `machine` for
one spoken only by programs — and it reaches this repo inside the corpus that
is already vendored and already replayed. Raising the bar is therefore an edit
at yog: one classification changes, and on the next re-vendor this gate reddens
until a control or a cited exemption answers it.

**The tag is `act:<op>`** (PARITY §4), with the op token — the help row's
`verb`, the envelope's `op`, the corpus filename — as the one name, so no
translation table is born. The visible label stays a human word. It was meant
to ride the control's accessibility node; §15.1 is why it rides a file
instead: written at the paint site of the widget that fires the op, into
app-private storage, armed by a directory the harness creates. What that
self-report can still honestly claim — *this control was laid out and its
rectangle was on the glass* — is argued in `src/shell/act.rs`, and it is
weaker than an observation in exactly one way: nothing outside this app
confirms it.

**The four assertions** are PARITY §5's, over three strings — the vendored
roster, the observed tags, and `parity.toml`:

    roster − exemptions ⊆ inventory      (coverage)
    tags(inventory) ⊆ ops(corpus)        (no unknown tag)
    ∀e ∈ exemptions: e ∈ roster          (no rotted exemption)
    ∀e ∈ exemptions: e ∉ inventory       (no stale exemption)

The last two are what stop the exemption file becoming a place to hide. The
judgement is `src/parity`, pure and under the 100% floor; the driver is
`tests/parity.rs`, `#[ignore]`d because the inventory does not exist until a
device has been driven, and run by the walk with `PARITY_DUMPS` pointed at
the run's output. It reads `.tags` (today's inventory) and `.ui.xml` (the
dump, still captured) with one scanner that looks for the token rather than
for a format, so the changeover at bl-a6f3 deletes code instead of adding a
gate. **The half that needs no device gates on every `make
check`**: `src/parity/tests.rs` reads this tree's own `parity.toml` against the
vendored roster, so an exemption that stops parsing, stops citing, or names an
op the engine no longer classes `control` reddens without an emulator.

**Presence is the claim; depth is this harness's** (PARITY §5). The gate says a
tagged node exists in the walked tree. That it is reachable in bounded
gestures, unclipped and on a screen an operator can get to are §15.4's
assertions and stay there. And **unproven is red**: a control that exists only
on a screen the walk never visits fails honestly — extend the walk, or move the
control. That is why `running` exists, and it is the shape of every future
answer to a coverage failure.

### 15.6 What it does not reach yet

The screens behind a text button — the two enrollment screens and the server
bootstrap — are named by the probe but not walked, because reaching them means
tapping a labelled control and the probe states only the mark's rectangle.
**The hits screen (§13.6) is unwalked for a second reason**: reaching it means
typing a needle AND an engine answering it, and this walk dials nothing. Its
affordance is reached — `act:search` is observed on the roster, which is what
the parity gate asks — and what the answer paints is asserted off-device.
Extending it to every control is the point at which this stops being a probe
and starts being an accessibility tree written by hand; if a walk needs those
screens, the honest next step is a ball that decides which of the two this is.

## 16. The full seat and the teleoperation corpus (bl-eac2)

Two operator rulings, 2026-09-03, and this section is the design they
authorize:

1. **The phone's role is Lernie AND Thrall** — a full seat plus a foot, not a
   chat-first companion. The chat-loop slice stops being a scope fence.
2. **Working teleoperation tools on the phone are wanted** — android tool
   development is in scope in this chain.

**The design lives here, not in REMOTE, and §12.1's own test says why**: *"an
ask that is really about a phone would be an amendment there, and none has
been."* Every teleoperation tool below is an ordinary REMOTE §5.1 advertised
element — three facts, a capture as text — and every seat surface below is an
op the vendored roster already classes `control`. Nothing in this section adds
a noun, a verb, a field or a protocol; where a half needs a doc amendment it is
this file's own (§8, §11, §13.4), each marked where it stands.

### 16.1 Half A — the teleoperation corpus

**Teleoperation means operating *through* the phone from elsewhere**: an agent,
driven from any seat, spending tools whose subject is this device — what it can
see, say, sense and show. The subject-locality invariant (REMOTE §5) is what
makes the corpus lawful without a wire change: these tools' subject is the
phone, so the phone is the executor, and the name a model loads
(`<client>_<tool>`, §5.2) already says so.

**Rung 0 stands and is the fallback.** The nine landed tools (§6: shell, the
file trio, the five interface tools) already teleoperate — `ui_read`/`ui_tap`/
`ui_type` can drive any app on the device by puppeting its glass. The corpus
below adds the *direct, honest verbs* for what the interface tools can only do
by puppetry, each priced in the platform's own currency. Four rungs, each a
ball, ordered by what each costs:

| rung | tools | platform cost | ball |
|---|---|---|---|
| 1 — the paper tools | `device`, `clipboard_set`, `notify`, `open` | no service; `notify` wants the POST_NOTIFICATIONS runtime ask (API 33+); `open` is platform-refused from background (BAL, API 29+) and says so in band | **landed** (bl-f34f) |
| 1b — the sighted pair | `camera` (a still, answered as a path — the screenshot precedent), `location` (one fix) | CAMERA / ACCESS_FINE_LOCATION runtime asks over the bl-d815 hook; both are foreground-bound at this rung — background camera is OS-refused, background location is a separate settings-trip grant this rung does not ask for | **landed** (bl-b0a9) |
| 2 — the notification listener | `notifications` (the shade as text) | a NotificationListenerService: the InterfaceService enable class — a settings act, and the restricted-settings block a second time for sideloads | **landed** (bl-5cbd) |
| 3 — the pocketed foot | no new tool: the host loop is held open by a foreground service, so invocations reach a phone in a pocket | the §14.2 rung-2 price — a permanent notification, radio wakes, task killers; off unless the leaf is foot-grade | **landed** (bl-8bd0, §18) |

**Rung 1's one open platform question is closed, and the answer is in the
platform's own source** (bl-f34f). *Is a clipboard WRITE restricted the way a
read is?* No: `ClipboardService.clipboardAccessAllowed` applies the
focused-window and default-IME tests only under `OP_READ_CLIPBOARD`; its
`OP_WRITE_CLIPBOARD` arm is three lines — *"Writing is allowed without
focus"*, `allowed = true`, break — and is identical in every AOSP branch from
android10 through main. Two limits ride with it and both are written where a
model reads them: a denied write is a bare `return` from a void binder call,
so nothing throws and nothing reports (the one denial left to meet is the
`WRITE_CLIPBOARD` appop set to ignore, which defaults to allowed), and Android
13+ auto-clears a clip about an hour after it is set. Reading the rule rather
than watching one device do it is the stronger answer here, because the
question was never *does this phone allow it* but *what does the platform
permit* — and a read-back check cannot even be built, the READ being the half
that is blocked.

**Rung 1b answered two questions this section left open, and both answers are
about honesty rather than mechanism** (bl-b0a9).

*What a still ANSWERS.* A path, and the file lives in **the app's own storage
under one fixed name** (`camera.jpg`), overwritten by the next call unless the
caller names its own. Three facts decide it. A capture is text (REMOTE §5.3),
so the bytes cannot ride the wire and encoding them would be this client
adding a shape to the boundary — the screenshot's answer, given again because
it is the same question. The app's storage is the one directory this uid can
always write, which is why the interface tools already default there. And a
timestamped name per shot would leave an agent that photographs all day
filling private storage nobody is watching, on a device with no sweep; a
caller that wants to keep two names the second itself. The capture sentence
carries what a reader can act on without the image: the dimensions, the byte
count, which lens, and the path.

**The honest limit rides with it: nothing in this repo fetches those bytes
back.** `read_file` answers text, so a JPEG through it is replacement
characters, and no gesture on the wire carries a file. The path is a handle
for the operator holding the phone (or the cable), and `ui_read` remains what
a model should read when the question is *what is on the screen*. That gap is
the screenshot's too, it is a **wire** question rather than a client one, and
no ask has been made — recording it here is the point, because a tool
description that implied otherwise would be the decoy shape again.

*What a fix must SAY.* Three lines — the position, the accuracy in metres, and
**always the age** — because the failure mode is not a refusal, it is a model
acting on a stale fix: a phone indoors for an hour still has a last-known
location and it is somewhere else. Two details make the age load-bearing
rather than decorative. It is computed from the platform's **monotonic**
elapsed-realtime stamp, not `Location#getTime`, because a clock correction
would otherwise make an hour-old fix read as new. And an answer states its
**provenance** — a new fix taken while the call waited, or the last one this
device recorded — rather than deriving staleness from a threshold constant
this design would then have to defend; the caller is told which it is and the
age tells it how much that matters. A fix with no accuracy says *accuracy
unknown* rather than reporting a zero, which is `device`'s rule for a battery
that reports no level.

*Three gates, and one of them is this app's own scanner.* The runtime grant
(the dialog once per run when this app is in front, on **each tool's own
request code** through the bl-d815 hook, and the settings act named otherwise —
`notify`'s shape, and the scanner's request id stays the scanner's so a tool's
answer can never be read as an answer to the enroll screen's ask); the
foreground fact, which the platform enforces for the camera and for a new fix
alike; and, for the still only, **the enrollment scanner holding the same
camera** — opening it twice would evict that session and leave an operator
staring at a dead preview mid-enrollment, so a scan in progress refuses in band
naming the act that clears it. It is reachable precisely because a still needs
this app in front, which is when the scan screen might be up.

**Rung 2 IS the SMS-adjacent surface, and the SMS permissions are refused.**
The teleoperation want behind "SMS" is reading what the phone was told — a 2FA
code, a message — and the shade already carries it as the messaging app's
notification text, behind one settings enable. `READ_SMS`/`SEND_SMS` are
hard-restricted permissions (Play refuses them by policy; sideloads meet the
appops gate), and sending SMS is the operator's own voice on a channel with no
undo — not built without an explicit operator ruling. One enable instead of a
hard-restricted grant, and the read want is answered whole.

**Rung 2 landed with three rulings, and all three are about what a tool may
know rather than what it may do** (bl-5cbd).

*What a caller may read is the whole shade, and no filter is offered.* The
platform's grant has one shape — a listener sees every notification on the
device or none — so a per-app allowlist inside this app would advertise a
narrowing the OS does not enforce, and it is this section's refused per-tool
toggle screen in another costume: a second authority beside the OS grant,
drifting the first time one of them is changed. The severability the house
rule wants is the enable itself.

*Nothing is retained, and that had to be decided rather than defaulted.* A
listener service is a continuous feed — the one tool in this corpus that
receives without being asked — so the tempting shape is a buffer that answers
"what arrived while you were away". It is refused. The service overrides
neither callback, holds no history, writes no file and logs nothing (logcat is
device-wide, and a shade is the last material that belongs there); every answer
is `getActiveNotifications` at the moment of the call, because the platform
already holds the shade and a copy would be a second store of one fact,
durable, on a device nothing sweeps, carrying exactly the material this rung
exists to read. **The cost is real and is stated where the model reads it**: a
dismissed notification is gone, and the tool cannot answer for a moment nobody
asked about. A capability that forgets is worth more here than one that
remembers, and saying so is cheaper than the buffer would have been.

*The rung reads and does not act.* A bound listener may dismiss a notification
and fire its buttons. Neither is built: those are acts on somebody else's app
with no undo, and the ball that wants one is where that gets argued rather than
arriving as a side effect of the enable.

**What only a real device can answer** is the sideload's restricted-settings
block (§6): over the debug bridge the enable is unconditional, so the emulator
proves the service binds and the walk's own beats prove the states either side
of the enable, while the block — which presents as a toggle that will not
stick — meets an operator on a phone. Every refusal names it anyway, because
the sentence has to be right before the device is met.

**Rung 3 shares its grant with the attention lane, and §18 is where it landed.**
A foreground service is the platform's one "my ask may stand" grant (REMOTE
§14), and this device should hold at most one: the service that keeps the
host's `invocations` read standing is the same service bl-b82d's attention lane
wants. `dev.yog.Pocket` is that service, founded by this rung with **one lane
and room for the second** — bl-b82d adds its lane to this class rather than
declaring another, and §18.6 states what it inherits. Unlike the attention
half, the foot half is app-only: `invocations` is an existing follow-class
read, gated on nothing upstream.

**The consent surface is three gates that already exist, and no new one.**
thrall's model is an operator-authored document whose entries are the consent
(its DESIGN §3.4); this device's lawful deviation (§6) is that the table is
built in, so the document cannot be the gate. What gates instead, in order of
standing:

1. **The mint.** A foot-grade leaf — or a seat leaf, whose host rides beside it
   — is the operator's explicit enrollment of *this device as hands* (§9: the
   grade is on the certificate, and minting it is the friction §4.2 wants).
2. **Registration.** Which workspaces' agents can ever see the advertisement is
   REMOTE §1.5's partition, decided at the engine by the operator.
3. **The platform's own grants.** Each capability class is an OS permission or
   a service enable the operator grants in system settings — severable there,
   per capability, enforced by the one party that actually can (the OS), and
   revoking it turns every invocation into an in-band refusal naming the
   grant. A per-tool toggle screen inside the app was considered and refused:
   it would be a second authority beside the OS grant, drifting the first time
   the operator revoked one and not the other — the severability the house
   rule wants is already where the capability is.

Two standing rules carry over unchanged: **the advertisement is static and
whole** (§6's two-tables argument, and one more since the usurper guard — a set
that tracked permission state would rewrite the engine-side document on every
grant flip), and **`subject_cwd` stays never-consented** (§6's invariant, with
its test). A model reaches a phone tool only through the §5.2 `load` act, after
a `get` that showed the descriptions — which is where the per-tool background
and permission honesty (§6's containment rule) must be written, because it is
the one text the model reads before spending a call.

**Reconciliation with bl-5710 ("no tool corpus ships"), which this design does
not reverse.** Three facts, verified against the yog store and REMOTE §5.4:

- bl-5710 is **resolved, not standing**: the operator ruling of 2026-08-31
  (*ship some basic tools — a default install must be able to write a file*)
  landed as the worktree lane's last rung — the engine performs its own
  builtins at its front door. Its residual (does thrall ship a *pool* corpus)
  is thrall's question and blocks nothing here.
- It **never governed a foot's advertised set**. Its defect was the shipped
  worker grant offering names in every conversation's `tools:` array with
  nothing behind them — a decoy paid for once per conversation. A phone tool
  is in no grant: it reaches a model only through an explicit `load` from a
  roster that shows presence, so the decoy shape cannot arise.
- Its editorial lesson is kept as a rule of this corpus: **every refusal a
  phone tool can earn names the one operator act that fixes it** (the grant,
  the enable, the foreground fact), because a refusal that teaches is the
  difference between a priced capability and a decoy.

**The usurper hazard is inherited as a fix, not a defect.** thrall bl-2d78
found the advertised set last-writer-wins under one identity; yog bl-1462
closed it engine-side (REMOTE §5.1: a second parked `invocations` read refuses
in band, and a set-changing advertisement is refused while one is parked). The
phone inherits that guard for free, and the visible residual is the same guard
working — two processes serving under one CN is refused loudly, which §6's
one-identity-two-connections shape never does.

**But the guard is not free of a window, and the phone opens it itself**
(bl-cc54). Both halves of it stand on a parked read, and a foot executing a
tool holds none: this loop is serial, so for a tool's whole runtime the device
is absent and its set may be replaced with no refusal. The remedy is §6's —
re-assert at the end of every hand-off, which bounds the window to one tool's
runtime, and read the receipt's `wrote` (PROTOCOL 8), which turns a silent
self-heal into a sentence on the roster. Neither half is a client amending
REMOTE: the re-assertion is an ordinary §5.1 gesture, and `wrote` is a field
the engine now states.

**Refused shapes, recorded**: the SMS permission pair (above); a generic
run-any-intent tool (the wrapper meta-tool REMOTE §5.2 refused twice — `open`
is typed instead); a clipboard *read* tool (platform-blocked in background,
focused-app-only in foreground — `ui_read` is the honest alternative and reads
what is actually on the glass); and any third-party wake path (§14.3's
refusal, already ruled).

### 16.2 Half B — the full-seat re-scope of the parity ledger

**The fence is retired.** Forty-three of `parity.toml`'s lines cited one group
reason — *outside the chat-loop slice (DESIGN §2)* — which was a scope
statement wearing an exemption's clothes. Under ruling 1 a scope fence is no
longer a reason: every line now cites the ball that will build its surface, or
a reason that survives the ruling. **No line may cite §2 again.**

**All forty-three became unbuilt-with-ball, and zero became
per-platform-never.** Attacked before settling: every one is an engine act or
read fired from glass — the typed-name armings, count fields and text bodies
they want are things this app's composer and rows already do, and nothing
about a phone refuses any of them. The reasons that DO survive are structural,
and were never §2 citations: `follow` and `roles` are ops no gesture fires
(their views are reached through other controls), and they stand unchanged;
`search` (bl-4c2b, **landed** — §13.6) and `seen` (bl-2889) were already
unbuilt-with-ball.
`enroll` is the one re-classification: §11's "enrolls nobody" was the
chat-first framing speaking, and under the mesh ruling (§5) a full phone seat
mints and *displays* the QR the next device scans — bl-2ee8, which amends
§11's clause when it lands.

The groups mirror the seat's own, one ball each:

| group | ops | ball |
|---|---|---|
| conversation acts | interrupt, retarget, flag — **landed** (§13.5); `fork` held back on a read it needs | bl-f97c, then bl-99fd |
| the held tool call | answer, revoke, restore | bl-b39d |
| work review | files, work-diff | bl-5a56 |
| conversation machinery reads | agent, steps, step, rail, governing, inbox | bl-146b |
| the ball pane | balls, workspace-balls, board, close, assign, release, create, update | bl-d587 |
| candidates | fan, retire, deliver, science | bl-2f17 |
| fleet and watch | fleet, disband, arm, disarm | bl-477e |
| trail and attention | ops, ack, clear-trail, attention | bl-35bd |
| admin and armed deletions | config, marks, scan, delete-agent, delete-workspace | bl-f645 |
| roster and discovery | clients, lineages, help | bl-3685 |
| the minting seat | enroll | bl-2ee8 |

**Ordering.** The teleoperation corpus is the operator's stated want and goes
first (bl-f34f → bl-b0a9 → bl-5cbd → bl-8bd0, serialized on `src/tools.rs` and
`android/`). The seat groups serialize against one another — they share the
conformance decision table, `parity.toml` and the shell's surfaces — and rank
behind the corpus where they contend; bl-f97c and bl-b39d lead them, because
interrupt and the held-call answer are the two the daily-chat workflow (W2)
actually misses.

**What does not move yet.** The conformance decision table's §2 constants
(`tests/conformance/expect.rs`) remain true statements of the tree — this
codec still spells none of those shapes — and each group ball moves exactly
its own rows when it lands, which is the codec's standing
grow-per-consumer rule. §8's "one rung" paragraph is narrowed rather than
deleted: the bare start rung stays the default, and the ball rung becomes
wanted when the ball pane (bl-d587) lands, which is §8's own growth rule
doing what it says.

## 17. The scheduled fetch: attention reaches a pocketed phone (bl-fcc5)

Rung 1 of yog REMOTE §14, whole. The gap it closes is stated there and is
platform-shaped, not wire-shaped: *"a phone seat learns its turn has come at
its next read, and a phone in a pocket performs none: the platform ends a
backgrounded app's sockets and schedules it nothing."* The engine initiates
nothing toward a client (REMOTE §3) and §14.3 refuses every shape that would
change that — an engine-side punch at a phone whose NAT mapping is dead, a
vendor push relay, an out-of-band mail adapter. What is left is the platform's
own door, and this section is this app walking through it.

**The whole of it: the OS runs a job, the job performs one ordinary ask, a
rise becomes a notification.** No wire change, no new gesture, no engine
dependency, nothing upstream to wait for.

### 17.1 What it asks

`Query::Workspaces` — the roster read this seat already performs at cadence.
Its rows carry `attention`, the per-workspace count the roster screen paints
its `●` from (`src/shell/screens.rs`), and that is the cheapest
attention-shaped read the vendored corpus answers: one connection, one frame,
rows a handful long, and nothing derived on this end that the engine did not
already say. REMOTE §14.1's `Query::Attention` lane — the standing ask,
answered as a sequence — is upstream's ball (yog bl-09aa) and rung 2's
(bl-b82d); this rung is gated on nobody and could land the day it was
designed.

### 17.2 What wakes a human

**A rise, and only a rise.** `src/attention.rs` keeps what each workspace's
attention stood at when it was last announced. A count that stayed put was
already announced; a count that FELL is the operator having dealt with it. So
a notification is posted only for a workspace whose attention is higher than
its own last announced number — and every run, silent or not, records what it
saw, so a count that drops and climbs again wakes the operator a second time.
Rows are joined into one notification, never one apiece.

**The post replaces, it never appends** (`Notify.STANDING`, a fixed id): a
pocketed phone carries one standing attention row rather than a stack of
them. That is REMOTE §14.1's own rule about frames, arriving at the same
answer from the platform's side.

**Every failure is silence.** No material, an engine that will not answer, an
answer this end cannot read: the run ends with no notification and *no state
written*, and the next schedule tries again. A phone in a pocket must never
nag about network — the operator did not ask for a fetch report. The one
direction the design deliberately fails in is the other one: an unreadable
memory file reads as *nothing announced*, which can cost one notification the
operator has seen before and can never cost a wake that does not happen.

**It never writes the paint-first cache** (§14). That cache has one writer,
the seat model's worker; a fetch that stored a roster over a focus the
operator had taken deeper would paint the wrong screen on the next resume. The
fetch's memory is its own file — `<internal>/attention/seen.json`, a sibling
of `wire/` and of `cache/`, one writer and one reader, holding no key and no
world.

### 17.3 What it costs, and where the operator turns it off

**JobScheduler, not WorkManager.** WorkManager's whole job on API 28+ is to
build the same `JobInfo` and add a database to remember it. This app schedules
ONE periodic job with no chaining and no work graph, so the library would be a
dependency (AGENTS.md rule 6, and it drags Room and a startup provider) bought
for an API already in the platform.

**The OS owns the cadence and the battery price is the platform's floor.**
`setPeriodic` is a request: 15 minutes is the platform's minimum rather than
this app's choice, and in Doze the run is batched into a maintenance window
and can be hours late. Each run is one short mTLS connection and a handful of
rows; the job declares a network requirement, so a phone with no network is
never woken to fail. **Timeliness is what this rung does not solve and cannot**
— that bound is the platform's (REMOTE §14.2), and rung 2 is what buys it,
with a permanent notification and radio wakes as the price.

**The off switch is Android's own, and it is one switch.** The `Attention`
notification channel carries the cost statement above in its own description,
where the operator reads it in system settings, and `Watch` asks
`Notify.armed` both before it arms AND at the top of every run — so silencing
the channel stops the *checking* as well as the telling, and a run that finds
it silenced cancels the job. A fetch whose only product is a notification
nobody may see is battery spent for nothing. There is no second switch inside
the app: that would be a second authority beside the OS grant, drifting the
first time one of them was revoked (§16.1's refused per-tool toggle screen,
for its reason).

**It is armed on every resume**, from `MainActivity` — which is also the
resume after the permission dialog is answered, and re-scheduling an identical
job is how JobScheduler is told nothing changed. It is `setPersisted(true)`
(hence the `RECEIVE_BOOT_COMPLETED` declaration, which receives no broadcast
and exists only because JobScheduler requires it for persistence): a fetch
that quietly stopped at the next reboot until the operator happened to open
the app is the silent degradation this design excludes, and a phone that
reboots in a pocket is the case the rung is for. An unenrolled device is not
gated separately — the run stats one directory and returns, and duplicating
the material contract in Java to avoid that would be a second copy of a fact
`src/material.rs` owns.

### 17.4 The direction of the call, and what a test can reach

**Java calls Rust here, where every other bridge is Rust calling Java.** A job
may start this process with no Activity ever created, and `ndk_context`'s
globals are filled by android-activity on the way to `android_main` — so a
bridge asking the JVM for a class would be reading a handle nothing had
written. `dev.yog.Watch` declares one `native String probe(String dir)`;
`Java_dev_yog_Watch_probe` lives in `src/shell/sys.rs`, the crate's one
`unsafe` location, and is four lines over `attention::sweep`. The answer is
the two-line protocol this crate already speaks (title, then the line under
it), and an empty string is silence.

**So the decision is host-testable and is tested at the floor**: the sweep
dials a real one-shot mTLS server on loopback (the transport's own recipe) and
the suite drives a first rise, a repeat, a fall, an empty roster, and every
class of silence — no material, half material, material that will not build a
seat, an engine that does not answer, an answer to another question. What only
a device can prove is that the platform actually RUNS the job: `make screens`
reads `dumpsys jobscheduler` back and fails if this app has no periodic job
armed after a launch.

### 17.5 Two platform findings the walk paid for

**A job scheduled in the first resume after a force-stop or an install can be
cancelled by the platform seconds later.** `JobScheduler.schedule` returns
`RESULT_SUCCESS`, `dumpsys` shows the registration for a moment, and it is
gone — the cancellation the stopped state implies landing after the call that
made it. It was measured on a cold-booted emulator, where the race is wide;
on a warm one the same sequence sticks. **This is why `MainActivity` arms on
every resume rather than once at startup**, which was written for the
permission dialog's sake and turns out to be load-bearing for a second
reason: the next resume re-arms, so the exposure is one period at worst and
nothing an operator can reach is left holding a promise the platform dropped.
`make screens` judges a resume for the same reason — it asks the question in
the window where the answer means something.

**A `grep -q` on a piped `dumpsys` fails on the runs that MATCH.** `dumpsys
jobscheduler` is hundreds of kilobytes, `grep -q` exits at the first hit, the
writer takes SIGPIPE, and `set -o pipefail` makes the pipeline 141 — so the
beat reddens exactly when the thing it is checking for is present. Both fetch
beats hold the dump in a variable and match with a herestring. It cost two
full walks and it is the shape to look for whenever a harness beat fails only
under `pipefail`.

## 18. The pocketed foot: a foreground service holds the lane (bl-8bd0)

Rung 3 of §16.1, and the gap it closes is the one §16.1's own table names:
until this rung the tool host served only while the app process lived.
Backgrounded, the platform ends a cached app's sockets and the foot is absent
until somebody looks at the phone — fine for a seat with hands beside it,
wrong for a device enrolled AS hands.

**The whole of it: a foreground service holds the PROCESS, and the host was
moved out of the activity so there is a process worth holding.** No new tool,
no new gesture, no engine dependency. `invocations` is the same follow-class
read it always was.

### 18.1 The host belongs to the process now

Until this rung the `Host` handle was a field of `shell::boot::Running`, so its
lifetime was the activity's — and the platform destroys the activity when the
app is swiped out of Recents (`stopWithTask` defaults to false; the service
survives, the activity does not). A foot that went with the screen would be
absent exactly when the phone is pocketed, which is the rung's whole subject.

`src/state.rs` is the crate's first lock and AGENTS.md rule 7's named home for
it: one slot, holding **at most one LIVE host**. Two things fall out of that
and both are the point.

- **A stopped host does not own the slot forever.** `Health::Stopped` is a
  refusal no redial mends, and it is published as the worker returns, so
  `Host::alive` reads the PUBLICATION rather than the thread — a `JoinHandle`
  finishes a moment later, and a predicate that read it would answer
  differently depending on when it was asked. Without this the operator's own
  remedy (open the app) would silently do nothing inside a process that had
  stopped one.
- **A latent race is dissolved rather than papered over.** An activity that is
  destroyed and created again — the ordinary android relaunch — used to build a
  *second* `Host` on this device's certificate while the first worker was still
  parked on its `invocations` read, which REMOTE §5.1's one-reader guard refuses
  naming this very device. The slot refusing the second is what makes that
  question unaskable.

The frame reads `state::standing()` where it used to hold a handle, so the
roster's tools line and the shade's notification are written from one fact.

### 18.2 The operator act is the leaf, and there is no switch

§16.1's consent surface is three gates that already exist, and this rung adds
none. **A foot-grade leaf IS the act**: REMOTE §4.2 puts the grade on the
certificate, §9's bootstrap discipline derives the component from it and never
stores one, so a device carrying `OU=foot` holds its lane while pocketed and
every other device does not. `crate::pocket::line` answers `None` for anything
else, and `None` is the service's whole stop condition.

**An in-app toggle was considered and refused**, for §16.1's own reason: a
second authority beside the fact the certificate already states, disagreeing the
first time an operator replaced a leaf without visiting the switch. It would
also have to be *stored*, and a stored want is exactly the second home §9
refused for the component itself. The severability the house rule wants is
already where the capability is — re-provision a Lernie leaf and the next
resume takes the hold down, which the walk asserts.

**The platform's own switches are the other two**, and they are real: the
`Serving tools` notification channel in system settings (whose description
carries the price), and **Active apps → Stop**, which Android documents as
removing the whole app from memory. Neither is duplicated in this app.

`MainActivity.onResume` arms it, for §17.5's reason and one more of its own: an
API 31+ foreground service may only be started from a user-visible state, and a
resume IS that state. Re-starting a service that already runs is how the
platform is told nothing changed.

### 18.3 The type is `specialUse`, because the alternatives have clocks on them

Android 15 (API 35, which this app targets) permits `dataSync` and
`mediaProcessing` foreground services **six hours in any 24**, then calls
`Service.onTimeout` and throws if the service does not stop itself. For a foot
that is meant to be reachable for days that is not a price, it is a defect on a
timer — and `dataSync` is additionally barred from being started by a
`BOOT_COMPLETED` receiver on 15. `connectedDevice` is exempt from both but
means a Bluetooth/NFC/USB companion and carries a runtime prerequisite this app
has no business declaring. **`specialUse` is the platform's own "none of the
above"**: uncapped, no runtime prerequisite, and its
`PROPERTY_SPECIAL_USE_FGS_SUBTYPE` is where the honest sentence goes. That
property is reviewed by Play and never by the platform; this crate is
`publish = false` and ships through no store, so the sentence is true because it
was written to be, not because anything checks it. `FOREGROUND_SERVICE` and
`FOREGROUND_SERVICE_SPECIAL_USE` are both declared — API 34 made the per-type
permission mandatory, and without it `startForeground` throws `SecurityException`
rather than degrading.

**No BOOT_COMPLETED receiver, and the reason is not the platform's.**
`specialUse` is *not* on Android 15's barred list, so one would be lawful. It
would also be useless: this service cannot CREATE a lane. A service may start a
process with no Activity in it, and this app's tool bridges resolve their
classes through handles android-activity fills on the way to `android_main`
(`src/shell/jvm.rs`), so a host built from a service would be a foot whose
platform tools all refuse. `onStartCommand` returns `START_NOT_STICKY` for the
same reason. **The honest limit: after a reboot the foot is absent until the app
is opened once.** The scheduled fetch (§17) is `setPersisted` and still wakes
the operator about attention, so the phone is not silent — only its hands are.
The named exit is **bl-d22d**, which is about making a host startable without
an Activity — a question about the bridges rather than about the service, and a
bigger one than this rung.

### 18.4 What the notification says, and where the price is stated

The notification is not decoration: a foreground service without one is a
service the platform kills, and it is the only surface a pocketed phone has.
`crate::pocket::notice` writes it from the host's standing, in the roster's own
vocabulary — one fact, two surfaces, and a phone that says `reconnecting` on
the glass must not say `serving` in the shade. Four states and no fifth:

| standing | title | what the line under it carries |
|---|---|---|
| serving, presented | *this phone is standing by as hands* | how many tools are offered, and either "nothing called yet" or the served count and the last tool — then the price |
| serving, not yet presented | *this phone is offering its tools* | how many tools are being presented, and the price |
| redialling | *this phone is reconnecting* | the sentence that broke the channel, that no tool call reaches this phone until it returns, and that yog keeps trying more slowly each time |
| stopped | *this phone has stopped serving* | the sentence that ended it, and that **nothing is on the network now** — the other half of an honest price |
| hands, no lane | *this phone is not serving* | the one act that answers both of its causes |

**A healed disarming is appended wherever it applies**, in `host::RESTORED`'s
own words and with a count. REMOTE §5.1's guard heals a replaced advertised set
automatically; being *told* is the part that is not automatic, the roster is
where it is painted, and a pocketed phone's roster is not being looked at.

**The price is stated in both places it is read** (the house rule): the
channel's description in system settings, which is where a standing cost
belongs (§17.3's precedent) and which also names the two acts that end it, and
the notification's own text, which is what the operator actually sees. §14.2
prices this rung as a permanent notification and radio wakes and this says so
in the operator's own words.

**A stopped lane does NOT stop the service, and that was a decision.** The
tempting shape is to stop when the host stops, since a stopped host spends
nothing. It is refused: the notification is then the only evidence that a
pocketed phone has stopped answering, and a service that vanished would take
that evidence with it. A stopped host holds no socket and no thread — the line
says exactly that — so what stands is a notification and a resident process,
which is the cheapest possible way to keep an operator-actionable fact where
the operator will meet it.

**Without `POST_NOTIFICATIONS` the service still runs**; Android documents the
notification as simply absent from the drawer while remaining in the
foreground-services manager. Nothing here asks for the grant — a service is not
a screen, and `Notify` is where the ask lives.

### 18.5 The redial ledger: what this client takes from thrall, and what it does not

thrall's redial loop (its bl-916d, `src/run/redial.rs`) is the same problem on a
box that roams less. Reading it against this client's landed ladder (bl-8641)
found a defect that would have made this whole rung a lie, so the ledger is not
a formality. **Adopted:**

- **Classification by who failed at which leg, never by the engine's prose.** A
  device that decided its own lifetime by reading sentences would be one the far
  end could rewrite by rewording. `Stop::Wire { why, read, served }` carries the
  leg; `crate::transport::Wire` carries the class.
- **The three-row matrix, which needed a third class here.** `Wire` had two
  variants and collapsed *the engine said no* with *the engine said something
  unreadable* — the decoder already draws that line (its outer error is
  unreadable, its inner one is the engine's `ok: false`) and `answered` was
  discarding it. `Wire::Unusable` is that line made visible: a stream that ended
  without answering, a frame that is not JSON, a reply of a kind the gesture does
  not earn, a version that cannot be spoken to.
- **THE ONE REFUSAL THAT MUST BE RETRIED, and it is the defect.** A read parked
  when the connection dropped does not leave until the engine tries to answer
  it, so a redial inside that window meets REMOTE §5.1's one-reader guard
  refusing **this very device** — its own dying predecessor, not a rival. This
  client treated every refusal as final, so a phone's *first wifi handover*
  stopped its foot for good, three seconds after the drop, with a sentence
  naming itself. A pocketed foot cannot have that defect and it is the one this
  rung exists to prevent.
- **The predecessor floor, at thrall's own constant.** 32 seconds — REMOTE
  §5.1 states one hold's width as a contract (*"a peer that vanished without a
  FIN frees the slot within one hold's width — thirty seconds"*), plus two,
  because this end's window began before it noticed the drop. Asking sooner
  earns the same sentence and spends a handshake to hear it.
- **An advertise-refusal still ends the channel** (bl-2d78's settlement): that
  one means another connection holds this device's read with a different set in
  force. Two refusals, two answers, never collapsed into one retry.
- **A channel that SERVED returns the ladder to its floor** — replacing this
  client's weaker "a channel that was accepted". The two differ exactly where it
  matters: a rival holding this device's read accepts every advertisement while
  refusing every read, so the weaker predicate would reset the ladder forever on
  the one ending that has to back off. An answered read — an empty one counts —
  is the engine having parked this device for its own hold.
- **No disarm knowledge across redials.** A `wrote: true` on a channel's FIRST
  presentation says nothing and is discarded (`host::serve`, bl-cc54); a redial
  makes a fresh channel, so that silence holds across one. Nothing is remembered
  over the gap and there is nothing to resume.
- **No deadline of its own**, and no attempt count. A device that changes
  networks hourly has no number of failures after which giving up is right.

**Deliberate divergences, each with its reason:**

- **The cap moved from 30 seconds to thrall's 64, and it had to.** This
  client's 30 was chosen so a phone walking back into the house is served again
  within half a minute — but a cap *under* the 32-second floor makes the ladder
  inert for the one ending that repeats, so a rival permanently holding this
  device's read would be dialled every 32 seconds for as long as the battery
  lasted. Above it, the series climbs past the floor and that case settles at a
  minute. The half-minute promise is kept by the reset, not by the cap: a
  channel that was served starts again at one second, which is what a wifi
  handover is.
- **The entry is opened per host, not per dial**, as thrall does — but this
  client's `Foot` is opened once by `shell::boot` and outlives every channel,
  because the material read is a fact about this box and asking it again would
  ask the same question.
- **No `notice` callback.** thrall prints the ending sentence as it happens
  because under a loop it is no longer returned; this client publishes a
  `Standing` on every boundary and the sentence rides `Health::Redialling`,
  which the roster and the shade both read. Same requirement, and this end
  already had the channel for it.
- **A single-entry process does not exit.** thrall still exits when it cannot be
  a foot at all, because supervision is the right owner on a server box. There is
  no supervisor on a phone and no exit code anything would read; the equivalent
  is `Health::Stopped` reaching both surfaces, which §18.4 is about.

### 18.6 Doze, flaps, and what bl-b82d inherits

**Doze does not take the socket away, and that is the platform's own rule.**
Network access is restricted per UID for processes below the
foreground-service threshold; a running foreground service puts this process
at `PROCESS_STATE_FOREGROUND_SERVICE`, which is above it. What Doze still costs
is *wakeups*: a foreground service grants no wakelock, so with the CPU
suspended nothing here fires on time. **So the redial is the feature, not the
fallback.** A phone changes networks, sleeps, and comes back with a dead TCP
mapping the far end has not noticed; the ladder above is what turns that into a
one-second gap instead of a dead foot.

**Vendor task killers remain the residual §14.2 named** and no design here can
answer them: an OEM power manager that stops a foreground service is doing what
Android's own Active-apps switch does, and the app cannot tell them apart.

**bl-b82d adds its lane to `dev.yog.Pocket`, not a second service.** What it
inherits: the grant and the type (one `specialUse` service per device), the
`Notify` channel discipline (its own channel, its own description, its own
price), the arming point (`MainActivity.onResume`), and `START_NOT_STICKY`. What
it must decide for itself is the lane's own stop condition and whether the
service should stand when only one of the two lanes is wanted — the natural
shape being that the service runs while EITHER lane answers, which is what
`pocket::line` returning `None` already means for this one.

### 18.7 What the emulator proves, and what only a device can

Host tests at the coverage floor own the decision: which devices hold the
pocket, what the shade says in every state a host can publish, and the whole
redial matrix over a real mTLS server (`src/pocket/tests.rs`,
`src/state/tests.rs`, `src/host/tests/redial.rs`). `make screens` owns the half
no host test can reach — whether the PLATFORM accepts any of it — in seven
beats (`scripts/screens-background.sh`), which is also why the two background
lanes moved into their own file: unlike the read-only platform beats, these
MOVE the device.

The walk asserts, in order: a **seat**-grade device holds no foreground service
(the off-by-default half, without which the next beat would prove the opposite
of the design); a foot-grade leaf makes the platform hold `dev.yog/.Pocket`;
the platform **promoted** it (`isForeground=true`, which is what puts this
process above the Doze threshold) and recorded the **specialUse** type; a
notification stands on the foot channel; the process **survives a background
kill with the screen away** (`am kill` reaps a package's background processes,
and a foreground-service process is not one — surviving it is precisely the
property this rung buys); the hold stands through an **airplane-mode cycle**,
with the flap itself asserted before the survival is judged; and
**re-provisioning a seat leaf stops the hold**, which is the operator act
reversed.

**What only a real device can answer**, and none of it is dodged here: whether
a vendor's power manager leaves the service alone for days; what the hold
actually costs a battery over that time, which is the number §14.2 prices and
no emulator can measure; and Doze's real behaviour on a phone that is genuinely
still, screen off, off charge, for hours — the emulator never enters deep Doze
on its own. Those are recorded as this rung's real-device residue on bl-8bd0.

## 19. The lost reply: what an act in doubt does here (bl-07b1)

yog's REMOTE §3 gained one bullet (its bl-d1f1) and it is the whole of this
section's premise:

> **A lost reply leaves an act IN DOUBT, and the recovery is a read — never a
> resend.** A connection that dies between the engine completing an act and the
> reply frame landing tells the client nothing about whether the effect ran, and
> nothing on the wire can be added to say it: an act is not idempotent (§9.8 —
> two clicks of Nudge are two nudges), an engine-side receipt journal could only
> assert *dispatched*, never *committed* [...] So no idempotency token rides the
> act envelope and no redelivery slot exists for acts: a client whose act earned
> a transport error instead of a reply paints the failure and consults the world,
> which is the durable record [...] **Asks are the opposite case and re-ask
> freely**: a read is answered in place, and asking twice is asking once (§9.7).

Nothing wire-visible moved — PROTOCOL stands at 8, the corpus is unchanged —
so everything below is behaviour, and most of it was already right. What was
not is named as a defect, because that is the half worth writing down.

### 19.1 Where the doubt begins, and why the line is the write

`transport::Wire` carries the class, and it gained a fourth member rather than
a flag on an existing one:

| class | what happened | dial again? | in doubt? |
|---|---|---|---|
| `Transport` | the channel failed **before** the gesture left | yes | no |
| `Lost` | the gesture was written and **nothing answered it** | yes | **yes** |
| `Refused` | the engine spoke, and said no | the leg decides (§18.5) | no |
| `Unusable` | the engine spoke, and this end cannot read it | no | no |

**The line is drawn by the framing, not by a guess.** An `io::Error` out of a
write is that write's bytes not being accepted, and `frame`'s reader takes a
length header and then exactly that many bytes without ever scanning — so a
frame that failed mid-write is a frame no engine decoded, and a socket that
would not open said nothing at all. Both are ordinary failures. A frame written
whole is the opposite: this end cannot learn whether it arrived, whether it was
answered, or whether the answer went into a socket that had already gone.

**Two questions, two predicates, and keeping them apart is the point.**
`Wire::transport()` is *is the channel what failed* — the tool host's ladder
reads it, and `Lost` answers yes, because a foot that stopped on a dropped
completion would be exactly the wifi-handover defect §18.5 exists to prevent.
`Wire::in_doubt()` is *may the act have run anyway*, and only `Lost` answers
yes. The engine's own preface is the one exchange after the write that is still
definite, so a version this end cannot read stays `Unusable`: yog's listener
writes its version before it reads a request frame (`wire::hello::admit`), so a
peer that never stated one never read the gesture.

**Asks consult none of it.** `seat::asks` — the selectors' three reads and the
live tail — re-ask on the next tick with nothing remembered, and a read's
failure is one sentence for the banner exactly as it always was. That is why
those four functions moved out of `seat::acts` into a file of their own: the
contract's own line is the seam, and a file that may resend is not the file
that must not.

### 19.2 The seat: three fates, and the draft that must not come back

`seat::posted::Posted` is the outcome every act now ends in — `Took`,
`Refused`, `InDoubt` — and `Snapshot` counts deposits in three rather than two.

**The defect this closed is one tap wide.** The composer's echo watches the
deposit counters move (bl-66fb): on a refusal it takes the operator's text back
into the field, because the engine said no and saying it again is an ordinary
first attempt. A lost reply used to count as a refusal — so a phone that
dropped its connection mid-deposit handed the draft back, one tap from a second
copy of a message the engine may already have taken. On a device whose radio
drops connections routinely, that is the tempting reconnect-and-replay loop the
contract forbids, wearing an operator's finger as its retry.

So an act in doubt **keeps the echo standing**, marked, with the contract under
it in the operator's own words — the reply was lost, this may or may not have
been taken, it was not sent again, and the transcript will show it if it
landed. It is a state and not a control: there is no resend button, because a
resend is a second message and the read that settles it is the transcript the
echo is already standing in. The seat re-reads that transcript every cadence
without being asked, so the recovery needs no gesture at all — and when the row
appears, the echo dissolves into it exactly as a landed one does.

**Every act names its own read**, because "consult the world" is only useful if
the client says which part: a deposit points at the transcript, a stop or nudge
at the conversation's row and its `flight`, a start at the workspace's
conversation list, and the three config writes at the assignments read the
worker makes straight afterwards anyway (bl-e9f9). One sentence builder
(`seat::posted`), so the contract is worded once.

**The banner keeps the sentence for one cadence and the echo keeps it for as
long as the doubt lasts**, which is the split that matters: an act's `note` is
taken once and replaced by the next pass, and in-doubt is a durable fact about
a particular message rather than a transient failure.

### 19.3 The foot: the one gesture that may not be repeated

The pocketed foot redials forever (§18.5), and a redial re-asserts the
advertised set on every fresh channel. That is lawful and stays: a presentation
is idempotent by design, and since PROTOCOL 8 the engine's own `wrote` reports
whether it changed anything. **The completion is the gesture that may not be
repeated**, and the loop already did the right thing — the capture is moved into
the gesture and goes with the channel that could not carry it, and the redial
presents and reads afresh rather than carrying an answer over. What was missing
was that anyone had said so, and a test: the ladder had been proven on the
`invocations` leg and never with a hangup on `complete`.

**The recovery here is the engine's, and this device must not help.** REMOTE
§5.3's invocation leg is at-least-once *by design*: a claim whose taker vanished
is requeued (yog bl-e658), so the work this device could not answer is offered
again on the next `invocations` read, and this device runs the tool again and
answers the new delivery. Nothing here remembers an invocation id to suppress
that — thrall's DESIGN §3.8 declines exactly the same memory and re-runs, and
this is that ruling from the other side. A device that redials must not keep
memory across the gap it redials over; the honest cost is that a tool can run
twice, and it is the effect owner's to make safe, never the wire's to promise.

**Nothing new is painted for it.** The shade and the roster already say
`reconnecting` with the sentence that broke the channel, and a doubted
completion adds nothing an operator can act on: the engine re-delivers, this
device answers, and a mark for the seconds in between would be noise on the one
surface a pocketed phone has.
