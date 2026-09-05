#!/usr/bin/env bash
# **The instruments**: how the harness reads the app and writes a row about it
# — relaunch it, ask what it says it is painting, wait for it to stop moving,
# capture the evidence, and judge one screen against what the walk asked for.
#
# Sourced by `scripts/screens.sh`, and its own file for the seam that file's
# own header already draws (bl-46e6): the walk decides WHERE to go and what
# each beat proves; nothing here knows which screen it is looking at, and
# nothing there knows how a screen is read.

relaunch() {
  "${ADB[@]}" shell am force-stop "$PKG"
  "${ADB[@]}" logcat -c || true
  "${ADB[@]}" shell am start -W -n "$PKG/.MainActivity" >/dev/null
}

# The last thing the app said it was painting (`src/shell/app/probe.rs`).
# Matched on the MESSAGE marker, never on a logcat tag: android_logger tags a
# record with its module path, so a tag filter would silently stop matching
# the day that file moves. The `|| true` is load-bearing under `pipefail` — a
# grep that finds nothing yet is this loop's ordinary state, not its failure.
probe() { "${ADB[@]}" logcat -d 2>/dev/null | grep -o 'yog\.screen .*' | tail -1 || true; }

# Wait for the app to stop moving, with a deadline: the first frames report a
# mark rect that is still travelling, because the platform's top inset is a
# throttled JNI probe that lands over the first second or two (DESIGN §3). A
# tap aimed at the first answer lands above the control. Stable means the same
# line twice running; SETTLED is what everything downstream reads.
#
# The budget is generous on purpose. The FIRST launch after an install can sit
# on the splash for a minute or more — a software GPU compiling this app's
# shaders — and a deadline tuned to a warm start turns that into a red walk
# with a picture of a launcher icon in it. A deadline that is too long costs
# time only when something is already wrong; one that is too short costs a
# false verdict on a good tree, and that is the more expensive mistake.
SETTLE_TRIES=${SETTLE_TRIES:-80}
SETTLED=""
settle() {
  local prev="" now="" i=0
  SETTLED=""
  while [ "$i" -lt "$SETTLE_TRIES" ]; do
    now=$(probe)
    if [ -n "$now" ] && [ "$now" = "$prev" ]; then SETTLED="$now"; return 0; fi
    prev="$now"; sleep 1.5; i=$((i + 1))
  done
  SETTLED="$now"
}

STEP=0
FAILED=0
verdict() {            # verdict <pass|fail> <label>
  echo "  $1  $2" | tee -a "$OUT/verdict.txt"
  [ "$1" = fail ] && FAILED=1
  return 0
}

# One screen: settle, capture the picture, capture the (empty) accessibility
# dump beside it as evidence, record what the app said, and judge it.
capture() {            # capture <name> <expected-screen>
  STEP=$((STEP + 1))
  local n; n=$(printf '%02d-%s' "$STEP" "$1")
  settle
  "${ADB[@]}" exec-out screencap -p > "$OUT/$n.png"
  "${ADB[@]}" exec-out uiautomator dump /dev/tty 2>/dev/null | sed 's/UI hierchary dumped to.*//' > "$OUT/$n.ui.xml"
  pull_parity "$OUT/$n.tags"
  echo "$SETTLED" > "$OUT/$n.probe"
  local said="${SETTLED#yog.screen }"
  case "$said" in
    "screen=$2"|"screen=$2 "*) verdict pass "$n: painted $2" ;;
    "") verdict fail "$n: the app said nothing — expected $2" ;;
    *) verdict fail "$n: expected $2, got ${said%% *}" ;;
  esac
}
