# yog-android

The **phone seat** for [yog](https://github.com/mudbungie/yog): an Android
client of yog's remote boundary — a wire client that dials the engine over
mTLS and speaks `Act | Ask` frames in, `Reply` frames out. Agents run on the
server; seats attach and detach, and the work does not.

The wire contract is the server's (yog `docs/REMOTE.md`): big-endian `u32`
length-prefixed JSON frames, a zero-length terminator, mTLS with
operator-provisioned certificates, and the client always the asker. This
repo's `docs/DESIGN.md` records what the client is and mirrors; `AGENTS.md`
is the working discipline.

## Status

Client standing, seat model pending. The frame layer, the chat-loop codec
slice, mTLS transport, key material, and the Android shell (egui via
GameActivity — DESIGN §3) are landed; the seat view model is the tracked
next ball. On-device verification is operator-assisted; an unregistered
leaf correctly sees empty rows.

## Dev loop

```
make check          # the complete local gate == CI == pre-commit hook
make test           # cargo test
make coverage       # tarpaulin, 100% floor (pinned 0.35.2)
make lint           # line-cap + leak-scan + clippy + ast-grep + cargo-deny
make apk            # cargo-ndk (aarch64) + gradle assembleDebug
make install-hooks  # seat the pre-commit / commit-msg hooks, once
```

The APK build needs the Android NDK, `cargo-ndk`, the
`aarch64-linux-android` target (pinned in `rust-toolchain.toml`), and a
**system gradle** (8.7+, JDK 17). There is deliberately no gradle wrapper:
the wrapper is a committed jar, and the disclosure gate refuses any binary
it cannot read — correctly. The release profile is load-bearing, not an
optimization (see the Makefile `apk` target).

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
