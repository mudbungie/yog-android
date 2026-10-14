+++
title = "the runner's coverage gate reads one small function uncovered on roughly half its runs with no tree change: src/dht.rs:172 (the body of dht::random) on four runs today, src/rendezvous.rs:131 on two before it; the same trees pass on the builder and on the next runner run"
created = 1791941440
updated = 1791942754
claimant = "CoverageLane"
priority = 3
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
Four Release-plz runs on 2026-10-13 failed 'make ci' at 99.99% with the one uncovered line src/dht.rs:172 — the first line of dht::random's three-line body, which every DHT id, mutable seed and rendezvous nonce calls. Two earlier runs the same morning failed on src/rendezvous.rs:131 instead. Between them the same trees passed on the runner (v0.0.29–31 released) and the builder's gate passed every one. Same rustc and the same tarpaulin pin on pass and fail.

The mechanism is in tarpaulin 0.35.2, src/statemachine/instrumented.rs. The first test binary processed creates one trace per coverable line with that binary's hits. Every LATER binary looks each line up in its own report and ASSIGNS: 'if let CoverageStat::Line(ref mut x) = trace.stats { *x = hits as _; }' — not added, not maxed. And the lookup, llvm_profparser's hits_for_line, is '.find' over a HashMap of regions, so when more than one region spans the line the one returned is not ordered. So a line's final count is whatever the LAST binary that knew the line said, and binaries are processed in build-completion order, which on a 4-core runner is not stable; the 40-core builder lands the lib's own test binary last every time and never sees it. A multi-line expression is the exposed shape because tarpaulin's source analysis folds it to its first line: the function's other lines are not coverable, so the one folded line is the only one a zero can land on.

Remedies, none taken yet: patch upstream (assign becomes max), pin a patched tarpaulin build, or make the build order deterministic (one job). Until one lands, a held release is re-judged by a workflow_dispatch of Release-plz, which clears on roughly every other attempt.