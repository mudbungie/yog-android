# yog-android — Style

The visual language of the phone seat, and its one home (bl-549b, operator
request 2026-09-06: *a more modern look — fewer boxes, soft neon, an
interface that makes sense from its layout*). The desktop seat cites this
file and carries only its own deltas (lernie bl-73d2); the server has no
face. `docs/DESIGN.md` says what each screen IS; this file says what every
screen LOOKS like, and it is normative: a screen that departs from it is a
defect, and the fix is here or in `src/theme.rs`, never on the screen.

## 1. The three rules

1. **Colour means state.** A hue on the glass says what a thing is DOING and
   nothing else. Six states, six colours, no seventh (operator ruling
   2026-09-06, §3). Hierarchy — what is a title, what is a row, what is a
   detail — comes from spacing, type size and alignment, never from colour
   and never from a border.
2. **No outlines.** Nothing on the glass is boxed. A block is raised from the
   ground by a tint (§2, the elevation ladder); rows are separated by
   spacing and at most a hairline; a control at rest is bare ground and
   tints only under a thumb. The one stroke that remains is the brand focus
   ring on the field that holds the caret.
3. **One module.** Every colour, size and gap on the glass is read from
   `src/theme.rs` — the palette as bytes, the state accents, the spacing,
   type and touch scales — through `src/shell/theme.rs`, the one adapter
   into egui: it installs the tokens as egui's `Visuals` once at app start
   and answers a `Color32` by the NAME of a state at a paint site. No screen
   spells a colour (`rules/no-literal-colour.yml` refuses one), and no
   screen holds a size of its own.

## 2. Ground, elevation, ink

Dark, with no light face. The ground is the mark's own void, so the app and
its launcher icon stand on one surface.

| token | bytes | what it is |
|---|---|---|
| `GROUND` | `10 8 15` | the glass; every screen's body; a control at rest |
| `SURFACE` | `22 20 30` | elevation one: a field, a popup's body, a row under a thumb |
| `RAISED` | `36 33 48` | elevation two: a control while pressed or open |
| `HAIRLINE` | `48 44 62` | the most a boundary may be: one point between rows |
| `INK` | `232 230 238` | primary text |
| `INK_WEAK` | `152 148 168` | secondary: a stamp, a count, a hint, a resting row |
| `INK_FAINT` | `98 94 114` | a placeholder, a disabled control's words |

The ladder climbs: each row above is brighter than the one before, asserted
(`theme::tests::elevation_climbs_and_ink_descends`). Ink is legible on every
surface — `INK` at WCAG 7:1 or better, `INK_WEAK` at 4.5:1 — also asserted.
An ink may lean toward the ground's violet, but never so far as to read as
a hue: it stays under half the saturation of the least saturated accent.

## 3. The six states, and the accent each wears

Operator ruling, binding: six colours, each a STATE, soft neon. Tokens name
the state, never the hue.

| state | colour | bytes | covers |
|---|---|---|---|
| `Attention` | green | `110 222 148` | asking for you: waiting on the operator, a parked call held for an answer, a queue entry, the attention mark on a row or a workspace |
| `Working` | blue | `96 168 255` | doing work: a tool call running, a foot executing, a step in flight. **The brand** — the focus ring, the send, the operator's own words |
| `Inference` | purple | `196 140 255` | the model is generating: the streaming tail, a turn in inference |
| `Annotation` | orange | `255 170 92` | high salience, neither working nor failed: an operator note, an interrupt, a host that restored its own set, a reply in doubt, a warning a screen must say |
| `Rest` | grey | `= INK_WEAK` | done, archived, nothing happening — the opposite of attention. A tool result that returned well |
| `Error` | red | `255 108 132` | failed and it will not mend itself: a refusal, a stopped host, a wire that will not dial |

**Soft neon is a bound, not a mood:** no accent is more than two-thirds
saturated (`theme::tests::accents_are_soft`), and every one reads on every
surface at 4.5:1 or better. An accent is spent as INK or as a RULE (a
three-point vertical line beside a block), and as a tint of a block at low
alpha; never as a full-saturation fill behind text.

**The brand is not a seventh colour.** It is the working blue, and it is
worn where the operator's own act is on the glass: the caret's field, the
send, the operator's message rule. The application mark keeps the walk's own
hues (`crate::icon`, pinned byte-for-byte against the launcher icon); it is a
picture and not a state.

**The wire's row tone reads onto the six** (`theme::tone`, REMOTE §11):
`good` is done and so is `Rest`; `bad` is `Error`; `live` is the streaming
tail and so `Inference`; `in-flight` is a tool running and so `Working`;
`plain` and `weak` are the ink scale, not a state.

**One row tone is the seat's own and no wire token spells it**: a tool call
the capability control has PARKED is `Attention`, because it is asking for you
(DESIGN §7, PROTOCOL 18). A conversation row never asks for it — a row tone
says what a conversation is doing, and a parked call is a thing inside one —
so it lives beside the six rather than in the wire's table.

**A step's framing reads onto them too** (`theme::framing`, REMOTE §4.4): a
step still being written is *a step in flight* and so `Working`; one an
interrupt cut is *an interrupt* and so `Annotation`; a failed step is `Error`
and a complete one is `Rest`. The word beside the hue is the engine's own, and
this seat never spells a second one for it.

**A word this build has not heard of wears RESTING ink** (REMOTE §3.2, DESIGN
§2.1). The protocol is a major now and additions ship inside one, so any of
those vocabularies can carry a word a newer engine spells and this build does
not. It is never painted as the nearest known hue — that is the lie the rule
exists to stop — and never costs the row. It rests, because rest is the one
accent that claims nothing, and the label says *unknown `<noun>`: `<word>`* so
the operator reads the engine's own word rather than a colour standing in for
it.

**A speaker is told by weight, not hue** (`theme::speaker`). The rule beside
a transcript block is the brand for the operator's words, full ink for the
model's, weak ink for a peer agent's, faint for an ended one. Four speakers
would otherwise be four colours, and colour means state.

## 4. Scales

Spacing, in points — every gap on the glass is one of these (`theme::space`):
`XS 4 · S 8 · M 12 · L 16 · XL 24`. The default item gap is `S`; a control's
padding is `M` across and `S` down; a screen's side gutter is `M`.

Type, in points — four sizes and no fifth (`theme::type_scale`):
`SMALL 12 · MONO 13 · BODY 15 · HEADING 20`. A button's words are body size:
a control is text, not a smaller kind of text.

Touch: every row and every control stands at least **48 points** tall
(`theme::TOUCH`, DESIGN §13.2), full width where it lists. A block's corner
is `RADIUS 10`; a speaker's rule is `RULE 3` wide.

## 5. Anatomy

**The bar** (DESIGN §13.2): the mark, the back control, the title in
`HEADING` ink. No fill, no rule under it; the gap below it is the boundary.

**A row** (`shell::theme::row`) — any list's line: full width, `TOUCH` tall,
its words at the left edge plus `M`, vertically centred, elided at the
width it has. Bare ground at rest; `RAISED` under a thumb; a `HAIRLINE`
under it. Its words may carry more than one ink in one line — the name in
`INK`, a count in `INK_WEAK`, a mark in an accent — and that ordering is the
hierarchy. Never a `Button`: a button centres its text, which is how a list
read as two alignments.

**A threaded row** (the conversation list, DESIGN §13.20): the row above, at
its own indent, its lines elided against the width it has — the first line
(name, mark, stamp) in the row's tone, every line after it in `INK_WEAK` —
with the connector rails in `INK_FAINT`. No fill at rest, no stroke ever.

**A block** (the transcript, DESIGN §7): aligned, ruled, never bubbled. Every
block starts at the same left edge; a `RULE`-wide vertical line in the
speaker's weight (§3) stands beside its header line, and the body wraps at
the width it has. A tool row's state is its words' ink; a folded one's
preview is dimmed ink, so *there is more* reads before the triangle does.

**The composer** (DESIGN §13.2): a `SURFACE` field with no stroke at rest and
the brand ring when it holds the caret, resting at `TOUCH` and growing to
its cap; beside it the send, its word in the brand. Under it the controls
band: selectors that show the VALUE they hold in `INK` and their name in
`INK_WEAK` when they hold none, greyed to `INK_FAINT` where the engine does
not offer the setting (bl-809d).

**A control** — a button, a selector, a toggle: `SURFACE` fill, no stroke,
`RADIUS` corners, `TOUCH` tall, its words in `INK`. Pressed or open it is
`RAISED`. Armed (a two-tap irreversible act, DESIGN §13.8) its words are the
attention accent, because an armed control is asking you. Disabled, its
words are `INK_FAINT` and it stays on the glass.

**A banner** — a sentence about what just happened, under the bar: the
state's accent as ink, no fill, no box. An error is `Error` ink; a note is
`Annotation` ink; nothing about the shape changes with the state.

**A band of chips** (the workspace bar, the admin foot, the controls under
the composer; `shell::theme::chip`): a band `TOUCH` tall divided EQUALLY
among its controls, so the count of controls is a fact the layout cannot
lose (DESIGN §13.14) — each chip a `SURFACE` block with `RADIUS` corners, its
label eliding inside its share rather than pushing a neighbour off the
glass. Dark chips stay on the glass in `INK_FAINT` and say what would light
them. The world entries on the roster are rows, not chips: a list of places
is a list.

## 6. Where it is asserted

`src/theme.rs` is pure and under the 100% floor: the ladder, the contrast
floors, the softness bound, the six-and-no-seventh, the tone and speaker
readings and the scales are all `theme::tests`. `src/shell/theme.rs` is the
adapter and is android-only (`tarpaulin.toml`): what it decides is nothing.
`make screens` captures every screen for eyes (DESIGN §15); the pictures
that adopted this language are in the usability campaign's round-3 notes.
