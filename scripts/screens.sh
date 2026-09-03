#!/usr/bin/env bash
# **The render-and-see loop** (bl-243b): boot a headless emulator, install this
# app, walk it through its named screens, capture each one, and answer whether
# the walk went where it said it would. `make screens` is the door.
#
# WHAT GATES AND WHAT DOES NOT. The PNGs are for an agent's eyes and nothing
# asserts over them — no golden image, no pixel diff, because a picture that
# fails on a font bump teaches nobody anything. What gates is STRUCTURAL: the
# screen the app says it painted, read out of logcat (`src/shell/app/probe.rs`),
# against the screen this walk asked for. The standing assertion is the last
# pair — the configuration surface is reachable from the roster and the mark
# toggles back out of it — which is the defect class this loop exists to catch.
#
# THE ACCESSIBILITY DUMP IS STILL EMPTY, AND THE SECOND GATE READS A FILE
# INSTEAD (bl-fe4c). This app paints with egui into one opaque view, so
# `uiautomator dump` returns a single `android.view.View` carrying no text, no
# button and no row. Exporting egui's tree to the platform is one eframe
# feature and one dependency away, and it ABORTS the process the moment any
# accessibility client attaches — this walk's own dump killed the app on the
# second screen (DESIGN §15.1). So the app writes the `act:<op>` tags it
# painted to a file in its own private storage, armed by a directory this
# script creates, and the last beat judges THAT against the engine's roster
# (yog docs/PARITY.md §5, §6's named fallback). The dump is still captured
# beside every screenshot: the emptiness lives in the run's own evidence, and
# the day upstream stops unwrapping (bl-a6f3) the same files say so.
#
# THE ENGINE IS NOT DIALLED AND DOES NOT NEED TO BE. Two seeds put the app on
# any screen without a server anywhere:
#   * key material minted here by `openssl` — a self-signed CA and a leaf under
#     it. `transport::Seat::open` dials nothing, so a leaf is enough to make
#     this device a seat; the address points at a closed port and the wire
#     failure is painted, which is itself a screen worth a picture.
#   * the paint-first cache (DESIGN §14), seeded from `corpus/` — the wire
#     corpus vendored out of the server's own codec. That is the "recorded
#     endpoint": the rows are the ENGINE's spelling, not a second one invented
#     here, and the focus stored beside them is what selects the screen. A
#     cache this build refuses is discarded whole, which shows up as the wrong
#     screen name and reddens the walk rather than passing quietly.
set -euo pipefail

AVD=${AVD:-yog-screens}
PORT=${PORT:-5584}
KEEP=${KEEP:-0}
APK=${APK:-android/app/build/outputs/apk/debug/app-debug.apk}
OUT=${OUT:-target/screens}
PKG=dev.yog

while [ $# -gt 0 ]; do
  case "$1" in
    --avd) AVD="$2"; shift 2 ;;
    --port) PORT="$2"; shift 2 ;;
    --apk) APK="$2"; shift 2 ;;
    --out) OUT="$2"; shift 2 ;;
    --keep) KEEP=1; shift ;;
    *) echo "usage: $0 [--avd N] [--port N] [--apk P] [--out D] [--keep]" >&2; exit 2 ;;
  esac
done

cd "$(git rev-parse --show-toplevel)"

SDK=${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}
ADB_BIN="$SDK/platform-tools/adb"
EMULATOR_BIN="$SDK/emulator/emulator"
SERIAL="emulator-$PORT"

die() { echo "screens: $*" >&2; exit 1; }

# Preflight, all of it before anything is booted: every failure here names the
# one command that fixes it, because the alternative is a five-minute boot that
# dies on a missing binary.
[ -x "$ADB_BIN" ] || die "no adb at $ADB_BIN — set ANDROID_HOME, or install platform-tools"
[ -x "$EMULATOR_BIN" ] || die "no emulator at $EMULATOR_BIN — sdkmanager --install emulator"
command -v openssl >/dev/null || die "openssl is required to mint this walk's leaf"
command -v python3 >/dev/null || die "python3 is required to assemble the cache seed"
[ -f "$HOME/.android/avd/$AVD.ini" ] || die "no AVD named '$AVD' — create it once with:
  make screens-avd
(that target installs the system image, which may need an SDK licence
accepted; accepting one is an operator act and no script here will do it.)"
[ -f "$APK" ] || die "no APK at $APK — build one first:
  make apk ABIS=x86_64 GRADLE=/path/to/gradle"

ADB=("$ADB_BIN" -s "$SERIAL")
rm -rf "$OUT"; mkdir -p "$OUT"

# The emulator this run started, and the ONLY process this script ever kills.
# A pattern kill would take another agent's emulator with it.
EMU=""
teardown() {
  local code=$?
  [ -n "$EMU" ] || exit $code
  if [ "$KEEP" = 1 ]; then
    echo "screens: leaving $SERIAL up (--keep); kill it with: kill $EMU" >&2
    exit $code
  fi
  "${ADB[@]}" emu kill >/dev/null 2>&1 || true
  EMU="$EMU" timeout 60 bash -c 'while kill -0 "$EMU" 2>/dev/null; do sleep 1; done' || kill "$EMU" 2>/dev/null || true
  exit $code
}
trap teardown EXIT

echo "screens: booting $AVD headless on port $PORT" >&2
"$EMULATOR_BIN" -avd "$AVD" -port "$PORT" -no-window -no-audio -no-boot-anim \
  -no-snapshot -wipe-data -gpu swiftshader_indirect >"$OUT/emulator.log" 2>&1 &
EMU=$!

# ONE bounded wait with a deadline, and it watches for the failure state too:
# an emulator that died looks exactly like one still booting.
ADB_BIN="$ADB_BIN" SERIAL="$SERIAL" EMU="$EMU" timeout 420 bash -c '
  until [ "$("$ADB_BIN" -s "$SERIAL" shell getprop sys.boot_completed 2>/dev/null | tr -d "\r")" = "1" ]; do
    kill -0 "$EMU" 2>/dev/null || { echo "the emulator exited before boot; see the log" >&2; exit 2; }
    sleep 3
  done' || die "boot did not complete (see $OUT/emulator.log)"

echo "screens: installing $APK" >&2
# Uninstall first, so the walk judges what THIS build does rather than what a
# previous one left behind (bl-fcc5). `-r` upgrades in place and keeps the
# app's data AND its scheduled jobs — and the scheduled fetch's job is
# `setPersisted`, so it outlives even a reboot of this AVD. A walk that
# inherited it would report the fetch armed on a build that never armed
# anything. The uninstall is allowed to fail: on a fresh AVD there is nothing
# to remove, which is not an error.
"${ADB[@]}" uninstall "$PKG" >/dev/null 2>&1 || true
"${ADB[@]}" install -r -g "$APK" >/dev/null || die "install failed"

# The seeds — key material and the paint-first cache — are the other half of
# this loop and live in their own file: what a screen IS, against what this
# file DOES with it. Sourced rather than executed, because they speak to the
# same emulator through the same `ADB` and the same `$OUT`.
. scripts/screens-seed.sh
# The platform's own book — the grants it accepted and the job it holds — read
# in its own file for the same seam reason the seeds have one: this file walks
# screens and judges where the walk went, and neither beat there is about a
# screen at all.
. scripts/screens-platform.sh
# The two background lanes — the scheduled fetch and the pocketed foot — in
# their own file again: those beats MOVE the device (home and back, a forced
# job, a re-provisioned leaf, airplane mode), while the file above only reads
# what the platform already holds.
. scripts/screens-background.sh
# How the harness REACHES a control (`screens-reach.sh`). Its own file because
# it answers a question none of the others do: this app's accessibility tree is
# empty (§15.1), so every control in it is unaddressable by name and the only
# way to one is a rectangle the app itself reported plus a synthesized gesture
# at it. That was one control and one gesture until the row menu (§13.5) made
# it two of each, which is the seam.
. scripts/screens-reach.sh
arm_parity

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

: > "$OUT/verdict.txt"

held_grants

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

# 4. The two deeper screens, each selected by the focus stored beside its rows.
seed_cache conversations; relaunch
capture conversations conversations

# 4b. THE ROW MENU (DESIGN §13.5, bl-f97c). Not a sixth screen — the app says
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

# 6. THE PARITY GATE (yog docs/PARITY.md §5, bl-fe4c). Everything above judges
#    where the walk went; this judges what it could REACH. The dumps captured
#    beside each screenshot are the inventory — the `act:<op>` tags egui's
#    accessibility tree carried into the platform layer — and `src/parity`
#    judges them against the corpus roster and the committed exemptions. The
#    report prints whether it passes or fails, because an absence recorded in
#    `parity.toml` is a ledger only if somebody reads it.
echo "screens: judging interface parity" >&2
if PARITY_DUMPS="$OUT" cargo test --test parity -- --ignored --nocapture >"$OUT/parity.txt" 2>&1; then
  sed -n '/^parity:/,$p' "$OUT/parity.txt"
  verdict pass "parity: every control op is reachable or cited"
else
  sed -n '/^parity:/,$p' "$OUT/parity.txt"
  verdict fail "parity: the roster and this walk disagree (see $OUT/parity.txt)"
fi

echo "screens: $OUT" >&2
[ "$FAILED" = 0 ] || die "the walk did not go where it said it would (see $OUT/verdict.txt)"
echo "screens: every screen was reached" >&2
