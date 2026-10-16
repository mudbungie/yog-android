# The harness's REACH: turning a rectangle the app reported into a gesture.
#
# Sourced by `screens.sh`, and its own file for a reason the other three share
# — it answers a different question. This app's accessibility tree is a single
# opaque view with no text in it (DESIGN §15.1), so no control here can be
# found by name, and `uiautomator` has nothing to click. What exists instead is
# `src/shell/app/probe.rs`: the app states, in device pixels, where the two
# controls a walk must reach were painted. Everything below is that rectangle
# plus the gesture the control answers to.
#
# Both helpers `settle` first, because the first frames report a rect that is
# still travelling (the top inset is a throttled JNI probe), and a gesture
# aimed at the first answer lands somewhere else.

# Tap a control the app NAMED, in device pixels, as the app itself reported it
# (`app/probe.rs`). None of them carries text or an accessibility node — the
# mark is a picture, a conversation row and the two world entries are egui
# labels — so a rectangle the app states is the only way to any of them, and it
# is the app's own answer rather than a coordinate guessed here.
tap_control() {        # tap_control <name>
  settle
  local rect; rect=$(printf '%s' "$SETTLED" | sed -n "s/.*[ ]$1=\([0-9,]*\).*/\1/p")
  [ -n "$rect" ] || { verdict fail "the screen reported no $1 to tap"; return 0; }
  local x y w h; IFS=, read -r x y w h <<<"$rect"
  "${ADB[@]}" logcat -c || true
  "${ADB[@]}" shell input tap $((x + w / 2)) $((y + h / 2))
}

# The mark, which every screen paints and which toggles the configuration
# surface. Named because the standing assertion is about that surface and
# reads better for it.
tap_mark() { tap_control mark; }

# The platform's back, which on the chat opens the switcher (DESIGN §22 item
# 7). A key event rather than a rectangle: back is the platform's gesture and
# has no control of its own on the glass.
press_back() {
  settle
  "${ADB[@]}" logcat -c || true
  "${ADB[@]}" shell input keyevent KEYCODE_BACK
}

# slash <typed>: open the verb sheet by typing `/<typed>` into the composer
# (DESIGN §22 item 5) — the narrowing a thumb does, so the row the walk taps
# next is on the glass however long the sheet is. The door itself (`prompt`)
# is proven by its own beat.
slash() {
  tap_control composer
  "${ADB[@]}" shell input text "/$1"
}

# Press and HOLD the first conversation row, which is the one way the row menu
# opens (DESIGN §13.5). The rectangle is the app's own answer, for the mark's
# reason exactly: a row carries no accessibility node either.
#
# It has to be a held press and not a tap. egui synthesizes the secondary click
# from a touch held past its own `max_click_duration` (0.8 s) and wakes itself
# to check, so the hold must outlast that with room for a frame on a software
# GPU; `input tap` never can, and `input swipe` is read as a drag, which is the
# one thing that cancels a long press. Hence DOWN, wait, UP as separate events.
#
# The logcat is deliberately NOT cleared: opening a menu changes no screen
# name, so the probe line does not change, and the app says a line only when it
# changes — clearing here would leave `settle` reading nothing until its
# deadline.
# A long press on a NAMED control (DESIGN §23 item 10): the control tells
# its sentence rather than firing, and says it on the probe as `yog.told`.
long_press_control() {   # long_press_control <name>
  settle
  local rect; rect=$(printf '%s' "$SETTLED" | sed -n "s/.*[ ]$1=\([0-9,]*\).*/\1/p")
  [ -n "$rect" ] || { verdict fail "the screen reported no $1 to press"; return 0; }
  local x y w h; IFS=, read -r x y w h <<<"$rect"
  local cx=$((x + w / 2)) cy=$((y + h / 2))
  # The log is NOT cleared here: a long press changes no screen, so no new
  # probe line follows it, and a cleared log would read as the app saying
  # nothing. The told line is emitted once per sentence either way.
  "${ADB[@]}" shell input motionevent DOWN "$cx" "$cy"
  sleep 2
  "${ADB[@]}" shell input motionevent UP "$cx" "$cy"
}

# Whether the last long press was told: the sentence on the probe's own
# channel. Absent, the control stayed silent, which is the defect §23 item
# 10 exists to make impossible.
require_told() {
  local said; said=$("${ADB[@]}" logcat -d 2>/dev/null | grep -o 'yog\.told .*' | tail -1 || true)
  if [ -n "$said" ]; then verdict pass "told: ${said#yog.told }"; else verdict fail "a long press told nothing"; fi
}

long_press_row() {
  settle
  local rect; rect=$(printf '%s' "$SETTLED" | sed -n 's/.*row=\([0-9,]*\).*/\1/p')
  [ -n "$rect" ] || { verdict fail "the screen reported no conversation row to press"; return 0; }
  local x y w h; IFS=, read -r x y w h <<<"$rect"
  local cx=$((x + w / 2)) cy=$((y + h / 2))
  "${ADB[@]}" shell input motionevent DOWN "$cx" "$cy"
  sleep 2
  "${ADB[@]}" shell input motionevent UP "$cx" "$cy"
}
