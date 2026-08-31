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
default path is mTLS client enrolment; running the engine on the phone is
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
Java OnKeyListener in the Gradle shell. The IME action key (Send) is a known
residual: GameActivity writes the action where the enter key does not read
it, so enter stays a newline until upstream moves.

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
| `src/codec.rs` + `codec/{fields,ws,conv,transcript,reply}` | the chat-loop slice: encode message/workspaces/conversations/transcript, strict decode of their replies; spellings pinned to the server byte for byte | landed (bl-fe33) |
| `src/material.rs` | the seat's key material: three answers (off / half-provisioned named in full / provisioned) | landed (bl-48d9) |
| `src/tls.rs` | rustls client config, ring named never defaulted | landed (bl-48d9) |
| `src/transport.rs` | the Seat: one connection per ask, server name off the address | landed (bl-48d9) |
| `src/test_support.rs` + `test_support/serve.rs` | tests only: openssl-minted PKI; the one-shot and scripted multi-connection mTLS answering servers | landed (bl-48d9, split bl-5a98) |
| `src/rows.rs` + `rows/{build,compacted,project,project/blocks}.rs` | the transcript's one-line row projection: the row vocabulary (class, tone, role, fold), the per-entry match and its labels, the preview/body split — pure, no paint | landed (bl-0ed6) |
| `src/rows/turns.rs` + `turns/{steps,counts}.rs` | the turn rollup: where a turn is, when its machinery folds to one aggregate line, and the census that line says | landed (bl-0ed6) |
| `src/tools.rs` + `tools/{shell,files}.rs` | what this machine can run: the built-in table, its advertisement, and the dispatch | landed (bl-d366) |
| `src/foot.rs` | REMOTE §4.2's foot set as a type: the three gestures, and no way to reach a fourth | landed (bl-2040) |
| `src/host.rs` | the tool host loop: advertise, ride the follow read, run, complete | landed (bl-d366) |
| `src/tools/ui.rs` | the interface tools: their advertised elements, argument reading, and the two-line answer protocol — pure | landed (bl-1511) |
| `src/tools/ui/bridge.rs` | android-only: the JNI into the accessibility service, class resolved through this app's own loader | landed (bl-1511) |
| `android/…/{InterfaceService,UiTree,Gestures,Screens}.java` | the platform service: read the node tree, dispatch a tap, type, press a system control, screenshot | landed (bl-1511) |
| `src/seat.rs` + `seat/model.rs` + `seat/tests/{reads,deposit,start}.rs` | the view model: owns the `Seat` on one worker thread, re-asks the standing set at cadence, publishes `Snapshot`s, posts deposits | landed (bl-5a98) |
| `src/shell.rs` + `shell/span.rs` | shell root + UTF-16 span math (the host-tested sliver) | landed (bl-c761) |
| `src/shell/{sys,inset,bridge,app}.rs` | android-only glue: the confined `unsafe` + entry, the JNI inset probe, the two-way IME mirror, the frame loop | landed (bl-c761) |
| `src/shell/screens.rs` | android-only: the three screens by focus depth over the model's snapshot | landed (bl-5a98) |
| `src/shell/chat.rs` | android-only: painting one projected row — the stripe, the toggle, the two-line speaking shape | landed (bl-0ed6) |
| `src/bootstrap.rs` | which component this device is, derived from the leaf on disk; the three offers a cold device paints | landed (bl-7714) |
| `src/leaf.rs` | the DER walk over this device's own leaf: its client name and its REMOTE §4.2 grade | landed (bl-7714) |
| `src/shell/boot.rs` | android-only: the bootstrap gate — read the standing, start exactly that component, start nothing otherwise | landed (bl-7714) |
| `src/shell/enrol.rs` | android-only: the first-run surface — the three bootstraps, and no button | landed (bl-7714) |
| `android/` | the minimal Gradle shell: manifest (INTERNET), games-activity trio, the OnKeyListener backspace shim | landed (bl-c761) |

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
rule the codec has kept since it landed.

**Two gestures on the wire, one act at the glass.** The staging and the firing
are one thing to the operator — a sentence typed into the field at the bottom
of the conversation list — so the seat model runs both and the composer knows
nothing about the pair. That field shares the chat composer's widget id
deliberately: only one of the two is ever on screen, and the IME mirror
addresses exactly one field by that id (bl-014e).

## 9. One app, three components, three bootstraps (bl-15bd, landed bl-7714)

**The ruling** (operator, 2026-08-30): the Android app is named **yog** and
ships all three runnable components, each gated behind an explicit bootstrap
rather than auto-started. The default bootstrap is mTLS client enrolment — the
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

**The first-run surface has no button, deliberately.** REMOTE §1.4 stands:
*"there is no pairing protocol in the wire, no token exchange a stranger on
the network could initiate. Bootstrap is always an act performed through
existing trust."* Every widget on that screen reads; none acts. A tap that
"started enrolment" would be the unauthenticated connection §1.4 forbids,
dressed as a convenience. What the screen carries is what each bootstrap makes
this device, the act that takes it, and the directory material lands in — the
fact an operator holding a cable is actually there for.

**The server offer states its dependency chain and starts nothing** (bl-d6c6).
A button that started an engine which refuses every act would be worse than a
sentence saying what is missing, and §12's ship-inert ruling is the precedent.

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
