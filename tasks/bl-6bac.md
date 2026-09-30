+++
title = "the pre-commit gate collapses to exec bl-gate: scripts/check carries the three-word exit, make check is its door, the laptop never compiles in a gate (ops bl-3166)"
created = 1790735904
updated = 1790736500
claimant = "Junketing-yoga"
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
Phase 2 of the shared-gate rollout (ops bl-3166, remote-builds.md 'Phase 2'). .githooks/pre-commit keeps the mainline refusal and execs bl-gate (userconf). scripts/pre-commit is deleted; scripts/check (yog pattern) = make fmt-check lint && exec scripts/check-coverage.sh, and make check runs it. AGENTS.md/README say the laptop does not compile in the gate, tarpaulin/llvm-cov are shimmed here, bl-remote-run <target> runs a make target on the builder.

---

Tree is staged in the worktree, unclosed. Hook, scripts/check, Makefile check target, docs all done. The builder FAILED the tree twice (out/7470d95…, out/66dc25f…): 18 then 21 timing tests (dht::tests::* with 300 ms Config.deadline; ladder::tests::held) — the same tree passes plain make test on the builder (922 ok in 47 s) and its parent main passed GitHub CI at 12:20. Under tarpaulin on a builder now running 3 concurrent slots (load 59 on 40 cores) the unit binary took 320 s then 595 s, and any >300 ms stall between ask and land turns the stub's socket error into 'no DHT node answered'. Infra, not the tree; two signed FAIL verdicts for tree 12f1cf7 exist on the builder (local copies deleted). Retry needs the hook re-seated: ln -sfn <worktree>/.githooks/pre-commit .git/hooks/pre-commit (main's seated hook still execs the deleted scripts/pre-commit until this closes).
