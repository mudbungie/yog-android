#!/usr/bin/env bash
# **THE WALK** (bl-243b, split out under bl-477e when the beats took
# `screens.sh` past the 300-line cap): where this loop goes, and what each beat
# proves. Sourced by `scripts/screens.sh`, and its own file on the seam that
# file already draws three times — the INSTRUMENTS that read the app
# (`screens-capture.sh`), the SEEDS that put it on a screen
# (`screens-seed.sh`), the REACH that turns a rectangle into a gesture
# (`screens-reach.sh`), and now the itinerary that spends all three.
#
# Nothing here knows how a screen is read or how a control is reached; nothing
# in those files knows which screen is being visited. What is left in
# `screens.sh` is the boot, the install and the two gates that judge the run.

echo "screens: walking" >&2

# 1. Nothing provisioned: the bootstrap chooser, which is every device's first
#    screen and the one that needs no seed at all.
wipe_app; relaunch
capture cold configuration


# 2. A leaf, and this device is a seat. Nothing is focused, so the switcher
#    stands (DESIGN §22 item 7): the seat panel at its head, every workspace a
#    folded heading under it.
mint_material; seed_cache roster; relaunch
capture seated conversations


# 4. A workspace focused and no conversation: the chat in start mode.
seed_cache conversations; relaunch
capture start start

# 4m. THE STANDING ASSERTION: the configuration surface is reachable from the
#     chat, by the one control that leads there — the mark, at the left of
#     the status rule (DESIGN §22 item 2). Then the mark toggles back — a way
#     in with no way out is the same defect wearing the other face.
tap_mark
capture settings configuration
tap_mark
capture back-to-roster start

# 4w. THE WORLD SURFACES (DESIGN §13.8, §13.9, §13.14), by verb now (§22
#     items 5 and 7): the roster's entries went with the roster, and every
#     chat has a prompt line to open the sheet from. Each from a fresh launch,
#     the harness's existing door. `/balls all` is a two-word row the probe
#     does not name; its tag is read off the sheet in the slashing beat.
slash att
tap_control attention
capture waiting attention

relaunch
slash tra
tap_control trail
capture trail trail

relaunch
slash boa
tap_control board
capture board board

relaunch
slash hel
tap_control help
capture help help

# 4a. THE BALL PANE'S AIMED READ (DESIGN §13.9). The one of the three that
#     names a place, and it is offered on the screen that names it — a
#     workspace's conversation list — so this is the only depth it can be
#     reached from.
relaunch
slash bal
tap_control balls
capture workspace-balls workspace-balls

# 4b. THE CANDIDATES SCREEN (DESIGN §13.12). The other read that names a
#     workspace, so the other entry on this same list. The engine is not
#     dialled, so what this captures is a screen that opened and said nothing
#     was read; what the parity gate wants from it is the four controls, and
#     all four paint — three of them disabled, saying which row to tap first.
relaunch
slash sci
tap_control science
capture candidates science

# 4c. THE FLEET SCREEN (DESIGN §13.13). The third aimed entry on the same
#     list, and the one screen in this app that reads nothing at all — what its
#     four acts DID is on the board. So this beat captures a screen with no
#     answer to show and four controls, two of them dark and saying which word
#     would light them, which is what the parity gate asks of them.
relaunch
slash fle
tap_control fleet
capture fleet fleet

# 4d. THE MACHINES ROSTER (DESIGN §13.14). The fourth entry on the aimed band,
#     and the one surface in this app with no control inside it at all — every
#     other op in REMOTE §5 is a machine's. The engine is not dialled, so this
#     captures a screen that opened and said nothing was read.
relaunch
slash cli
tap_control clients
capture clients clients

# 4da. THE WORK SCREEN (DESIGN §13.15). The fifth aimed entry, and the other
#      half of the candidates screen's row: what an attempt CHANGED, off the
#      same diff object `science` carries. The engine is not dialled, so this
#      captures a screen that opened and said nothing was read; what the parity
#      gate wants from it is the entry, which is on the band either way.
relaunch
slash dif
tap_control diff
capture work work-diff

# 4db. THE ADMIN SCREEN (DESIGN §13.17). The sixth aimed entry: the config
#      files, the task branch, the inbox flush, what a reviewer has STAGED, and
#      the unmaking of the workspace itself. The engine is not dialled, so this
#      captures a screen that opened and said nothing was read; what the parity
#      gate wants from it is the six controls — five of them dark and saying
#      which word or which row would light them, `accept` and `reject` among
#      them, which is what puts `act:proposal` on the glass with nothing
#      staged.
relaunch
slash con
tap_control config
capture admin config

# 4dc. THE SIGN-IN SCREEN (DESIGN §13.19). The seventh aimed entry, and the
#      only one whose rows come out of the seeded cache rather than out of a
#      dial: the `conversations` seed carries the workspace's `providers`
#      envelope, so both provider rows paint with the engine's own credential
#      facts on them — one signable and one blocked, which is what puts
#      `act:login` on the glass enabled AND dark. The tail under them is
#      empty, because nothing is dialled and nobody has tapped a row.
relaunch
slash log
tap_control login
capture signin login

# 4s. THE SWITCHER (DESIGN §22 item 7, bl-3edc): back from the chat opens
#     it, the focused workspace open under its heading with its conversations
#     as the §13.20 tree and `+ new` under them. The seed has not changed, so
#     this is a relaunch and not a re-seed.
relaunch
press_back
capture switching conversations

# 4e. THE ROW MENU (DESIGN §13.5, bl-f97c). Not a sixth screen — the app says
#     `conversations` with a menu up, exactly as it says `transcript` with the
#     stop gates on — but the three conversation acts exist nowhere else, and
#     the parity gate below can only see a control a walked screen painted.
#     This beat is also the ONLY place the long-press synthesis is proven on a
#     device rather than read out of egui's source: no menu, no `act:` tags, and
#     the gate goes red naming all three ops.
long_press_row
capture row-menu conversations

# 4t. `+ new` closes the switcher onto the chat in start mode, the caret in
#     the field.
relaunch
press_back
tap_control new
capture started start

seed_cache transcript; relaunch
capture transcript transcript

# 4e2. THE MODEL DROPUP (DESIGN §22 items 2 and 10; operator ruling
#      2026-10-12): the rule's model segment opens its list upward from the
#      rule, the cached models at once and a reload asked; the mark on the
#      segment says which. Relaunched after, so the open list is not over the
#      door the next beat taps.
tap_control rule-model
capture modelled transcript
relaunch

# 4ea. THE VERB SHEET (DESIGN §22 item 5, bl-0d15). The prompt glyph is the
#      door: a tap puts `/` at the head of the draft and the sheet stands while
#      it does. At rest, so `nudge` is among its rows — the parity gate reads
#      the stop chips' tags off the sheet now, here and in the running beat.
tap_control prompt
capture slashing transcript

# 4f. THE RECORDS SCREEN (DESIGN §13.11). One drill-down depth behind the
#     transcript, reached by the one control that opens it — which is also the
#     affordance for the five reads it asks (PARITY §2). The engine is not
#     dialled, so what this captures is a screen that opened and said nothing
#     was read; what the parity gate wants from it is the sixth control, and
#     `step` paints on this screen disabled with its reason beside it.
tap_control records
capture records records

# 4g. THE FILES SCREEN (DESIGN §13.15). The other depth behind the transcript,
#     beside the records entry in the same band — `files` names the
#     conversation, so this is the only depth it can be reached from. Reached
#     from a relaunch rather than by backing out of the records screen, which is
#     the harness's existing door.
relaunch
tap_control prompt
tap_control files
capture files files

# 5. The same screen with the engine's stop gates ON. Not a sixth screen — the
#    app says `transcript` for both — but the controls row is a different set
#    of controls under it, and the parity gate below can only see a control a
#    walked screen actually painted.
seed_cache running; relaunch
tap_control prompt
capture running transcript

fetch_beats
# The shade beats last of all: the third one posts a notification, which puts a
# row in the status bar of every picture taken after it.
shade_beats
# The pocketed foot after them, because it is the one set of beats that changes
# what this device IS — it re-provisions the leaf as foot-grade and back — and
# every screen above wants the seat it was walked with.
pocket_beats
# The held attention lane last of all (§17.6): it is a SEAT's rung, and the
# beats above end by putting a seat leaf back on this device — so this is the
# one order in which both rungs are measured on the device they are for.
lane_beats
# The reboot last of all (§18.8): it is the one gesture in this walk that
# costs a device boot, and every beat above wants a device it did not just
# restart.
boot_beats

