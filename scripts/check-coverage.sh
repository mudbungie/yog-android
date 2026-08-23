#!/usr/bin/env bash
# Coverage gate step: `make coverage` (the one home of the tarpaulin pin and
# invocation), with stdout HELD — tarpaulin's stdout carries the per-test roll
# and the report, which a passing run should not print, but a FAILING gate
# must name what failed; discarding stdout outright once reduced a named test
# failure to tarpaulin's opaque `Error: "Test failed during run"` (bl-0dff).
# Held, then replayed on failure: quiet when it passes, complete when it does
# not. stderr is held too since bl-673a, but streamed live through `tee` rather
# than replayed — it is where tarpaulin's own log lines land, and one of those
# lines is now read.
#
# THREE OUTCOMES, NOT TWO (bl-673a). A run in which tarpaulin reports being
# SIGNALED is not a verdict about the tree: something outside the gate killed
# the process. Five sightings on GitHub Actions runners, the whole suite green
# up to the kill, and the macOS leg of one of them passed the same tree. So:
#
#   exit 0   the tree covers 100% — a verdict, and the caller may cache a PASS.
#   exit 75  EX_TEMPFAIL: tarpaulin was signaled on BOTH attempts. NOT a
#            verdict. `.github/workflows/speculate.yml` reads this code and
#            records nothing, because a FAIL verdict is keyed by tree and balls'
#            `speculate_run` stops the candidate chain at a stored FAIL forever
#            without rebuilding — so an infrastructure death recorded as a
#            verdict is a permanent false negative for that tree.
#   nonzero  anything else: the gate failed on the tree's own merits.
#
# THE RETRY IS ONCE, AND ONLY FOR THAT CLASS. A real test failure is never
# re-run — re-running it just spends another tarpaulin to learn the same thing.
# An interrupt is not that class either, and this is the trap that says so:
# tarpaulin CATCHES SIGINT and exits with the very message matched below, so an
# operator's Ctrl-C at close would otherwise buy a second multi-minute run.
#
# Fingerprinted by bl-speculate as part of the gate identity (GATE_FILES,
# balls src/speculate.rs) — see scripts/pre-commit.

set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

out="$(mktemp)"
err="$(mktemp)"
trap 'rm -f "$out" "$err"' EXIT
# A signal to THIS script is the same answer as a signal to tarpaulin: the run
# produced no verdict. One code for all three so it cannot matter whether a
# supervisor posts INT, TERM or HUP, nor whether it posts to the process group
# or to tarpaulin alone.
trap 'echo "coverage: interrupted — this run produced NO verdict." >&2; exit 75' \
  INT TERM HUP

# stdout -> $out (held), stderr -> $err AND live to the terminal. Redirection
# order is load-bearing: `2>&1` duplicates stderr onto the pipe while stdout is
# still the pipe, and `>"$out"` only then moves stdout to the file.
coverage_attempt() {
  : >"$out"
  : >"$err"
  make coverage 2>&1 >"$out" | tee "$err" >&2
}

# Tarpaulin's own words for "I was signaled", as a FIXED string. Deliberately
# not a variable: an empty pattern matches every stream, which would classify
# every real failure as infrastructure and vouch for nothing — `make
# beat-audit`'s shape B, in the one place where it would be silent.
signaled() {
  grep -Fq 'Attempting to handle tarpaulin being signaled' "$err" "$out"
}

# What the next sighting needs and none of the five had. Unprivileged and
# portable, and an unreadable kernel log SAYS so rather than looking like an
# absence of evidence — the OOM question is the one this has to answer.
diagnose() {
  echo "coverage: tarpaulin reports being SIGNALED (attempt $1) — a kill from" >&2
  echo "  outside the gate, not a verdict about this tree (bl-673a)." >&2
  free -m >&2 2>/dev/null || echo "  free(1) unavailable on this host" >&2
  ps -eo pid,ppid,rss,comm 2>/dev/null | sort -k3 -rn | head -n 6 >&2 || true
  if dmesg -T >/dev/null 2>&1; then
    echo "  kernel log, OOM lines only:" >&2
    dmesg -T 2>/dev/null | grep -iE 'oom-kill|out of memory|killed process' |
      tail -n 5 >&2 || true
  else
    echo "  kernel ring buffer unreadable (dmesg restricted): no OOM evidence" >&2
  fi
}

code=0
for attempt in 1 2; do
  code=0
  coverage_attempt || code=$?
  # Loop only on the signaled class; a pass and a real failure both leave here.
  { [ "$code" -ne 0 ] && signaled; } || break
  diagnose "$attempt"
done

if [ "$code" -eq 0 ]; then
  exit 0
fi
if signaled; then
  echo "error: tarpaulin was signaled on both attempts — NO verdict for this" >&2
  echo "       tree. Nothing may be cached from this run (bl-673a)." >&2
  exit 75
fi
echo "error: the coverage gate failed. tarpaulin's held stdout follows." >&2
cat "$out" >&2
exit 1
