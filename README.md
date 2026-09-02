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
make screens-avd    # create the emulator the loop below boots, once
make screens        # headless emulator: walk the screens, capture each one
make install-hooks  # seat the pre-commit / commit-msg hooks, once
```

The APK build needs the Android NDK, `cargo-ndk`, the two Android targets
pinned in `rust-toolchain.toml`, and a
**gradle** 8.7+ on JDK 17 — `make apk GRADLE=/path/to/gradle` when it is not
on `PATH`. There is deliberately no gradle wrapper:
the wrapper is a committed jar, and the disclosure gate refuses any binary
it cannot read — correctly. The release profile is load-bearing, not an
optimization (see the Makefile `apk` target).

One APK carries **both ABIs**: `arm64-v8a` for the phone and `x86_64` for the
emulator the enrollment stories run on. Gradle packs a `jniLibs/<abi>/`
directory per ABI and the installer picks one, so there is no
emulator-only artifact to confuse a test verdict. Override with
`make apk ABIS=arm64-v8a` on a box that only ever flashes a phone.

## Looking at it without a phone

`make screens` boots a headless emulator, installs the APK you built, walks the
app through its named screens and leaves a PNG of each in `target/screens/`
beside a verdict — so an agent can *see* this app, and a defect in how a screen
is REACHED can fail a check instead of waiting for someone's thumb. DESIGN §15
is the whole design; three things are worth knowing before running it:

- **It builds nothing.** Run `make apk ABIS=x86_64` first. A target that
  quietly rebuilt would hide which tree the pictures are of, which is exactly
  how this loop's first run lied to its author.
- **No engine is dialled.** A leaf is minted per run and the paint-first cache
  is seeded from the vendored wire `corpus/`, so every screen is reachable with
  no server anywhere.
- **Only structure gates.** The app says which screen it painted
  (`src/shell/app/probe.rs`) and the walk judges that. Nothing compares
  pictures. The accessibility dump captured beside each PNG is *empty* — egui
  paints into one opaque view — and that is a finding recorded in evidence,
  not a gap in the harness.

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
