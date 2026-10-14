+++
title = "the runner's coverage gate reads one small function uncovered on roughly half its runs with no tree change: src/dht.rs:172 (the body of dht::random) on four runs today, src/rendezvous.rs:131 on two before it; the same trees pass on the builder and on the next runner run"
created = 1791941440
updated = 1791941865
priority = 3
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
Four Release-plz runs on 2026-10-13 failed 'make ci' at 99.99% with the one uncovered line src/dht.rs:172 — the body of dht::random, which every DHT id, mutable seed and rendezvous nonce calls, so no test order can skip it. Two earlier runs the same morning failed on src/rendezvous.rs:131 instead. Between them the same trees passed on the runner (v0.0.29–31 released) and the builder's gate passed every one of them. Same rustc on pass and fail. The shape is tarpaulin's llvm-engine merge, not a missing test: a small function compiled into more than one test binary carries an unexecuted record from the binaries that never call it, and which record wins the merge is not ordered. Decide: a second coverage pass in make ci before failing, or pinning the engine the builder runs, or an upstream tarpaulin fix. Until then a held release is re-judged with a workflow_dispatch of Release-plz.