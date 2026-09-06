+++
title = "fork needs a fork point: the picking surface over a conversation's history"
created = 1788405992
updated = 1788656855
claimant = "Animations-AD"
priority = 3
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"

[[blockers]]
id = "bl-146b"
on = "claim"
+++
The fourth act of bl-f97c's roster, held back when the other three landed on the
conversation row's long-press menu (DESIGN 13.5).

The wire has the op and this client can spell it: `{op: fork, workspace, parent,
from, role, goal, skills[]}`, answered `outcome`. What it cannot supply is
`from` — the fork point. The engine's own `fork::Attempt` states the rule: "Empty
is not a value ... a fork with no ref is a different gesture." A fork point is
either a commit of the conversation's own history (a pinned mark) or a
`config/<name>` head for a clean start.

Neither is nameable from this seat today. The marks and the tip ride the `agent`
read (bl-146b, unbuilt); the lineage names ride `lineages` (bl-3685, unbuilt). A
free-text field where an operator types a commit sha on a phone is not a surface
— it is this app asking the operator to be the read DESIGN 8 forbids it to
derive — so the menu offers three items and not four, and `fork` keeps its
`parity.toml` line re-cited here.

What closing this needs, in order:
- the read that names the points: bl-146b's `agent` read carries `marks` and
  `tip`, which is the picking surface's whole content for the history half. The
  `config/<name>` half wants bl-3685's `lineages`; a first cut may offer only
  the history half and say so.
- `RowAct` gains a `Fork` variant in `src/codec/row.rs` (the group's one home)
  or a shape of its own if the picking surface makes it a different gesture —
  attack that before building: fork's subject is a POINT in a conversation, not
  the conversation, which may mean it belongs on a transcript row rather than a
  conversation row.
- `role` is `worker`, this seat's one role (`seat::acts::WORKER`), and `skills`
  is `[]` until a skills read exists.
- the goal is the composer's text, as the other parameterised row acts take it.
- `tests/conformance/requests.rs` moves `fork` from `Refuses(NO_FORK_POINT)` to
  `Reads`, and `parity.toml`'s `fork` line is deleted.