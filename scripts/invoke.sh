#!/usr/bin/env bash
# **The invocation loop** (bl-05b6): an engine, a foot leaf, a device, and four
# captures read back over the wire. `make invoke` is the door.
#
# WHAT THIS ANSWERS THAT NOTHING ELSE DOES. Every tool this client advertises
# is host-tested and is advertised on the strength of that. What no test here
# had ever done is put an invocation through the tool-host channel to a DEVICE
# and read the capture back — and that is exactly the half a host test cannot
# reach: an activity launch the platform refuses for being in the background, a
# notification that really is in the shade, a battery figure that is this
# device's. `make screens` walks screens and dials nothing; this dials and
# looks at no screen at all.
#
# IT IS NOT `yog gesture`, AND THAT IS THE BALL'S OWN FINDING. The invocation
# mailbox is per-process in-memory state, so an `/invoke` run in a second
# process addresses a different mailbox from the one the phone's parked
# `invocations` read is waiting on. The gesture has to cross the listener,
# which means a seat: `tests/invoke.rs` is that seat, and it is the only piece
# of this that is not shell.
#
# THE ADDRESS IS THE DEVICE'S OWN LOOPBACK, THROUGH `adb reverse`. The
# emulator's host alias is what a human reaches for and the disclosure gate
# refuses that literal — rightly, since no rule can tell one routable quad from
# another by looking. `adb reverse tcp:P tcp:P` makes the device's
# `127.0.0.1:P` the host's, which is a literal this tree may hold and, better,
# is the address the engine's own server certificate is minted for.
#
# EVERY KEY IT MINTS IS UNDER `target/` AND IS DELETED ON THE WAY OUT. The
# world root is a fixture the engine is booted on and torn down with; what
# survives the run is the captures, the verdict and the logs.
set -euo pipefail

AVD=${AVD:-yog-screens}
PORT=${PORT:-5588}
KEEP=${KEEP:-0}
APK=${APK:-android/app/build/outputs/apk/debug/app-debug.apk}
OUT=${OUT:-target/invoke}
YOG=${YOG:-yog}
PKG=dev.yog
# The fixture's own workspace, and the name this device is enrolled under. The
# leaf's COMMON NAME is the identity — never the basename of the file — so this
# one string is what `invoke` addresses and what the registration seats.
WORKSPACE=${WORKSPACE:-home}
FOOT=${FOOT:-yog-foot}
STATE=${STATE:-busy}

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

die() { echo "invoke: $*" >&2; exit 1; }

[ -x "$ADB_BIN" ] || die "no adb at $ADB_BIN — set ANDROID_HOME, or install platform-tools"
[ -x "$EMULATOR_BIN" ] || die "no emulator at $EMULATOR_BIN — sdkmanager --install emulator"
command -v openssl >/dev/null || die "openssl is required (the engine mints with it too)"
command -v python3 >/dev/null || die "python3 is required to read the fixture's answer"
command -v "$YOG" >/dev/null || die "no yog on PATH — build one from a yog checkout:
  cargo install --path <yog checkout>
(or point at one: make invoke YOG=/path/to/yog)"
[ -f "$HOME/.android/avd/$AVD.ini" ] || die "no AVD named '$AVD' — create it once with:
  make screens-avd"
[ -f "$APK" ] || die "no APK at $APK — build one first:
  make apk ABIS=x86_64"
scripts/screens-freshness.sh "$APK"

ADB=("$ADB_BIN" -s "$SERIAL")
rm -rf "$OUT"; mkdir -p "$OUT"

# The two processes this script starts, and the ONLY ones it ever kills. A
# pattern kill would take another agent's emulator — or another agent's
# engine — with it.
EMU=""
ENGINE=""
teardown() {
  local code=$?
  [ -n "$ENGINE" ] && kill "$ENGINE" 2>/dev/null || true
  # The material and the world go, always: a minted key outlives nothing here.
  rm -rf "$OUT/world"
  if [ -n "$EMU" ]; then
    if [ "$KEEP" = 1 ]; then
      echo "invoke: leaving $SERIAL up (--keep); kill it with: kill $EMU" >&2
      exit $code
    fi
    "${ADB[@]}" emu kill >/dev/null 2>&1 || true
    EMU="$EMU" timeout 60 bash -c 'while kill -0 "$EMU" 2>/dev/null; do sleep 1; done' \
      || kill "$EMU" 2>/dev/null || true
  fi
  exit $code
}
trap teardown EXIT

# 1. THE WORLD. `yog fixture` lays a state and states everything a harness
#    needs to dial an engine booted on it — the root, the address it is minted
#    for, and the operator-grade client leaf this run's seat uses. Booting is
#    ours because tearing down is.
echo "invoke: laying the $STATE fixture" >&2
FIXTURE_ROOT="$PWD/$OUT/world" WIRE_HOST=127.0.0.1 "$YOG" fixture "$STATE" > "$OUT/fixture.json" \
  || die "yog fixture refused (see $OUT/fixture.json)"
read -r ROOT ADDRESS ANCHORS CHAIN KEY <<EOF
$(python3 -c '
import json, sys
f = json.load(open(sys.argv[1]))
print(f["root"], f["address"], f["anchors"], f["chain"], f["key"])' "$OUT/fixture.json")
EOF
[ -n "$ADDRESS" ] || die "the fixture stated no address"
WIRE_PORT=${ADDRESS##*:}

# 2. THE FOOT LEAF, minted by the engine's own recipe rather than by hand: the
#    grade is `OU=foot` on the certificate (REMOTE §4.2) and `WIRE_FOOT` is the
#    one word that puts it there. Minting it any other way would be a second
#    spelling of the one fact this device's enrollment IS.
echo "invoke: minting the foot leaf $FOOT" >&2
WIRE_DIR="$ROOT/yog/wire" WIRE_LEAF="$FOOT" WIRE_FOOT=1 "$YOG" wire-certs >/dev/null \
  || die "yog wire-certs refused to issue the foot leaf"

# 3. THE REGISTRATION (REMOTE §1.5, §4.1): an empty file, and its existence is
#    the fact. No gesture manages one — the operator's bootstrap is `mkdir` and
#    `touch`, and so is this harness's.
CLIENTS="$ROOT/yog/world/state/yog/clients/$FOOT/workspaces"
mkdir -p "$CLIENTS" && : > "$CLIENTS/$WORKSPACE"

# 4. THE ENGINE, bounded: booted on the fixture root, killed in the trap, its
#    root deleted with it. The wait watches for the death too — an engine that
#    exited looks exactly like one still starting.
echo "invoke: booting the engine on $ADDRESS" >&2
XDG_DATA_HOME="$ROOT" "$YOG" > "$OUT/engine.log" 2>&1 &
ENGINE=$!
ENGINE="$ENGINE" LOG="$OUT/engine.log" timeout 60 bash -c '
  until grep -q "wire: listening on" "$LOG" 2>/dev/null; do
    kill -0 "$ENGINE" 2>/dev/null || { echo "the engine exited before it listened" >&2; exit 2; }
    sleep 1
  done' || die "the engine never listened (see $OUT/engine.log)"

# 5. THE DEVICE.
echo "invoke: booting $AVD headless on port $PORT" >&2
"$EMULATOR_BIN" -avd "$AVD" -port "$PORT" -no-window -no-audio -no-boot-anim \
  -no-snapshot -wipe-data -gpu swiftshader_indirect >"$OUT/emulator.log" 2>&1 &
EMU=$!
ADB_BIN="$ADB_BIN" SERIAL="$SERIAL" EMU="$EMU" timeout 420 bash -c '
  until [ "$("$ADB_BIN" -s "$SERIAL" shell getprop sys.boot_completed 2>/dev/null | tr -d "\r")" = "1" ]; do
    kill -0 "$EMU" 2>/dev/null || { echo "the emulator exited before boot; see the log" >&2; exit 2; }
    sleep 3
  done' || die "boot did not complete (see $OUT/emulator.log)"

echo "invoke: installing $APK" >&2
"${ADB[@]}" uninstall "$PKG" >/dev/null 2>&1 || true
"${ADB[@]}" install -r -g "$APK" >/dev/null || die "install failed"

# The one line that makes the device's own loopback reach this engine.
"${ADB[@]}" reverse "tcp:$WIRE_PORT" "tcp:$WIRE_PORT" >/dev/null \
  || die "adb reverse refused; the device cannot reach the engine"

# 6. THE SEED — the same two hops `screens` uses, with a FOOT leaf instead of a
#    seat one and an address that answers. `mint_material` is not reused: this
#    leaf is the ENGINE's, issued under the CA the engine trusts, and minting a
#    second CA here would only prove this harness can talk to itself.
. scripts/screens-seed.sh
# The instruments too, for one of them: `relaunch` is force-stop, clear, start
# — the same act this loop needs and the walk already spells. `verdict` rides
# in with it (that file sources `scripts/verdict.sh`), which is why nothing
# here sources it twice.
. scripts/screens-capture.sh
MATERIAL="$OUT/material"; mkdir -p "$MATERIAL"
cp "$ANCHORS" "$MATERIAL/ca.pem"
cp "$ROOT/yog/wire/$FOOT.pem" "$MATERIAL/client.pem"
cp "$ROOT/yog/wire/$FOOT.key" "$MATERIAL/client.key"
printf '127.0.0.1:%s' "$WIRE_PORT" > "$MATERIAL/address"
push_app "$MATERIAL" files/wire
rm -rf "$MATERIAL"

# The verdict file, opened once the instruments that write it are in scope.
: > "$OUT/verdict.txt"

# **Launched twice, and the second one is not superstition.** Installing a
# package makes the platform re-apply its overlays, which lands as a
# configuration change and DESTROYS the activity that is already up — and a
# destroy that catches this app mid-dial hangs in `GameActivity`'s native
# teardown (`NativeCode::~NativeCode` waits for an app thread that is inside a
# wire read), which the platform then ANRs and kills. The walk never meets it
# because its seeds and taps put minutes between the install and its first
# relaunch. So: launch, let the churn happen, and relaunch onto the settled
# package. The hang itself is a defect of this app and is filed as one; the
# harness must not be what discovers it every run (bl-be13).
echo "invoke: launching the app" >&2
"${ADB[@]}" shell "am start -n $PKG/.MainActivity" >/dev/null || die "the app did not launch"
# The two reads below hold their output and match it with a herestring rather
# than piping into `grep -q` (bl-3627): a logcat dump and a `dumpsys` are both
# far past the pipe buffer, `grep -q` exits the instant it matches, and under
# this file's `pipefail` the SIGPIPEd writer decides the status — so the wait
# would never end and the beat would fail on precisely the runs that matched.
PKG="$PKG" ADB_BIN="$ADB_BIN" SERIAL="$SERIAL" timeout 60 bash -c '
  until grep -q "yog\.screen" \
    <<<"$("$ADB_BIN" -s "$SERIAL" logcat -d 2>/dev/null || true)"; do sleep 2; done' \
  || die "the app never said what it painted"
relaunch

# The driver, twice, and the order is the design. `drive` is one named test.
NONCE="invoke-$$-$(date +%s)"
drive() {              # drive <test> <log>
  WIRE_ADDRESS="$ADDRESS" WIRE_ANCHORS="$ANCHORS" WIRE_CHAIN="$CHAIN" WIRE_KEY="$KEY" \
  FOOT_CLIENT="$FOOT" FOOT_WORKSPACE="$WORKSPACE" INVOKE_NONCE="$NONCE" \
  INVOKE_OUT="$PWD/$OUT" \
  cargo test --test invoke -- --ignored --nocapture --exact "$1" >"$OUT/$2.log" 2>&1
}

# 7. THE DEVICE IS A TOOL HOST, proved BEFORE it is pocketed: the foreground
#    service that holds a foot's read open is armed by the activity's own
#    resume (DESIGN §18.1), so the app has to be in front once — and the
#    honest signal that it got that far is the advertisement, never a sleep.
echo "invoke: waiting for $FOOT to advertise" >&2
if drive the_device_advertises_the_set_this_build_offers advertise; then
  verdict pass "the device advertised its set over the wire"
else
  sed -n '/^running /,$p' "$OUT/advertise.log"
  verdict fail "the device never advertised (see $OUT/advertise.log)"
fi

# 8. THE POCKET. Every invocation below is answered with the app in the
#    BACKGROUND, which is the state a teleoperated phone is actually in — the
#    foreground service is what holds the host's read open there (DESIGN §18),
#    and the `open` refusal in the judgement is a platform answer that exists
#    only in this state.
"${ADB[@]}" shell input keyevent KEYCODE_HOME >/dev/null
services=$("${ADB[@]}" shell dumpsys activity services "$PKG" 2>/dev/null | tr -d '\r' || true)
if grep -q "ServiceRecord{[^}]*$PKG/\.Pocket" <<<"$services"; then
  verdict pass "the pocketed foot holds the platform's foreground service"
else
  verdict fail "no foreground service — this device is not pocketed, and the invocations below are a foreground app's"
fi

# 9. THE FOUR, fired by a seat that dials the listener.
echo "invoke: firing four invocations at $FOOT" >&2
if drive four_invocations_reach_the_platform_and_their_captures_come_back driver; then
  verdict pass "the four invocations were answered"
else
  sed -n '/^running /,$p' "$OUT/driver.log"
  verdict fail "the invocations did not come back (see $OUT/driver.log)"
fi

# The app's own logcat, kept beside the run's other evidence and filtered to
# this process: logcat is device-wide, and a whole-device dump would carry
# every other app's business into a file an agent then reads.
APP_PID=$("${ADB[@]}" shell pidof "$PKG" 2>/dev/null | tr -d '\r' | awk '{print $1}')
[ -n "$APP_PID" ] && "${ADB[@]}" logcat -d --pid="$APP_PID" > "$OUT/app.log" 2>/dev/null || true

# 10. THE JUDGEMENT — each capture against the device it came from, which is the
#    whole point of running this on a device at all.
. scripts/invoke-judge.sh
judge_captures

echo "invoke: $OUT" >&2
[ "$FAILED" = 0 ] || die "the invocations did not answer as they should (see $OUT/verdict.txt)"
echo "invoke: every invocation reached the platform" >&2
