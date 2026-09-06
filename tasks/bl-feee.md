+++
title = "the workflow selector: third control under the composer, gated on the boundary speaking litany's workflow verb"
created = 1788317686
updated = 1788659888
priority = 3
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
GATE NOT MET at yog PROTOCOL 13 — checked against the tree, not the ball body.

`grep -rn Workflow src/boundary/` on yog finds exactly one thing: `ConfigFile::LitanyWorkflow { name }`, a DESTINATION of the `config` op (`src/boundary/config/file.rs`, `codec/config.rs`, `line/config.rs`). That is a file write of `workflows/<name>.yaml` by name. It is not litany's per-agent workflow MARK and it cannot stand in for one. There is no `workflow` op token in `corpus/request/`, no `workflow` reply kind in `corpus/reply/`, and no `Query`/`Action` variant for either half.

Three things this seat would need before a selector is anything but invention:

1. A READ enumerating the named workflow configs a workspace offers. `config` reads ONE file at a destination the client must already name, so a selector would have to guess names. Nothing lists `workflows/*.yaml` in the governing tree.
2. A READ of which workflow governs a conversation NOW, and by which of the two routes — the per-agent mark, or the followed tip. `governing` answers a config COMMIT, not a workflow mark, so the "mark vs tip" distinction the ball asks the selector to show is unaskable.
3. An ACT setting or clearing the mark — litany's `workflow <ws> <agent> [--config <name> | --clear]`, unexposed at the boundary.

Until those exist the honest client-side answer is nothing at all: a control that listed names off a directory it cannot read, or set a mark through a verb the boundary does not speak, would be the decoy shape DESIGN §16.1 refuses. Left open, gated on yog.
