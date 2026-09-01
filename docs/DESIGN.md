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

**The wire is at PROTOCOL 2 (yog bl-77be), and this seat speaks it (bl-8553).**
The number lives in `src/hello.rs` and in `corpus/shapes.json`, and the
conformance suite asserts they are the same — so a corpus vendored from a yog
that has moved on, and a preface bumped without re-vendoring, are each a red
test rather than a skew discovered on a handshake. Three meanings moved, and
they are not the same kind of move:

- **Two the ledger caught**, which is why there is a version 2 at all: REMOTE
  §5.1's advertised element gained an optional `subject_cwd`, and §5.3's
  invocation gained an optional `cwd` — the two halves of the worktree lane.
  Both are spelled in `src/codec/tools.rs`, because `request/advertise`
  round-trips and `reply/invocations` reads; a field this codec did not carry
  would be one dropped on the way out. What this device *does* about them is
  §6's, not the codec's.
- **One the ledger could not see**, and it is the hazard worth naming: REMOTE
  §5.5 made the follow lane's frame an **append** — *"absorb every frame of a
  read, in order, onto an empty fold"* — under a wire spelling that did not
  change. A signature ledger records field paths and types, so nothing forced a
  bump and no fixture can fail. A client of that lane must read the section
  rather than re-vendor the fixtures and call it consumed.

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
| `src/host.rs` | the tool host loop: advertise, ride the follow read, run, complete — and refuse a carried `cwd` (§6) | landed (bl-d366, bl-0ac8) |
| `src/tools/ui.rs` | the interface tools: their advertised elements, argument reading, and the two-line answer protocol — pure | landed (bl-1511) |
| `src/tools/ui/bridge.rs` | android-only: the JNI into the accessibility service, class resolved through this app's own loader | landed (bl-1511) |
| `android/…/{InterfaceService,UiTree,Gestures,Screens}.java` | the platform service: read the node tree, dispatch a tap, type, press a system control, screenshot | landed (bl-1511) |
| `src/seat.rs` + `seat/model.rs` + `seat/tests/{reads,deposit,start}.rs` | the view model: owns the `Seat` on one worker thread, re-asks the standing set at cadence, publishes `Snapshot`s, posts deposits | landed (bl-5a98) |
| `src/shell.rs` + `shell/span.rs` | shell root + UTF-16 span math (the host-tested sliver) | landed (bl-c761) |
| `src/shell/{sys,inset,bridge}.rs` + `shell/app.rs` + `app/pass.rs` | android-only glue: the confined `unsafe` + entry, the JNI inset probe, the two-way IME mirror, what the shell IS and what one frame does with it | landed (bl-c761, split bl-dd7b) |
| `src/shell/screens.rs` | android-only: the three screens by focus depth over the model's snapshot | landed (bl-5a98) |
| `src/shell/mark.rs` | android-only: the yog mark control — the walk said in egui's primitives, toggling the configuration surface | landed (bl-387f, drawn mark bl-ff27) |
| `src/icon.rs` + `icon/arc.rs` | the application mark's generation walk, ported from the yog crate: compass-work arcs, the flat shape list, the hue drive — pure, host-tested | landed (bl-ff27) |
| `src/shell/chat.rs` | android-only: painting one projected row — the stripe, the toggle, the two-line speaking shape | landed (bl-0ed6) |
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
| `android/` | the minimal Gradle shell: manifest (INTERNET, CAMERA), games-activity trio, the OnKeyListener backspace shim, the permission-result hook | landed (bl-c761, bl-d815) |

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

**The read path is a re-read at cadence, not the follow lane — and since
REMOTE §5.5 that is a decision worth stating (bl-2842).** `seat::model::fill`
asks `workspaces` → `conversations` → `transcript` on every pass of the
worker's own clock, and a gesture wakes it immediately; `Query::Follow` is
sent nowhere. What changed upstream is not the lane's spelling but its
meaning: a follow frame now carries *what landed since that read's previous
frame*, and the rule is one line —

> *"Absorb every frame of a read, in order, onto an empty fold. What you hold
> after the last frame you have received is what you paint."*

The frame body is byte for byte what it was, so **nothing mechanical here can
notice**: the corpus ledger records field paths and types, no version bump was
forced, and a green conformance run says nothing about it. That is why the
decision lives in a place a person reads — the `follow` rows in
`tests/conformance/expect.rs` carry the reason, and `transport::Seat::answered`
carries the trap at the line that would spring it, because it decodes
`stream.last()` and the last frame of an append stream is the final delta
alone. `Seat::ask` hands back every frame and is the lane's door.

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
rule the codec has kept since it landed.

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
multiline — enter is a newline (the residual made it one anyway, and it is
what a phone composer does with enter), the field grows to a cap and scrolls
inside it, and the button is the one send.

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
`src/envelope.rs` when it was vendored — they agree.

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
  changes what runs.
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
- **The composer is one shared row** at two depths (§8): a multiline field
  that grows to a cap and scrolls inside it, beside a send button that is
  THE send. The platform residual makes this the only honest shape: the
  IME's enter key stays a newline on this stack (§3 — GameActivity writes
  the Send action where the enter key does not read it), which is also what
  every phone chat app does with enter anyway.
- **Touch targets:** every navigation row and action control stands at
  least 44 points tall, full width where it lists — `shell/mark.rs` holds
  the one constant and `screens.rs`'s row helper spends it. In-content
  affordances (a transcript row's fold toggle) read at text size; a row an
  adult thumb misses is a defect, not a style.
- **Status where it happened:** a connection error is a banner under the
  bar; an enrollment refusal paints on the enrollment screen, verbatim from
  the one place the sentence is made. No toast, no dialog — nothing in this
  app is modal except the scan screen, which is a camera.

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

### 13.4 The parity ledger

What a phone LLM app has, and where this seat stands. **In-wire** (this
codec already carries the fact — build it here): the conversation list with
unread marks and status ink (landed), starting a conversation (landed), the
send control (landed), streaming-at-cadence reads (landed §7), settings
reachable from anywhere (landed), the drawn mark, the standing bar, the
multiline composer and the touch-target floor (children of this section).
**Upstream asks** (the wire does not carry the fact; a ball on the server's
board, not a shim here): conversation timestamps on roster rows;
conversation search; a stop control for an in-flight turn (the slash
vocabulary is the boundary's — whether a deposited `/stop` is that gesture
is upstream's answer to give); push notifications (the engine dials
nothing, so a channel is a REMOTE design act, not an app feature).
**Already on this board:** entries — one device as a client of many engines
(REMOTE §8.2, bl-d0d2).
