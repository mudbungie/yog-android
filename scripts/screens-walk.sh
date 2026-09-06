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


# 2. A leaf, and this device is a seat. The roster is "main".
mint_material; seed_cache roster; relaunch
capture roster roster


# 3. THE STANDING ASSERTION: the configuration surface is reachable from the
#    roster, by the one control that leads there. Then the mark toggles back —
#    a way in with no way out is the same defect wearing the other face.
tap_mark
capture settings configuration
tap_mark
capture back-to-roster roster

# 3b. THE TWO WORLD SURFACES (DESIGN §13.8, bl-35bd). Neither is a depth of the
#     focus — the queue spans every workspace and the trail is the engine's own
#     record — so both are reached from the roster's own entries and both leave
#     the focus where it was. Each is walked from a fresh launch rather than by
#     backing out of the other: the way back is the bar's back control, which
#     carries no rectangle, and a relaunch is the harness's existing door.
tap_control waiting
capture waiting attention

relaunch
tap_control trail
capture trail trail

# 3c. THE BALL PANE'S TWO WORLD READS (DESIGN §13.9, bl-d587). Neither names a
#     workspace, so both are the roster's entries beside the two above — and
#     each is walked from a fresh launch for the same reason. The pane dials an
#     engine that is not there, so what these two capture is a screen that
#     opened and said so; what the parity gate wants from them is the control,
#     and the control is on the roster either way.
relaunch
tap_control balls
capture balls balls

relaunch
tap_control board
capture board board

# 3d. THE OP TABLE (DESIGN §13.14). The fifth roster entry, and the only screen
#     in this app that works with nothing dialled and nothing seeded — the
#     table is compiled in. So this beat is what proves `act:help` is on the
#     glass, and the picture is the vocabulary itself.
relaunch
tap_control help
capture help help

# 4. The two deeper screens, each selected by the focus stored beside its rows.
seed_cache conversations; relaunch
capture conversations conversations

# 4a. THE BALL PANE'S AIMED READ (DESIGN §13.9). The one of the three that
#     names a place, and it is offered on the screen that names it — a
#     workspace's conversation list — so this is the only depth it can be
#     reached from. Walked before the row menu, because a menu left open would
#     be over the entry.
tap_control workspace-balls
capture workspace-balls workspace-balls

# 4b. THE CANDIDATES SCREEN (DESIGN §13.12). The other read that names a
#     workspace, so the other entry on this same list. The engine is not
#     dialled, so what this captures is a screen that opened and said nothing
#     was read; what the parity gate wants from it is the four controls, and
#     all four paint — three of them disabled, saying which row to tap first.
relaunch
tap_control science
capture candidates science

# 4c. THE FLEET SCREEN (DESIGN §13.13). The third aimed entry on the same
#     list, and the one screen in this app that reads nothing at all — what its
#     four acts DID is on the board. So this beat captures a screen with no
#     answer to show and four controls, two of them dark and saying which word
#     would light them, which is what the parity gate asks of them.
relaunch
tap_control fleet
capture fleet fleet

# 4d. THE MACHINES ROSTER (DESIGN §13.14). The fourth entry on the aimed band,
#     and the one surface in this app with no control inside it at all — every
#     other op in REMOTE §5 is a machine's. The engine is not dialled, so this
#     captures a screen that opened and said nothing was read.
relaunch
tap_control clients
capture clients clients

# 4da. THE WORK SCREEN (DESIGN §13.15). The fifth aimed entry, and the other
#      half of the candidates screen's row: what an attempt CHANGED, off the
#      same diff object `science` carries. The engine is not dialled, so this
#      captures a screen that opened and said nothing was read; what the parity
#      gate wants from it is the entry, which is on the band either way.
relaunch
tap_control work-diff
capture work work-diff

# 4db. THE ADMIN SCREEN (DESIGN §13.17). The sixth aimed entry: the config
#      files, the task branch, the inbox flush and the unmaking of the
#      workspace itself. The engine is not dialled, so this captures a screen
#      that opened and said nothing was read; what the parity gate wants from
#      it is the four controls, three of them dark and saying which word would
#      light them.
relaunch
tap_control config
capture admin config

# Back to the list the menu opens from: the seed has not changed, so this is a
# relaunch and not a re-seed.
relaunch

# 4e. THE ROW MENU (DESIGN §13.5, bl-f97c). Not a sixth screen — the app says
#     `conversations` with a menu up, exactly as it says `transcript` with the
#     stop gates on — but the three conversation acts exist nowhere else, and
#     the parity gate below can only see a control a walked screen painted.
#     This beat is also the ONLY place the long-press synthesis is proven on a
#     device rather than read out of egui's source: no menu, no `act:` tags, and
#     the gate goes red naming all three ops.
long_press_row
capture row-menu conversations

seed_cache transcript; relaunch
capture transcript transcript

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
tap_control files
capture files files

# 5. The same screen with the engine's stop gates ON. Not a sixth screen — the
#    app says `transcript` for both — but the controls row is a different set
#    of controls under it, and the parity gate below can only see a control a
#    walked screen actually painted.
seed_cache running; relaunch
capture running transcript

fetch_beats
# The shade beats last of all: the third one posts a notification, which puts a
# row in the status bar of every picture taken after it.
shade_beats
# The pocketed foot after them, because it is the one set of beats that changes
# what this device IS — it re-provisions the leaf as foot-grade and back — and
# every screen above wants the seat it was walked with.
pocket_beats

