#!/usr/bin/env bash
# **What the PLATFORM holds**, sourced by `scripts/screens.sh`. The walk next
# door judges where it went — a screen the app said it painted against the
# screen the walk asked for. Neither beat here is about a screen: they ask
# what the OS accepted at install and what it is holding on this app's behalf,
# which is the half of this client no host test can reach at any coverage.
#
# Both read `${ADB[@]}`, `$PKG` and `verdict` from the walk that sources them.

held_grants() {
  # THE GRANTS THE TELEOPERATION CORPUS ASKS FOR, as the INSTALLER sees them
  # (bl-b0a9). A runtime permission is a chain of three: a manifest declaration,
  # an installer that accepted it, and a grant. This install is `-g`, so every
  # runtime permission is granted outright — which makes the emulator the one
  # place the GRANTED half of each tool's gate is observable, and makes a missing
  # manifest line loud: an undeclared permission is not refused at install, it is
  # silently never granted, and the tool then refuses forever on a device where
  # the operator did everything right.
  #
  # What this does NOT do is invoke a tool. Putting an invocation through the
  # host channel to a device needs an engine, a foot leaf and something to fire
  # `/invoke` at it, none of which exists yet — that is bl-05b6's ball, and the
  # refusal halves stay host tests.
  held=$("${ADB[@]}" shell dumpsys package "$PKG" 2>/dev/null | tr -d '\r')
  for want in CAMERA POST_NOTIFICATIONS ACCESS_FINE_LOCATION ACCESS_COARSE_LOCATION; do
    if printf '%s
' "$held" | grep -q "android.permission.$want: granted=true"; then
      verdict pass "grant: $want is declared, accepted and held"
    else
      verdict fail "grant: $want is not held — is it declared in AndroidManifest.xml?"
    fi
  done
}

fetch_beats() {
  # THE SCHEDULED FETCH (bl-fcc5, DESIGN §17), in two beats, and last because
  # they move the app. The decision — what wakes a human and what stays silent —
  # is host-tested at the coverage floor over a real mTLS server; what NO host
  # test can reach is whether the PLATFORM accepted the job and can run it. A
  # manifest without the service, a service without BIND_JOB_SERVICE, a
  # `setPersisted` without RECEIVE_BOOT_COMPLETED, a native symbol the job
  # process cannot resolve: each fails only on a device, and on one where the
  # operator did everything right.
  #
  # IT IS A RESUME THAT IS JUDGED, NOT A LAUNCH, AND THAT IS A PLATFORM FACT.
  # The platform cancels a package's jobs when it is force-stopped and while it
  # is in the stopped state a fresh install leaves it in — and `relaunch` above
  # force-stops before every launch. Measured on a cold-booted emulator:
  # `schedule` returns RESULT_SUCCESS inside the first resume after such a
  # launch and the registration is gone seconds later, the cancellation landing
  # after the call that made it. That race is exactly why `MainActivity` arms on
  # EVERY resume rather than once at startup, so this beat exercises the
  # mechanism the design actually relies on: send the app to the background,
  # bring it back, and wait — bounded — for the registration the resume makes.
  "${ADB[@]}" shell input keyevent KEYCODE_HOME
  sleep 1
  "${ADB[@]}" shell am start -W -n "$PKG/.MainActivity" >/dev/null

  # THE PIPE IS THE TRAP IN EVERY READ BELOW, not the pattern. `dumpsys
  # jobscheduler` is hundreds of kilobytes; `set -o pipefail` is on; and `grep
  # -q` exits the moment it matches, which SIGPIPEs whatever is still writing —
  # so `dumpsys | grep -q` reports 141 on the runs that MATCH, and the beat
  # fails exactly when it should pass. Every read holds the dump in a variable
  # first and matches it with a herestring, which is not a pipeline at all.
  #
  # It reads the REGISTRATION and neither of the other two places this app's job
  # is named. The run LOG outlives the app — the same `<pkg>/.Watch` string is
  # still there after an uninstall — so a bare match would pass on the ghost of
  # a previous walk forever. The pending queue is the one-line `JobStatus{...
  # PERIODIC ...}` form, and that line appears only once the job is being
  # tracked for a run, seconds later again. `JOB #<uid>/<id>: <pkg>/.Watch` is
  # the registry's own header: it appears when `schedule` returns and it goes
  # with the package.
  jobs=""
  for _ in 1 2 3 4 5 6 7 8 9 10; do
    jobs=$("${ADB[@]}" shell dumpsys jobscheduler 2>/dev/null | tr -d '\r')
    grep -q "JOB #.*$PKG/\.Watch" <<<"$jobs" && break
    sleep 2
  done
  if grep -q "JOB #.*$PKG/\.Watch" <<<"$jobs"; then
    verdict pass "fetch: the platform holds a registered job for $PKG/.Watch"
  else
    verdict fail "fetch: no registered job for $PKG/.Watch — is Watch declared, and did onResume arm it?"
  fi

  # The job RUN, forced: the platform binds the service, the job process loads
  # the library, `Java_dev_yog_Watch_probe` resolves, the sweep answers, and our
  # own thread calls jobFinished — which is the line read back. It reaches
  # everything a device can reach without an engine: this walk's material points
  # at a closed port (§15.3), so the sweep's answer here is silence, which is
  # the behaviour a pocketed phone must have and not an absence of proof.
  #
  # The job id is read off the registration rather than written here: it is
  # `Watch.JOB`'s value, and a copy of it in this file would be a second home
  # for one fact. `sed -n 1p` and not `head -1`, for the pipe paragraph's reason
  # — `head` exits early and takes the writer down with it.
  job=$(sed -n "s@.*JOB #u0a[0-9]*/\([0-9][0-9]*\): .* $PKG/\.Watch.*@\1@p" <<<"$jobs" | sed -n 1p)
  "${ADB[@]}" shell cmd jobscheduler run -f "$PKG" "$job" >/dev/null 2>&1 || true
  sleep 3
  ran=$("${ADB[@]}" shell dumpsys jobscheduler 2>/dev/null | tr -d '\r')
  if grep -q "STOP-P: .*$PKG/\.Watch app called jobFinished" <<<"$ran"; then
    verdict pass "fetch: the platform ran the job and it finished"
  else
    verdict fail "fetch: the forced run did not reach jobFinished — see logcat for the service"
  fi
}
