+++
title = "the runner's coverage gate read one line uncovered on every run of some trees: the gate ran ptrace, because tarpaulin ignores --engine when tarpaulin.toml exists"
created = 1791941440
updated = 1791944148
claimant = "CoverageLane"
priority = 3
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
The runner's coverage gate failed at 99.99% on one line of a multi-line chain (src/dht.rs:172, src/rendezvous.rs:131 before it) on every run of the affected trees, while the builder passed every one. Same rustc, same tarpaulin pin.

## Cause

The gate never ran the llvm engine. tarpaulin 0.35.2 ignores `--engine` on the command line whenever `tarpaulin.toml` exists (the file/flag merge never copies it; lernie bl-11da found the same), so the Makefile's `--engine llvm` was dead and every gate ran ptrace, the Linux default. Proven on both machines: no profraw written, the test binaries carry no `__llvm_profile_runtime`, and the reported hit counts are breakpoint-shaped (1) where llvm counts 42.

Under ptrace the coverable set comes from each binary's DWARF line table. On the runner, a `--lib`-only run had no row for dht.rs:172 at all (36/36 coverable in that file), while an integration binary that links but never calls `dht::random` did carry it, at zero hits. tarpaulin sums per-binary traces, so the line read 0 on the runner and covered on the builder. Why the lib binary's line table differs between the two machines was not established; it is ptrace's problem and leaves with ptrace.

The earlier hypothesis in this body (per-binary assignment in `statemachine/instrumented.rs`) is wrong on two counts: that file is the llvm engine, which was not running, and per-binary results are combined by `TraceMap::merge`, which adds.

rustc's own regions were checked with llvm-cov on both machines: identical, dht.rs:172 at the true count.

## Fix

- `engine = "Llvm"` in `tarpaulin.toml`, with the reasoning in its header; the dead `--engine llvm` flag removed from the Makefile; README notes it.
- llvm is the stricter floor: it judged 7319 lines (ptrace 6700) and found five uncovered. Each is answered, none excluded:
  - `dht::transport::elapsed`'s false arm: a unit test with a non-timeout error.
  - `leaf::attributes`' attribute-without-value skip: a hand-built Name, since no minting recipe writes that shape.
  - `transport::open::each`'s ping skip: covered only when the ladder test's race went one way; now a direct test serves a ping ahead of the reply.
  - `test_support::serve` Feed: the write-error early return became `try_for_each(..).and_then(write_end)`, same behaviour, no timing-dependent branch.
  - `state::Watched::wait`'s bare `loop {` line: rustc maps no region to it and tarpaulin force-covers every line of a generic body, so it read 0 forever. Now `let answer = loop { .. break answer .. }; drop(held); answer`, which carries a region (the `drop` keeps clippy's let_and_return quiet and releases the lock before the answer leaves, as before).

No retry, no exclusion, floor unchanged at 100%. Builder gate and the 4-core runner both pass the tree.