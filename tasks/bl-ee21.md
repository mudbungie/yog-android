+++
title = "re-vendor the wire corpus: reply/follow gained the tool window and reply/ops gained the client that asked"
created = 1788675545
updated = 1788746276
claimant = "Cantaloups-A4"
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
tags = ["usability-r1"]
+++
yog round-1 fix lane Y5 moved two wire-visible shapes. The android client vendors `corpus/` and replays it against its own encode and decode (REMOTE §3), so this is the re-vendor plus the two client-side renderings the new fields exist for.

REMOTE §3 is the protocol authority and yog `src/wire/hello.rs` carries the version; the two moves are recorded at REMOTE §5.5 and §9.18.

## 1. `reply/follow` gained `tools` (yog bl-5305)

The follow lane carried the model prose alone, so an operator watching an agent
administer their machines saw a thinking marker and nothing about the commands.
The frame body now carries the **tool window** beside the fold, appended by the
same rule the prose obeys — concatenate the lists of a read frames in order:

    {"tool_use": "toolu_01", "tool": "box2_Bash",
     "input": "{\"command\":\"hostname && uptime\"}"}
    {"tool_use": "toolu_01", "exit_code": 0}

- Two entries per call, both transitions.
- `exit_code` **presence** is the status; absent is a call in flight.
- The closing entry restates neither name nor input — key it by `tool_use`.
- The machine a routed call ran on is already in `tool`: REMOTE §5.1 presents a
  loaded remote tool as `<client>_<tool>`, always.
- `tools` is **required**, empty list included, so a decoder must not treat
  absence as empty.

The lane liveness also widened: a follow read follows a STEP rather than the
model call inside it, so it stays open through the tool phase.

## 2. `reply/ops` rows gained `client` (yog bl-e59e, round-1 ruling 5)

Every trail row names the identity that made the act — a connection certificate
common name, or `local` for in-world callers. Read strictly, like `standing`.

## What this ball asks

1. Re-vendor `corpus/` and make the conformance replay pass on both shapes, in
   both directions.
2. The follow surface shows what is running, where, and what it came back with.
3. The ops surface names the author.
4. Record both in `docs/PARITY.md` (or its android equivalent) if the exemption
   ledger tracks per-op surfaces.