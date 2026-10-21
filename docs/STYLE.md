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
   ring on the field that holds the caret. **Amended by DESIGN §22
   (bl-bf74), the desktop's one exception adopted:** a `HAIRLINE` frame
   marks what FLOATS — an overlay, the verb sheet, the ask (its frame in the
   attention accent) — and the panel at the head of a transcript. A stroke on
   a thing that floats says what is over what; nothing else is boxed.
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
a control is text, not a smaller kind of text. **The one exception is the
controls band under the composer** (bl-0691), whose six controls at body size
spend two hundred points of a phone's row on three verbs and leave the two
selectors nothing to read a provider name in; its words are `SMALL` and its
chips are padded at `S`, and that is the whole of the exception — an opened
list is body size like every other list. **The exception goes with the band
(DESIGN §22 item 10)**: its controls move to the model overlay at a full row
each, where body size fits. The status rule's runs are `SMALL` ink inside
a `TOUCH`-tall line, which is a line of facts and not a control's words —
its knobs are doors in that line, not buttons (DESIGN §23 item 14).

Touch: every row and every control stands at least **48 points** tall
(`theme::TOUCH`, DESIGN §13.2), full width where it lists. A block's corner
is `RADIUS 10`; a speaker's rule is `RULE 3` wide.

## 5. Anatomy

**The shape, whole** (DESIGN §22, bl-bf74 — the desktop's Hermes shape at a
phone's width; **§23 adds the tab strip over the transcript from the second open conversation, the drawer swipe, the bubbles, the told sentence on a long press and the settings overlay, and item 4 lays the floor in the desktop's order**): the chat is the home screen; the transcript fills it; from
the floor, bottom-up, stand the status rule at the bottom edge, the banner
line and the dock over it while they have something to say, the prompt line
directly under the transcript, and the ask on the prompt line while the
conversation asks. Everything else is an
overlay, asked for by a tap on the rule or on `›`, and closed by its name line
or the back gesture.

**The status rule** (`status`; DESIGN §23 item 14): one line `TOUCH` tall
at the bottom edge, `SMALL` runs: the mark at the left end, its picture the
line's height in a `TOUCH`-square seat, then `— <workspace> ⏶ | <provider> ⏶
| <model> ⏶ | <effort> ⏶ | <priority> | <state · age> | <tokens> | <spend> |
<context> — <name>`, the `|` and `—` in `INK_FAINT` with `XS` either side.
**Each knob is a door** (`shell::theme::door`): its words in `INK` and `⏶`
in `INK_WEAK` inside one seat, `TOUCH` tall with `XS` of air either side,
bare ground at rest and `RAISED` while pressed; the reload's `…`
(annotation) or `✔` (attention) stands in the mark's place, a flip and a
dark knob wear none, and a dark knob's words are `INK_FAINT`. The state is
in its state's ink, the other diagnostics `INK_WEAK`, the name `INK` and a
door. The diagnostics drop from the right, then priority, then effort; the
selectors and the name never drop and elide, and where one line will not
hold both halves the diagnostics and the name stand on a small line under
it. No chip, no fill and no caret on the rule.

**The prompt line** (`composer`): `›` in the brand at the left edge plus `M`,
a `TOUCH`-square target of its own (it opens the verb sheet); the platform
field beside it, dressed by `shell::theme::skin` — `SURFACE`, the one-point
brand ring while it holds the caret, the attention tint while the
conversation asks, resting at `TOUCH` and growing to its cap only while
focused; `send` at the right end in the brand on bare ground, painted only
while the draft holds text. Its hint in `INK_FAINT`: `say it`; in start
mode `begin a conversation on <workspace>`; while the conversation asks
`it is waiting on you`. It stands directly under the transcript, on the
rule (DESIGN §23 item 4).

**The verb sheet** (`slash`): a `SURFACE` box with a `HAIRLINE` frame and
`RADIUS` corners, laid out directly above the prompt line — never over the
field — as tall as its rows or the room above it, whichever is less, and
scrolling inside that. One `SMALL` line of what it is narrowed to, then rows
`TOUCH` tall: `/word` in `INK`, the summary in `INK_WEAK`, elided at the row's
width; a verb's glyph (`■ stop`, `▶ nudge`, `⚡ interrupt`) before its word; a
toggle's state (`on`/`off`) at the right end in `INK_WEAK`; `RAISED` under a
thumb.

**An overlay** (`overlay`): a `SURFACE` box with a `HAIRLINE` frame, full
width less `S` each side, from `S` under the top inset to the floor. Its
first line is a `TOUCH`-tall row — the name in `INK` at `BODY`, what it is
open on beside it in `INK_WEAK`, `close` at the other end in `INK_WEAK` — and
the whole line is the close. Its body under that, scrolling inside the box.
The rule and the prompt stand down beneath it.

**The switcher** (`switcher`): the overlay above, named `conversations`. At
its head the seat panel's lines; then one heading row per workspace (name in
`INK`, the attention mark in its accent, the agent count in `INK_WEAK`), the
focused one open beneath it as the threaded rows below, `+ new` in the brand
as the last row under it.

**The ask** (`ask`): the overlay's box with its frame and name line in the
attention accent, laid out directly above the prompt line: the name
line `it is asking` with `why +` at its other end, the held line in the
attention accent under it, the row's other words one line each in their
inks, then the choices as numbered cells two to a row, each `TOUCH` tall and
half the width, their words wrapping to a second line rather than eliding,
`RAISED` under a thumb; `1. seen` alone when nothing is parked.

**The tab strip** (`tabs`; DESIGN §23 item 2): one `TOUCH`-tall row over
the transcript, standing from the second open conversation, scrolling
sideways; a tab is its name at `BODY` with `S` of air either side, the
asking `?` before it in the attention accent, the focused one in `INK` over
a brand hairline and the rest in `INK_WEAK`, `×` in `INK_FAINT` as its own
seat after it; the `RAISED` wash while a thumb is on either.

**The dock** (`dock`): one `TOUCH`-tall row on the rule, under the prompt
line, `+ N running ·
<name>` in `INK_WEAK` with the count in the working accent; unfolded, one row
apiece beneath the head, name in `INK`, state in its ink, age in `INK_WEAK`.

**The panel at the head** (`chat::panel`): a `SURFACE` card with a
`HAIRLINE` frame and `RADIUS` corners, `M` across and `S` down inside, one
fact a line — the name in `INK`, the rest in `INK_WEAK`, the long ones behind
`+`; the seat's card the same, the update offer in the brand.

**The bar** (DESIGN §13.2; **since §22, only on the whole-window surfaces** —
the configuration surface, the foot's screen and the enrollment screens): the
mark, the back control, the title in `HEADING` ink. No fill, no rule under it;
the gap below it is the boundary.

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

**A block** (the transcript, DESIGN §7; **gutter since §22 item 4; bubbled by role since §23 item 7** — the operator's turn on the brand's tint, a trail on the ground inside a hairline, intermediate prose on `SURFACE`, the final prose on `RAISED`, each at `RADIUS` with the gutter glyph inside it):
aligned, guttered, bubbled. Every block starts at the same left edge
with its glyph in the gutter — `›` in the brand for the operator, `¦` in
`INK_WEAK` for the model, `·` in `INK_WEAK` for a peer, a tool's answer or the
machinery — then who spoke in their weight (§3) on a line of its own, with
no colon: `user` (or `op`) for the operator, the model id for a model's
prose, `<model id> (thinking)` for its reasoning (DESIGN §23 item 15) — and
the body wraps at the width it has, in `INK`. A finished thought or
intermediate response on a trail is one `<who> · <first line>` line in
`INK_WEAK`, elided, its `+` at the end; a finished call is its line with
`✔`/`✖` and one weak line of output under it (DESIGN §23 item 6). A turn's calls stand under one fold head `+ tool calls (N)` in
`INK_WEAK`, each on a painted stem (the threaded row's rails) with `•` in the
brand, `✔` in `INK_WEAK` or `✖` in the error accent; a `HAIRLINE` parts
turns. A tool row's state is its words' ink; a folded one's
preview is dimmed ink, so *there is more* reads before the fold glyph does;
an unfolded tool answer is in the code face (`MONO`).

**The composer** (DESIGN §13.2; **retired by §22 — the prompt line above
replaces it**): a `SURFACE` field with no stroke at rest and
the brand ring when it holds the caret, resting at `TOUCH` and **growing to
its cap only while it holds the caret** (bl-0691); beside it the send, its
word in the brand and centred in its own box like every other control's.
**The field is a platform `EditText`** (bl-8bbb) and is dressed from these
same tokens through `shell::theme::skin`, which is this file's one adapter
wearing its other face: packed ARGB and device pixels where egui takes a
`Color32` and points. The ring it wears while focused is one point wide — the
most a boundary may be (§2) — because what a focused field changes is the
brand, never the weight.

**The controls band** (DESIGN §13.2, bl-0691; **retired by §22 item 10** —
its stops are verbs, its selectors and knobs are the model overlay's rows):
ONE band of chips under the
composer, `TOUCH` tall, holding every conversation-level control — the stop
acts under the engine's own gates, the tuning pair, and the two selectors.
Its width is divided by `shell::place::row` rather than equally: a chip that
names a verb keeps the width of its own words, and the two selectors take
what is left and elide inside it, because a model name is as long as a
provider chose to make it. Its words are `SMALL` — the one place in this app
where a control's are, because six controls at body size leave the selectors
nothing to say a name in — and its chips are padded at the `S` step rather
than the `M` one. Each shows the VALUE it holds in `INK` and its own name in
`INK_WEAK` when it holds none, and goes `INK_FAINT` where the provider's row
does not offer the setting (bl-809d). **A dark chip here is still a target**:
its reason is a sentence and not two words, so the tap is what says it, in
`Annotation` ink on the line above the band (bl-0691).

**A control** — a button, a selector, a toggle (**every one carries its help sentence since §23 item 10**: a long press opens it in the overlay's box above the control, `INK` at `BODY`, until the next tap): `SURFACE` fill, no stroke,
`RADIUS` corners, `TOUCH` tall, its words in `INK`. Pressed or open it is
`RAISED`. Armed (a two-tap irreversible act, DESIGN §13.8) its words are the
attention accent, because an armed control is asking you. Disabled, its
words are `INK_FAINT` and it stays on the glass.

**A banner** — a sentence about what just happened (**since §22, in the
rule's state slot**, and a tap puts it down): the
state's accent as ink, no fill, no box. An error is `Error` ink; a note is
`Annotation` ink; nothing about the shape changes with the state.

**The held band** (DESIGN §13.7; **retired by §22 item 8 — the ask above
replaces it**): the three rows a parked call stands on,
directly under the composer. One line in the attention accent saying what is
held and which class the control put it in, with a `why` chip beside it that
opens the control's own decision above the line; then the scope chips; then
the verdict chips. Both control rows are bands of chips (below), so neither
can lose a control off the glass. **The call's input is never in this band** —
a tool's JSON belongs on the transcript row that carries the call, not between
the message field and the answers.

**A band of chips** (the workspace bar, the admin foot, the ask's cells —
never the rule, whose knobs are doors; `shell::theme::chip`): a band `TOUCH` tall divided EQUALLY
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
