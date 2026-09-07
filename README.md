# yog-android

**yog on Android** (`dev.yog`): the app ships all three of the harness's
runnable components — the **seat**, the **foot** (tool host) and the
**server** — each gated behind an explicit bootstrap rather than auto-started
(DESIGN §9). The default path is mTLS client enrollment: a leaf provisioned out
of channel, and this app dialling a host engine with it. Agents run on that
engine; seats attach and detach, and the work does not.

The **server** bootstrap is offered and does not start: the engine
cross-compiles and links for this architecture, but Android ships no `git` and
refuses to execute the shell shims the world seeds into app-private storage.
DESIGN §10 is the whole chain, rung by rung.

**Which component runs is read off the leaf, never stored.** No material and
nothing runs — the first screen is the three bootstraps as branded choices:
**Lernie** the seat, **Thrall** the foot, **Yog** the server. A tap opens the
screen that explains one; it never dials and never stores a mode. The two
enrollment screens take the **enroll envelope** a trusted seat minted —
pasted today, scanned once a decoder is adjudicated — validate it against the
leaf's own grade, land it, and bring up the component the certificate names. A leaf with `OU=foot`
runs the tool host; any other leaf runs the seat (REMOTE §4.2). The foot's
wire surface is a type with three methods, so a gesture outside the foot set
is a compile error here rather than a refusal at the engine.

The wire contract is the server's (yog `docs/REMOTE.md`): big-endian `u32`
length-prefixed JSON frames, a zero-length terminator, mTLS with
operator-provisioned certificates, and the client always the asker. This
repo's `docs/DESIGN.md` records what the client is and mirrors; `AGENTS.md`
is the working discipline.

## Status

Client standing. The frame layer, the chat-loop codec slice, mTLS
transport, key material, the Android shell (egui via GameActivity — DESIGN
§3), and the seat view model (workspaces → conversations → transcript +
composer) are landed. On-device verification is operator-assisted; an
unregistered leaf correctly sees empty rows.

**Wire v1.** Every connection opens with the version preface REMOTE §3
defines, and a skew is fail-closed with a sentence naming both versions. The
vocabulary is judged by yog's **conformance corpus** — generated from the
server's own codec, vendored here as `corpus/`, and replayed by
`tests/conformance/`: every frame decodes, everything this client emits
round-trips, and every shape outside its slice is a recorded decision that
refuses by name.

## Dev loop

```
make check          # the complete local gate == CI == pre-commit hook
make conformance    # replay the vendored wire corpus (REMOTE §3)
make test           # cargo test
make coverage       # tarpaulin, 100% floor (pinned 0.35.2)
make lint           # line-cap + leak-scan + clippy + ast-grep + cargo-deny
make apk            # cargo-ndk (arm64-v8a + x86_64) + gradle assembleDebug
make apk-release    # the release variant, signed with the permanent key
make deploy-phone ADDR=<ip:port>   # that APK, arm64 only, onto a phone
make screens-avd    # create the emulator the loop below boots, once
make screens        # headless emulator: walk the screens, capture each one
make parity         # judge a walk's dumps against the engine's control roster
make install-hooks  # seat the pre-commit / commit-msg hooks, once
```

The APK build needs the Android NDK, `cargo-ndk`, the two Android targets
pinned in `rust-toolchain.toml`, and a **gradle** 8.7+ on JDK 17. There is
deliberately no gradle wrapper: the wrapper is a committed jar, and the
disclosure gate refuses any binary it cannot read — correctly. The release
profile is load-bearing, not an optimization (see the Makefile `apk` target).

**Which gradle is `scripts/gradle.sh`'s one rule**, and both doors spend it:
`GRADLE=/path/to/gradle` outright, else `PATH`, else the newest bin
distribution under the gradle wrapper's own dists cache — because a box that
has ever run a wrapper has a whole distribution there and no `gradle` command,
which is exactly the box this repo is developed on. It had two spellings once,
and they disagreed: `make deploy-phone` found a gradle where `make apk` — the
target it builds through — had already given up (bl-2a31).

One APK carries **both ABIs**: `arm64-v8a` for the phone and `x86_64` for the
emulator the enrollment stories run on. Gradle packs a `jniLibs/<abi>/`
directory per ABI and the installer picks one, so there is no
emulator-only artifact to confuse a test verdict. Override with
`make apk ABIS=arm64-v8a` on a box that only ever flashes a phone.

## Onto a phone

`make deploy-phone ADDR=<ip:port>` builds the arm64 APK at the current tree and
installs it on a phone reachable over **wireless debugging** — `adb connect`,
then `install -r`, and the exit code carries the truth: it is non-zero unless
the device answered `Success`. Nothing is launched and no screen is walked;
install is the whole act.

The address is an argument, never a setting. It appears in no file here — a
routable address is a disclosure the leak gate refuses — and the wireless-debug
port rotates on every re-pair and reboot, so a stored one would be wrong by the
next boot anyway. That is also the honest limit: this is push-on-demand, not
unattended CD, because the address is something a human reads off the phone.

Two things it resolves so you do not have to. `ANDROID_HOME` defaults to the
conventional SDK location and is **exported**, since the Gradle Android plugin
reads it from the environment and a developer SDK exports nothing. And gradle
comes from `scripts/gradle.sh` above — the same rule `make apk` spends, called
rather than restated, so the two cannot answer differently again.
`make deploy-phone ADDR=... GRADLE=/path/to/gradle` still wins over both
probes, the same override `make apk` takes.

## The release channel

There is one publication event per version and the phone can read it, so the
phone reconciles the way every other box in the fleet does — with the last rung
replaced, because **Android will not let a process replace itself**. It detects
and offers; a person taps.

A push to `main` runs `release-plz.yml`: the whole gate, then the release PR
kept fresh and merged when it is green, then — for any manifest version no tag
yet names — the `v<version>` tag, the GitHub Release, and the **signed APK**
attached to it. The feed the app reads is
`GET /repos/<owner>/<repo>/releases/latest` and the artifact is one URL under
it, anonymous and unauthenticated: a device holds no registry credential, which
is the property that makes the channel possible rather than a place to park a
token. Nothing is published to a registry — the crate is `publish = false`, so
release-plz reads the last released version off the git tag instead.

**A protocol bump waits for the engine** (bl-5b19). yog mints the wire protocol
version and this app *vendors* a copy of the constant; the wire is fail-closed
on a mismatch and does not negotiate (yog `docs/REMOTE.md` §3). So
`merge-release-pr` **holds a release whose `PROTOCOL` exceeds the newest
published yog's** — thrall took that road first, publishing 16 while the newest
engine spoke 15, and here the landing is worse because the offer below carries
a released APK to every device that taps it. Strictly greater, not different: an
app *behind* the engine is the mirror-image defect and its release is the fix.
Between the two gates — yog holds a bump until the consumers' mains carry it,
each consumer holds a release until yog has published it — the ordering a bump
requires is: **the consumers' mains first, then yog publishes, then the
consumers publish.**

On launch the app asks that feed once, compares the tag with its own version,
and paints **one row on the roster** when the tag is newer — never a modal, and
nothing at all when it is not. Tapping the row downloads the asset and hands it
to the system installer; the person taps install. `docs/DESIGN.md` §20 is the
whole design, including what is deliberately not built (no silent install, no
background download, no version list, no downgrade, and no offer at all over
plaintext).

**The signing key is permanent** (operator ruling 2026-09-05). An Android
signing key is not rotatable, so there is one for this app's lifetime; it lives
outside every checkout, is backed up offline, and reaches CI as the repository
secrets `ANDROID_RELEASE_KEYSTORE` and `ANDROID_RELEASE_KEYSTORE_PASSWORD`. It
may not live in this tree and cannot — the disclosure gate refuses a
`.keystore` path outright. `scripts/sign-apk.sh` is the one recipe: zipalign,
then apksigner with **APK Signature Scheme v3** named and asserted, then a
verification the script fails on rather than merely printing.

**The one-time uninstall.** A phone that already holds a debug-signed build
cannot take a release-signed update: Android identifies an app by its signer as
well as its package, so the platform refuses to replace it in place. Uninstall
`dev.yog` on the phone once, then install again — the device's enrolment goes
with it and has to be re-landed. That is once, for the lifetime of the signing
key. `make deploy-phone` names that act when adb reports it, and it builds the
**release-signed** APK whenever the key is present on the box, precisely so a
phone deployed by hand is on the same signature as one that took an update.
A box without the key builds the debug APK exactly as before.

## Looking at it without a phone

`make screens` boots a headless emulator, installs the APK you built, walks the
app through its named screens and leaves a PNG of each in `target/screens/`
beside a verdict — so an agent can *see* this app, and a defect in how a screen
is REACHED can fail a check instead of waiting for someone's thumb. DESIGN §15
is the whole design; three things are worth knowing before running it:

- **It builds nothing.** Run `make apk ABIS=x86_64` first. A target that
  quietly rebuilt would hide which tree the pictures are of, which is exactly
  how this loop's first run lied to its author. It does *warn* when the APK it
  was given is older than the tracked source under `src/` and `android/` — a
  warning, never a refusal, because running the walk against a known-good
  artifact while the tree is mid-edit is a thing people do on purpose.
- **No engine is dialled.** A leaf is minted per run and the paint-first cache
  is seeded from the vendored wire `corpus/`, so every screen is reachable with
  no server anywhere.
- **Structure and reachability gate; pictures never do.** The app says which
  screen it painted (`src/shell/app/probe.rs`) and the walk judges that.
  Nothing compares images. The accessibility dump captured beside each PNG is
  still *empty* — egui paints into one opaque view, and the platform export
  that would fill it aborts this app (DESIGN §15.1) — so the reachability gate
  below reads a file the app writes instead.

## Invoking a tool for real: `make invoke`

`make screens` walks screens and dials nothing. The second loop does the
opposite: it boots a bounded engine (`yog fixture`), mints this device a
**foot**-grade leaf, seeds it, and then — from a seat that dials the listener —
fires four invocations and reads the captures back. It needs a `yog` on PATH
(`make invoke YOG=/path/to/yog` names one outright) and it is not part of `make
check`, for `screens`' reason doubled: an emulator *and* an engine.

Each of the four answers something a host test cannot produce: `shell` carries
back a value the run minted, `device` states a battery level the platform is
asked for a second way, `notify` leaves a row in the shade that `dumpsys`
reads, and `open` is refused **by Android** because the app is in the
background — every invocation is answered from the pocket, which is where a
teleoperated phone actually is. DESIGN §15.8 is the whole design.

Beside it, `make apk` now pins **every JNI name this crate resolves** against
the dex it just built, both directions — a renamed Java method is a build
failure instead of a `NoSuchMethodError` on somebody's phone — and, in the
third direction, every `native` the dex declares against the symbols the
packaged `lib/<abi>/*.so` actually exports, per ABI. A stale library married
to a current dex is the one build defect that kills the app outright, on every
launch, before it paints: it is a build failure now instead of
`UnsatisfiedLinkError` on somebody's phone (DESIGN §15.7).

## Interface parity: what this seat can reach

The desktop seat and this client must have **interaction parity** — not
identical placement, but if something is interactable in one it must exist in
the other, caught mechanically. yog's `docs/PARITY.md` is the contract; this
repo runs its client half.

Each seat is judged against **one roster**, never against the other client: the
engine's own help table, whose rows carry a `surface` class (`control` — every
seat owes this op a discoverable interactable; `machine` — spoken only by
programs), published inside the corpus this repo already vendors. Every
verb-firing control here carries an `act:<op>` tag, the app writes the tags it
painted to a debug-gated file in its own storage (the platform accessibility
export was tried first and aborts the process — DESIGN §15.1, exit ball
bl-a6f3), and the last beat of `make screens` reads them back out of the walk's
own output and judges four things: every `control` op is reached or exempted,
every tag names a real op, and no exemption has rotted or gone stale.

A deliberate absence is one line in **`parity.toml`** with a citation — a ball
that will build it, or the ruling that says it should not exist here. Deleting
the line re-reddens the gate and changes no code, and the roster prints on
every run, passing or failing. The half that needs no device — that every
exemption parses, cites, and still names a `control` op — gates on every `make
check`.

`make screens-avd` creates the virtual device, once. It may need an SDK licence
accepted, which is an operator's act and no target here performs it.

Task tracking is [balls](https://crates.io/crates/balls-cli) (`bl`): `bl prime
--as YOU`, `bl list`, claim → work in the worktree → close.

## How the gates hold

Four layers, one definition each, no drift by construction:

1. **Manifest lints** (`Cargo.toml [lints]`) — clippy pedantic at deny with a
   justified allow-list; the panic family and unchecked indexing denied. The
   manifest is the only home for a suppression.
2. **ast-grep** (`rules/*.yml`, pinned 0.44.1) — structural rules the type
   system misses: named lifetimes, borrow-returning `pub fn`, inline
   `#[allow]`, `assert!` in prod, locks outside `src/state.rs`, `Rc`/`RefCell`,
   bounds on `pub` generics. `make rules-audit` proves both directions: `src`
   clean AND every rule still fires on `rules/fixtures/`.
3. **Disclosure** (`make leak-scan`) — `scripts/leak-rules.sh` is the table of
   what may never be committed (keys, tokens, addresses, home paths,
   transcripts, session artifacts, unreadable binaries). It reads index
   blobs, self-tests per rule per line, and also gates every `bl` task-store
   publish and re-scans the published store daily
   (`.github/workflows/store-scan.yml`).
4. **CI** (`.github/workflows/ci.yml`) — `make ci`, which *is* `make check`:
   fmt + lint + 100% coverage. Actions pinned to commit SHAs.

The crate is `publish = false`: the deliverable is an APK, and a registry
release is a deliberate operator decision, not a reachable accident.

## License

MIT.
