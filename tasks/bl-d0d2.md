+++
title = "REMOTE §8.2 entries: this device as a client of many engines"
created = 1788139112
updated = 1788139112
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
REMOTE §8.2 (operator ruling 2026-08-24): *"a client can be a client of many
servers, and the client-side workspace is what names one — its own mTLS
material, its own address, its own conversations, separate everything."* An
entry is a directory:

    <yog-data-root>/wire/workspaces/<leaf>/
        ca.pem        the HOST engine's anchors — that operator's trust root
        client.pem    this box's leaf for this workspace
        client.key
        address       the host engine, host:port — "the server", entire
        workspace     OPTIONAL: the name this workspace bears on its host, when
                      it differs from <leaf>; absent, the leaf is the name

This device reads the **flat** directory today, which is not a deviation:
§8.2 rules that *"the flat directory therefore remains what it has always been,
the box's own root … Everything beyond the box's own engine is an entry."*
Zero entries is the whole of what a one-engine phone needs, and §8.2's own
window half says zero entries is byte for byte what came before.

What this ball buys is the phone reaching a second engine, and it is not a file
format — it is N of everything this app runs on one:

- **N standing-question sets, unioned into one roster**, each row stamped with
  the channel it came from — a client-side stamp, so no origin crosses the wire.
- **Name resolution over the union**, with a collision refusing in place of the
  answer and naming the remedy (rename the entry, never the workspace on its
  host).
- **The rename mapping spent at exactly one place**, the channel boundary, in
  both directions: a gesture crossing carries the host's name, a reply landing
  is labelled with the leaf. Adding the `workspace` file before that mapping has
  a consumer would be a field nothing spends.
- **N foot channels**, because a tool host has no name to resolve: §8.2 is
  explicit that an entry adds it *"a second engine to be present at, never a
  name to rewrite"*, one thread per channel, execution serial within each.
- **Per-channel refusal**, never the whole shell: a channel that cannot be
  dialled is that channel's workspaces painted unreachable.

The upstream window has landed all of this (§8.2's bl-4e31, bl-028a, bl-670c
notes), so the shapes are decided and this is the phone's own half.