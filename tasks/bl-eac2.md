+++
title = "design: the phone is a full seat and a thrall-class foot — the teleoperation tool corpus and the ledger re-scope"
created = 1788398874
updated = 1788398874
priority = 1
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
OPERATOR RULINGS (2026-09-03): (1) the phone's role is lernie AND thrall — a full seat plus a foot, not a chat-first companion; (2) working teleoperation tools on the phone are wanted — android tool development is in scope in this chain. This is a DESIGN ball: the deliverable is a tracked design doc in this repo (or an amendment to the authoritative doc if that is DESIGN sec 13 / yog REMOTE — decide and say why), plus implementation balls filed where the design settles. Do not implement here.

Two halves:

A. THE FOOT: a teleoperation tool corpus served by the phone. The phone as thrall-class foot advertises phone-local tools an agent can invoke over the tool-host channel — the candidate space is what makes a phone a phone: notifications, camera, location, clipboard, share-sheet, SMS-adjacent surfaces, device state. Attack scope hard: which tools are wanted for TELEOPERATION (operating through the phone from elsewhere), what each costs in android permissions and background execution, and what the consent surface is (thrall's advertised-set + subject_cwd consent model is the pattern — read thrall's design in its own repo, read-only). RECONCILE WITH THE STANDING RULING bl-5710 ('no tool corpus ships') — read that ball and its reasoning first; if this design reverses or narrows it, the amendment must cite the operator ruling above and fix the doc that states it. Also read the trap ledger: background execution and restricted settings are known hazards.

B. THE SEAT: re-scope the parity exemption ledger under the full-seat ruling. The ~43 exemptions citing DESIGN sec 2's chat-loop-slice are written as a scope fence; under the full-seat ruling they become UNBUILT (each needs a citable ball) or per-platform-never (each needs a stated reason that survives the ruling). Produce the re-cited parity.toml plan and file the surface balls in sensible groups — mirror the seat's grouping where it fits (conversation acts, tuning, ball pane, candidates, fleet) and order them behind the teleop corpus where they contend.

Verify every premise against the tree and stores; cite what you read.