+++
title = "REMOTE §8.2 entries: this device as a client of many engines"
created = 1788139112
updated = 1788658597
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

---

**Read against the tree on 2026-09-05 (Animations-AD), and the two premises hold.**
`src/material.rs` still reads the FLAT directory — `WANTED = [ca.pem, client.pem,
client.key, address]` in one directory the shell hands it — and nothing in this
repository knows the word `workspaces/`. So the ball's own statement is still
exact: zero entries is byte for byte what came before, and this is not a
deviation.

**It is not blocked on upstream and it is not a codec ball.** Every shape it
needs is already spelled here, and none of the five surfaces landed since
(§13.15–§13.18) moved anything under it. What it is blocked on is TWO RULINGS
this app cannot take for itself, and both are about what a phone IS rather than
about how a client dials.

**Ruling 1 — does one device still mean one leaf?** DESIGN §13.3 states the
vocabulary rule as *"One device, one name, one leaf: the grade is what a leaf
may say, never a second registration"*, and `crate::bootstrap` derives the
running component FROM that leaf — *"the component is derived, never stored,
and that is the whole design"*. An entry set is N leaves on one device, and
nothing says they agree: a phone could be operator-grade on one engine and
foot-grade on another, at which point `Running` is no longer one answer and the
first-run derivation has no single fact to read. Three shapes are available and
the operator owns the choice:

- **(a) One grade, N addresses.** Every entry must carry a leaf of the same
  grade as the flat one; a mismatched entry is refused at read time, naming
  both. `Running` stays one answer and §13.3 stands unamended. Cheapest, and it
  forbids the case an operator might actually want (a seat here, a tool host
  there).
- **(b) N components, one process.** `Running` becomes a set: the app is a seat
  on the entries whose leaf is operator-grade and a foot on the rest, with one
  host thread per foot entry (§18.1's process-owned host becomes N). This is
  what REMOTE §8.2 literally describes, and it is the expensive one — §18's
  pocketed foot holds ONE foreground-service lane today, and N lanes is a
  battery cost §17 and §18.4 have operator rulings about.
- **(c) The seat is N and the foot is one.** Entries are a SEAT-side fact only;
  the foot keeps the flat material and its one lane. The roster unions N
  engines, and this device offers its tools to exactly one of them. It costs
  nothing on the battery, it matches how the phone is actually carried, and it
  states a limit rather than hiding one.

**Recommendation: (c).** The want behind this ball is *read and drive several
engines from one phone*, and that is entirely a seat want — the roster, the
conversation lists, the transcript, the gestures. Offering this device's camera
and shell to N engines is a different want with a different price, and nothing
has asked for it. (c) also leaves (b) reachable later without undoing anything:
the foot's single lane becomes one of N the day someone pays for it.

**Ruling 2 — where does a second entry come from on a phone?** DESIGN §5's
three delivery channels all land material in ONE flat directory, and the
enrollment screen (§11, `shell/enroll/material.rs`) is shaped as *land it and
re-derive*. An entry needs a NAME before its material can be written, and the
envelope does not carry one — REMOTE §8.2 is explicit that `<leaf>` is the
CLIENT's name for the workspace and that a collision's remedy is a local
rename. So the enrollment screen grows a field it has never had, and the first
question a scan asks stops being *is this material good* and becomes *what
shall I call this engine*. That is a change to the one surface a cold device
lands on, and it is the operator's call whether a second engine is worth it.

**What the work is, once both are ruled.** With (c) and a named-entry
enrollment: `material::read_dir` gains an entries walk (four files, one level
down, plus the optional `workspace` file — and that file stays UNWRITTEN until
the rename mapping has a consumer, which is §8.2's own instruction); the seat
holds N `Seat`s and one `Standing` per channel; `Snapshot` rows gain a
client-side channel stamp that crosses no wire; `seat::asks`/`acts` route by
the channel the addressed workspace came from; the follow and attention lanes
dial per channel (§14.1 becomes N lanes, which is the one place the phone's
radio budget is actually spent and wants measuring); and a channel that will
not dial paints ITS workspaces unreachable rather than reddening the shell.
`parity.toml` is untouched — entries are §13.4's one true upstream-ask class,
an absence no roster can see, and they stay that way.

**Left open deliberately**, per the dispatch instruction: it wants a ruling, and
the recommendation above is this agent's, not a decision.
