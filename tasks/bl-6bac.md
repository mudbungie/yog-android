+++
title = "the pre-commit gate collapses to exec bl-gate: scripts/check carries the three-word exit, make check is its door, the laptop never compiles in a gate (ops bl-3166)"
created = 1790735904
updated = 1790735905
claimant = "Junketing-yoga"
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
Phase 2 of the shared-gate rollout (ops bl-3166, remote-builds.md 'Phase 2'). .githooks/pre-commit keeps the mainline refusal and execs bl-gate (userconf). scripts/pre-commit is deleted; scripts/check (yog pattern) = make fmt-check lint && exec scripts/check-coverage.sh, and make check runs it. AGENTS.md/README say the laptop does not compile in the gate, tarpaulin/llvm-cov are shimmed here, bl-remote-run <target> runs a make target on the builder.