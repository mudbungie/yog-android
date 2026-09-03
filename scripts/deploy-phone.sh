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

# GRADLE, in the two places a working one is actually found. `GRADLE` keeps the
# Makefile's override semantics: it names a command, so an absolute path and a
# bare name resolve the same way, and an operator's `GRADLE=/path/to/gradle`
# wins over everything below. Then PATH. Then the wrapper's own distribution
# cache, because a box that has ever run a gradle wrapper has a complete
# distribution under it while having no `gradle` on PATH — which is the state
# this target was written on. Newest wins; a failure names both probes, since
# "gradle not found" with only one of them stated sends you looking in the
# wrong place.
dists="${GRADLE_USER_HOME:-$HOME/.gradle}/wrapper/dists"
gradle=$(command -v "${GRADLE:-gradle}" 2>/dev/null) || gradle=""
if [ -z "$gradle" ]; then
  gradle=$(ls -d "$dists"/gradle-*-bin/*/gradle-*/bin/gradle 2>/dev/null | sort -V | tail -1) \
    || gradle=""
fi
[ -n "$gradle" ] && [ -x "$gradle" ] \
  || die "no gradle: '${GRADLE:-gradle}' is not on PATH and no bin distribution
  lives under $dists — install one, or name it:
  make deploy-phone ADDR=... GRADLE=/path/to/gradle"

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
