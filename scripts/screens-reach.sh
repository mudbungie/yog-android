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
