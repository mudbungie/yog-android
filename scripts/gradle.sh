#!/usr/bin/env bash
# **Which gradle this repo builds with, decided in one place** (bl-2a31).
# Prints the resolved command on stdout and nothing else; a box with no gradle
# anywhere gets the sentence naming both probes on stderr and a non-zero exit.
#
# ONE RULE, TWO CALLERS. `make apk` and `scripts/deploy-phone.sh` both need a
# gradle, and until this file existed they answered differently: the script
# probed `PATH` and then the wrapper's own distribution cache, while the target
# defaulted `GRADLE ?= gradle` and died with `gradle: not found` on the very
# box the script built on — after the long cargo-ndk half had already run. Two
# implementations of one rule, and the complete one was in the target nobody
# runs first. `deploy-phone` builds THROUGH `make apk`, so the two were also
# resolving the same question twice in one command.
#
# THE TWO PLACES A WORKING GRADLE ACTUALLY LIVES. `PATH` first, because a box
# with a system gradle means it. Then the newest bin distribution under the
# gradle wrapper's own dists cache: a box that has ever run a wrapper has a
# complete distribution there and no `gradle` command, which is the state this
# rule was written on. There is deliberately no wrapper committed here — the
# wrapper is a jar, and the disclosure gate refuses any binary it cannot read
# — so the cache another project's wrapper left behind is the second probe
# rather than the first.
#
# `GRADLE=` NAMES ONE OUTRIGHT and outranks both. It is a command, so an
# absolute path and a bare name resolve the same way, and the override is
# spelled identically at both doors: `make apk GRADLE=…`, `make deploy-phone
# ADDR=… GRADLE=…`.
#
# A FAILURE NAMES BOTH PROBES: "gradle not found" with only one of them stated
# sends you looking in the wrong place.
set -euo pipefail

dists="${GRADLE_USER_HOME:-$HOME/.gradle}/wrapper/dists"
gradle=$(command -v "${GRADLE:-gradle}" 2>/dev/null) || gradle=""
if [ -z "$gradle" ]; then
  gradle=$(ls -d "$dists"/gradle-*-bin/*/gradle-*/bin/gradle 2>/dev/null | sort -V | tail -1) \
    || gradle=""
fi
if [ -z "$gradle" ] || [ ! -x "$gradle" ]; then
  echo "no gradle: '${GRADLE:-gradle}' is not on PATH and no bin distribution
  lives under $dists — install one, or name it:
  GRADLE=/path/to/gradle" >&2
  exit 1
fi

printf '%s\n' "$gradle"
