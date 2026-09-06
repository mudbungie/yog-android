#!/usr/bin/env bash
# **Push this tree's APK to a phone** (bl-128f): `make deploy-phone ADDR=<ip:port>`
# is the door. Build the arm64 APK at the current tree, reach the phone over
# wireless debugging, install it, and answer with the exit code.
#
# ADDR IS AN ARGUMENT AND IS COMMITTED NOWHERE. The wireless-debug address is a
# routable address on a private network and the port rotates on every re-pair;
# both are operator input by nature, and an address in this tree is a
# disclosure the leak gate refuses (AGENTS.md). Pointing this at a different
# phone is a different argument, never an edit — the same severability rule
# yog's `scripts/deploy/seat.sh` holds for its HOST.
#
# THE EXIT CODE CARRIES THE TRUTH, same doctrine as seat.sh. `adb install`
# prints its own failure and still leaves plenty of ways to read success into a
# transcript, so nothing here reports by printing: every step is judged, and a
# deploy that did not install exits non-zero.
#
# IT IS PUSH-ON-DEMAND, NOT UNATTENDED CD, and that is a property of the
# channel rather than a gap in this script. Wireless debugging mints a new port
# on every re-pair and every reboot, so the address cannot be stored anywhere
# and no scheduler can supply it — a human reads it off the phone and types it.
#
# INSTALL IS THE WHOLE ACT. It does not launch the app and does not walk any
# screen: `make screens` is the harness's own door, with its own emulator, and
# a deploy that also drove the app would answer two questions with one exit
# code.
set -euo pipefail

addr=${1:-}
[ -n "$addr" ] || { echo "usage: make deploy-phone ADDR=<ip:port>" >&2; exit 2; }

cd "$(git rev-parse --show-toplevel)"

die() { echo "deploy-phone: $*" >&2; exit 1; }
say() { echo "deploy-phone: $*" >&2; }

# THE SDK MUST BE STATED, and nothing on a developer box states it: the SDK
# installs at the conventional user location and exports no variable, so gradle
# fails to find it and adb is not on PATH either. Default it here exactly the
# way the `screens` target defaults its SDK tools, and EXPORT it — the Android
# Gradle plugin reads `ANDROID_HOME` out of the environment, so defaulting it
# without exporting would fix adb and leave the build broken.
SDK=${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}
export ANDROID_HOME="$SDK"
ADB="$SDK/platform-tools/adb"
[ -x "$ADB" ] || die "no adb at $ADB — set ANDROID_HOME, or install platform-tools"

# GRADLE, by the one rule this repo has (`scripts/gradle.sh`, bl-2a31): an
# explicit `GRADLE=` outright, then PATH, then the newest bin distribution
# under the gradle wrapper's own dists cache. It is CALLED and not restated,
# because the target this one builds through — `make apk` — needs the same
# answer, and two implementations of one rule is the defect that got filed:
# `deploy-phone` found a gradle where `apk` had already given up. The script
# prints the command and nothing else; its refusal names both probes, so the
# only thing left here is to stop.
gradle=$(scripts/gradle.sh) || exit 1

# The build is `make apk`, not a second copy of it. That target is the one
# definition of how this APK is assembled (cargo-ndk into jniLibs, then
# assembleDebug, release profile load-bearing) and a phone needs exactly one of
# its two ABIs.
APK=${APK:-android/app/build/outputs/apk/debug/app-debug.apk}
say "building the arm64 APK with $gradle"
"${MAKE:-make}" apk ABIS=arm64-v8a GRADLE="$gradle" \
  || die "the build failed — nothing was installed"
[ -f "$APK" ] || die "the build answered success but there is no APK at $APK"

# `adb connect` on a device that is already connected says so and exits 0. That
# is a SUCCESS and not an error — a naive check on the message would refuse
# every second run of this target. What it may not do is fail silently, so the
# message is read either way: anything that is not a connection is fatal here,
# before an install is attempted against a device that is not there.
said=$("$ADB" connect "$addr" 2>&1) || die "adb connect failed: $said"
case "$said" in
  *"connected to"*) say "$said" ;;
  *) die "adb connect did not connect: $said" ;;
esac

# The install, and the one thing that proves it: `Success`. The exit code alone
# is not enough — this is the step whose failure modes are loudest and most
# survivable — so both are judged, and the device's own words are what a
# failure reports.
said=$("$ADB" -s "$addr" install -r "$APK" 2>&1) || die "install failed: $said"
case "$said" in
  *Success*) ;;
  *) die "the install did not answer Success: $said" ;;
esac

say "installed $APK"
