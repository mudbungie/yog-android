+++
title = "the merge job is push-only, so a held release has no re-judge door once the engine publishes"
created = 1788756431
updated = 1788756432
claimant = "Cantaloups-G2"
priority = 1
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
tags = ["usability-r3"]
+++
Measured 2026-09-06. yog published v0.0.53 at PROTOCOL 18, carrying the root
`PROTOCOL` file. this repository's release pull request (v0.0.4) stood open and
CLEAN — every guard would now pass, the protocol gate included. A
`workflow_dispatch` of Release-plz SKIPPED the `merge the release PR` job, so
nothing merged it.

WHY. The job carries `needs: [ci, release-plz-pr, release-plz-release]` and a
bare `needs` carries an implicit `success()`, which skips the job when ANY
dependency is SKIPPED. `release-plz-pr` is push-only
(`github.event_name == 'push'`), so on a dispatch it is skipped and this job
skips with it. The file's own header states that as a feature: "The dispatched
run cannot re-enter this job: `release-plz-pr` is push-only, so on a dispatch
it is SKIPPED and every `needs:` of it skips with it."

That is right about the POST-MERGE dispatch, where re-entry buys nothing, and
wrong about the case that actually happens. Guard 6 — the protocol gate — is
the only guard whose answer changes with nothing in this repository changing:
it clears when the ENGINE publishes, an event this repository never sees. So a
held release waits for an unrelated landing on `main` instead of for the
condition it is actually held by.

yog's `release-automerge.yml` has the door and names it in the comment it
leaves on a held pull request: a `workflow_dispatch` of that workflow "is the
door when the fix landed elsewhere."

THE FIX. The merge job runs on `workflow_dispatch` too, re-judging the open
release pull request against the newest published yog tag. `needs:` keeps the
ordering; the gate moves into an explicit `if` that names results rather than
inheriting them — `ci` and `release-plz-release` must have SUCCEEDED on both
triggers, and only `release-plz-pr` may be `skipped`, which is exactly what a
dispatch does to it and nothing else does. A failed PR job still skips this
one. Guard 5 is unaffected: release-plz rebases the release branch onto `main`
on every push, so the head's first parent is `main`'s tip, which is
`github.sha` for a dispatch on `main`. The header is corrected in the same
edit: the recursion stop is no longer this job's unreachability but the fact
that a dispatched run refreshes no pull request and the merge step dispatches
only when it merged something.