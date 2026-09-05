#!/usr/bin/env bash
# **Which tree are these pictures of?** (bl-c3fc) — the one question the walk
# could not answer about itself.
#
# `make screens` builds nothing on purpose (DESIGN §15.4): a target that
# silently rebuilt would hide which tree the pictures are of. That is the right
# call and it leaves a gap, which bl-243b's own first run fell into — the walk
# was driven against a stale APK, it showed a screen with a heading no source
# in the tree emits, and the wrong conclusion was nearly drawn about the app
# rather than about the artifact. A loop that cannot tell you which tree you
# are looking at will eventually lie to you.
#
# So this says so. It is a WARNING and never a refusal: the loop is often run
# deliberately against a known-good artifact while the tree is mid-edit, and
# refusing there would be wrong. What must not happen is a green verdict on
# pictures of a build nobody meant to look at.
#
# THE SCOPE IS `src/` AND `android/`, AND TRACKED FILES ONLY. A docs edit does
# not change what the APK paints, and a guard that fires on every prose commit
# is one that gets ignored — which is this guard's own failure mode wearing the
# other face. Tracked-only for a harder reason: the APK itself is BUILT under
# `android/`, so an enumeration of the worktree would compare the artifact
# against its own build tree and report every fresh build as stale.
#
# Its own file rather than three lines of `scripts/screens.sh`, for that file's
# standing seam reason: everything in the walk needs an emulator and this needs
# a git tree and two mtimes, so this is the half a host test can drive both
# directions of (`tests/freshness.rs`).
set -euo pipefail

APK=${1:-}
ROOTS=${ROOTS:-src android}

die() { echo "screens: $*" >&2; exit 2; }

[ -n "$APK" ] || die "usage: screens-freshness.sh <apk>"
[ -f "$APK" ] || die "no APK at $APK to judge the age of"

# `stat` on the index's own list, sorted newest first. One fork, not one per
# file — and `git ls-files` reads the INDEX, so a staged edit counts before it
# is committed, the same reason `make line-cap` reads it.
newest=$(git ls-files -z -- $ROOTS \
  | xargs -0 -r stat -c '%Y %n' 2>/dev/null \
  | sort -rn | head -1)

# The empty-set guard, `line-cap`'s own: a scan that enumerates nothing must
# fail loudly rather than pass as "not stale" forever.
[ -n "$newest" ] || die "enumerated no tracked file under $ROOTS — the scan is broken, not the tree"

apk_at=$(stat -c '%Y' "$APK")
src_at=${newest%% *}
src_file=${newest#* }

when() { date -d "@$1" '+%Y-%m-%d %H:%M:%S'; }

if [ "$apk_at" -lt "$src_at" ]; then
  echo "screens: WARNING: this APK is OLDER than the source it is meant to show" >&2
  echo "screens:   apk    $(when "$apk_at")  $APK" >&2
  echo "screens:   newest $(when "$src_at")  $src_file" >&2
  echo "screens: the walk runs anyway — these are pictures of an older build. Rebuild with:" >&2
  echo "screens:   make apk ABIS=x86_64" >&2
else
  echo "screens: the APK is newer than every tracked file under $ROOTS" >&2
fi
