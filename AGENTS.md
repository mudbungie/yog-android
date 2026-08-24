# yog-android — Agent Operating Guide

You are working in **yog-android**, a single Rust crate: the Android seat for
yog. Two authorities govern your work and they do not overlap:

- **`docs/DESIGN.md` is the architecture authority** — what this client *is*,
  its invariants, and the wire contract it mirrors. The server side's
  authority is yog's `docs/REMOTE.md`; where the two collide, REMOTE wins
  until REMOTE is amended. Never implement a deviation silently — fix the doc.
- **This file is the code-style authority** — the machine-enforced rules and
  the repo discipline around them.

The point of every gate here, stated once: **nothing publishes that should
not.** This repo is public from birth and its task store publishes beside it;
yog's 0.0.1 shipped an operator's home paths and had to be yanked, and every
rule below exists so that class of accident cannot recur.

## Code-style rules (Rust Bootstrap v3, adapted)

Contained Rust: complexity lives in function bodies, not type signatures.
Prefer clones / `Arc` / `Box<dyn Trait>` / enums over borrow-based APIs.
Machine-enforced by ast-grep (`rules/*.yml`), the manifest (`Cargo.toml
[lints]`), and cargo-deny.

1. **No named lifetimes.** `'static` and `'_` are fine. Borrow in (elided),
   owned out. Enforced: `rules/no-named-lifetimes.yml`.
2. **A `pub fn` returns an owned, concrete type** — never `&T`/`&mut T` nor
   `impl Trait`. Demote internals to `pub(crate)` rather than clone-to-own.
   Enforced: `rules/no-pub-borrow-return.yml`.
3. **No `unsafe`** — `unsafe_code = "forbid"` in the manifest. The Android
   shell may one day need FFI glue; that ball replaces `forbid` with a
   location rule confining `unsafe` to one named file (yog's precedent),
   deliberately and with the soundness argument written in that file.
4. **No panic paths outside tests.** unwrap/expect/panic!/todo!/
   unimplemented!/dbg! and unchecked indexing/slicing are `deny` in the
   manifest; the whole assert family — `debug_assert!` included — is banned
   in prod (`rules/no-assert-outside-tests.yml`, bl-383b: with no unsafe to
   guard and a 100% floor, a debug assertion is just a panic a test finds
   the hard way). Tests get carve-outs via `clippy.toml`.
5. **No `#[allow]` in prod.** Policy lives in `Cargo.toml [lints]`, justified,
   one line each. Test code may relax a lint. Enforced:
   `rules/no-lint-suppression.yml`.
6. **Zero new dependencies without explicit operator approval.** Registry
   pins only; no git deps, never a `path` dep. TLS is rustls-only with
   `ring` — `deny.toml` bans `openssl-sys`, `native-tls` AND `aws-lc-sys`, and
   the license allow-list is exhaustive over the committed lockfile. The
   crate is `publish = false`; making it publishable is an operator decision,
   not a chore.
7. **`Mutex`/`RwLock` only in `src/state.rs`** (create it when the first lock
   is needed); no `Rc`/`RefCell` anywhere, tests included. Enforced:
   `rules/locks-outside-state.yml`, `rules/no-rc-refcell.yml`.
8. **No async, no tokio today.** The wire is synchronous by design upstream
   (the server's listener is `std::net`). Do not add tokio to anticipate.
9. **No trait bounds on a `pub` item.** Trait object or concrete param;
   demote bounded helpers. Enforced: `rules/no-pub-generic-bounds.yml`.
10. **`thiserror` when an error enum earns it; `anyhow` never.**
11. **One crate.** The module tree plus the 300-line cap contain complexity;
    no `[workspace]`.

"pub" for rules 2 and 9 means bare `pub` — the surface `tests/` and a future
shell consume. `pub(crate)` is the honest demotion and the rules skip it.

## Repo discipline

- **Task tracking is `bl`.** Run `bl skill` before using it; session start is
  `bl prime --as YOUR_IDENTITY`, then `bl list`. Claim → work in the printed
  `work/<id>` worktree → close. Never edit `main` directly; always pass
  `--as`.
- **300-line hard cap on every source file**, inline tests included; docs and
  config exempt. `make line-cap` is the one definition; anything projected
  ≥200 is pre-split at design time (`make line-cap LINE_CAP=199` lists the
  band). Over the cap: split along a real seam, never shave lines.
- **100% test coverage**, tarpaulin pinned 0.35.2 (`tarpaulin.toml`). If it
  can't be tested, it mustn't be built. Coverage exclusions are added with
  reasoning, never to make a number.
- **`make check` is the complete local gate and mirrors CI exactly:**
  fmt-check → lint (line-cap + leak-scan + clippy + rules-audit + cargo-deny)
  → coverage. The pre-commit hook (`make install-hooks`, once) runs the same
  scripts. Tool pins: rustc 1.95.0, ast-grep 0.44.1, cargo-deny 0.20.2,
  tarpaulin 0.35.2 — bump only deliberately, and in lockstep with CI.
- **The disclosure gate** (`make leak-scan`) reads INDEX blobs and self-tests
  its own rules per line before every scan. `scripts/leak-rules.sh` is the one
  definition of what may not be committed: private keys, vendor tokens,
  credential assignments, routable addresses, MAC addresses, home paths
  (synthetic roots `/home/u`, `/home/op`, `/home/x` pass), personal emails
  (the maintainer's `mudbungie@gmail.com` is the one permitted identity),
  quoted dialogue, session artifacts, credential-shaped paths, unreadable
  binaries. There is no allowlist and no path exemption. Fix the rule, not
  the coverage. `.githooks/commit-msg` runs the same scanner over commit
  messages.
- **Never credit AI or tooling** in commit messages, code, or docs.

## What may never enter a ball body

`bl` keeps tasks on `balls/tasks`, pushed to the SAME remote — a ball body is
published text. The machine-global `bl-leak-gate` plugin runs this repo's own
scanner over every publishing op's commit (shipping `scripts/leak-scan.sh` is
the opt-in), and `.github/workflows/store-scan.yml` re-judges the published
ref daily. The gate catches the mechanical half; the editorial half is yours:

- No third-party names, handles, or addresses. The maintainer's own
  `mudbungie` identity is permitted (already public in LICENSE and the
  manifest); every other identity is a leak.
- No verbatim transcript prose — cite the conclusion and the ball id, never
  the exchange.
- No live machine state: pids, absolute home paths, host/device names,
  workspace names off a live world. Cite the shape (`/home/u`), not the
  instance.
- No provider auth state quoted from a provider.
- No conversation or session ids.

## Before anything ships beyond the repo

The gate scans one tree at its tip. Release text, Actions logs, other refs,
and anything already published are outside it — yog's AGENTS.md carries the
full publication checklist ("Before making the repo or a crate version
public") and its lessons apply here verbatim. Two standing differences: this
repo was public from its first commit with the gate in place (no private
history to certify), and `publish = false` makes a registry release
impossible until an operator deliberately reverses it. An APK release channel
gets its own checklist ball before the first APK leaves this box.
