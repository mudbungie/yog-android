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
# WHICH TREE ARE THESE PICTURES OF (bl-c3fc). The last preflight beat and the
# only one that warns instead of dying: an APK older than the tracked source
# under src/ and android/ is announced, and the walk runs anyway. Its own file
# because it is the one preflight question a host test can drive both
# directions of — see `scripts/screens-freshness.sh` for why it warns.
scripts/screens-freshness.sh "$APK"

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

# The INSTRUMENTS (`screens-capture.sh`): relaunch, probe, settle, verdict and
# capture — how this harness reads the app and writes a row about it, in their
# own file because they answer a different question from the walk that spends
# them (bl-46e6, forced by the cap under bl-35bd). Nothing in there knows which
# screen is being visited; nothing out here knows how a screen is read.
. scripts/screens-capture.sh

: > "$OUT/verdict.txt"

held_grants

# THE WALK ITSELF (`screens-walk.sh`): where to go and what each beat proves.
# Its own file for the reason the three above have one — this file boots a
# device and judges a run, and that one is an itinerary.
. scripts/screens-walk.sh

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
