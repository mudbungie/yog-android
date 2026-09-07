+++
title = "the release gate reads the engine's PROTOCOL from a path the engine no longer declares it at: the hold on 18 would never clear"
created = 1788754957
updated = 1788754958
claimant = "Cantaloups-G1"
priority = 1
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
tags = ["usability-r3"]
+++
`.github/workflows/release-plz.yml`'s `merge-release-pr` job fetches the
published engine's constant from a HARDCODED Rust path:

    raw "$ENGINE_REPO" "$engine_tag" src/wire/hello.rs "$gate/published"

The engine has since split that file. `src/wire/hello.rs` still exists but reads
`pub use version::PROTOCOL;` — the declaration moved to
`src/wire/hello/version.rs`. `protocol_of` anchors on
`^\s*(pub..)?const\s+PROTOCOL\s*:\s*u32\s*=\s*(\d+)\s*;`, and a `pub use`
matches nothing, so the fetch SUCCEEDS and the read finds no declaration.

`judge` holds on an unreadable published input, deliberately — a gate that could
not read its inputs has not answered. That is the right default and the wrong
answer here: the input is readable, at another address. So the moment the engine
publishes 18 — the event this hold is waiting for — the hold does not clear; it
becomes permanent, printing a sentence that reads as an engine defect rather
than as a stale path in this repository's own workflow. Filed against thrall as
bl-c618 and true here verbatim.

## The ruling (overseer, round 3)

The number gets ONE file-shaped home per repository that no split can move: a
top-level `PROTOCOL` file, one line, the integer. `build.rs` compiles it into
the vendored constant (`include!` of a generated const), so the file IS the
source and there is no second copy to test against. `scripts/protocol-gate.sh`
and the workflow read `PROTOCOL` at the repo root of every tree and tag they
judge — **no Rust path in any gate or roster, in any of the four repositories**.

A path LIST was the obvious repair and is refused: it is config that grows on
every engine refactor, and a gate that tries several places and takes the first
hit can be satisfied by a file that is no longer the authority.

## Here

- `PROTOCOL` at the root, stating this client's vendored number; `build.rs`
  reads and validates it and writes the constant `src/hello.rs` includes.
- `scripts/protocol-gate.sh`: one path constant for both reads, self-test
  rewritten over integer-file fixtures, and the unreadable-engine sentence names
  THIS repository's expectation rather than the engine's file.
- `.github/workflows/release-plz.yml` fetches `PROTOCOL` at both roots.

The engine's half is yog bl-3e57; the other consumers are thrall bl-c618 and
lernie bl-55b1. Landing this does not change the number, only where it is
written: whichever lane is vendoring 18 bumps `PROTOCOL` instead of the `const`.