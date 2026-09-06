#!/usr/bin/env bash
# **The two attention rungs' beats** (DESIGN §17; yog REMOTE §14), split from
# `screens-background.sh` at bl-b82d on the seam of what they are ABOUT: these
# are how a workspace that wants the operator reaches a pocketed phone, and
# that file is how a tool call reaches its hands.
#
# Rung 1 is the scheduled fetch and rung 2 is the held lane. They share a
# memory file, a notification id and a rise rule, so their beats share a file:
# an edit that moved one and not the other should be read beside the other.
#
# What they can prove here is the PLATFORM's half — a job the OS really holds
# and runs, a service the OS really promotes, a consent switch the OS really
# keeps. What rises and what stays silent is `crate::attention`, at the
# coverage floor; what only an engine beside the device can show is the wake
# itself, and that is measured by hand (§17.6).
#
# They read `${ADB[@]}`, `$PKG`, `$OUT`, `verdict` and `relaunch` from the walk
# that sources them.

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
  # so a `dumpsys` piped into a `grep -q` reports 141 on the runs that MATCH,
  # and the beat fails exactly when it should pass. Every read holds the dump
  # in a variable first and matches it with a herestring, which is not a
  # pipeline at all.
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

lane_beats() {
  # THE HELD ATTENTION LANE (bl-b82d, DESIGN §17.6; yog REMOTE §14 rung 2),
  # and it is the half no host test can reach. WHAT rises and what stays quiet
  # is `crate::attention`, at the coverage floor over a real held server; what
  # only a device can answer is whether the PLATFORM accepts a seat holding the
  # same `specialUse` service the foot holds, and whether the consent gate this
  # rung is off by default behind is a switch the OS really keeps.
  #
  # IT RUNS LAST, after `pocket_beats` has put a SEAT leaf back on this device.
  # The lane is a seat's — a foot may not ask the world anything (REMOTE §4.2)
  # — so this is the one order in which both rungs' beats are about the device
  # they are for. The exemption is granted and TAKEN BACK inside these beats,
  # so nothing after them inherits a device that holds a service.
  #
  # Every read holds its dump in a variable and matches with a herestring, for
  # `fetch_beats`' reason: a `dumpsys` piped into a `grep -q` reports 141 under
  # pipefail on exactly the runs that match.
  local svc="ServiceRecord{[^}]*$PKG/\\.Pocket"
  local services i

  # 1. OFF BY DEFAULT, and the gate is the one this rung adds. A seat device
  #    with no unrestricted-battery exemption holds nothing, which is REMOTE
  #    §14.2's "off by default, enabled as an explicit operator act" measured
  #    rather than asserted in prose.
  "${ADB[@]}" shell cmd deviceidle whitelist "-$PKG" >/dev/null 2>&1 || true
  relaunch
  services=$("${ADB[@]}" shell dumpsys activity services "$PKG" 2>/dev/null | tr -d '\r')
  if grep -q "$svc" <<<"$services"; then
    verdict fail "lane: a seat with no battery exemption is holding a service — the consent gate is not gating"
  else
    verdict pass "lane: a seat with no battery exemption holds no service"
  fi

  # 2. THE OPERATOR ACT, performed the way the platform keeps it. In settings
  #    it is "Unrestricted battery usage"; `cmd deviceidle whitelist +<pkg>` is
  #    the same list, which is what makes this rung's consent something a
  #    harness can grant and an operator can revoke in one place.
  "${ADB[@]}" shell cmd deviceidle whitelist "+$PKG" >/dev/null 2>&1 || true
  local granted
  granted=$("${ADB[@]}" shell cmd deviceidle whitelist 2>/dev/null | tr -d '\r')
  if grep -q "$PKG" <<<"$granted"; then
    verdict pass "lane: the platform holds the battery exemption for $PKG"
  else
    verdict fail "lane: the exemption did not take — the beats below judge nothing"
  fi
  relaunch

  local held=""
  for i in 1 2 3 4 5 6 7 8 9 10; do
    held=$("${ADB[@]}" shell dumpsys activity services "$PKG" 2>/dev/null | tr -d '\r')
    if grep -q "$svc" <<<"$held"; then break; fi
    sleep 2
  done
  printf '%s\n' "$held" > "$OUT/lane-services.txt"
  if grep -q "$svc" <<<"$held" && grep -q "isForeground=true" <<<"$held"; then
    verdict pass "lane: a consenting seat holds $PKG/.Pocket in the foreground"
  else
    verdict fail "lane: no promoted service for a consenting seat (see $OUT/lane-services.txt)"
  fi

  # 3. WHAT THE OPERATOR READS, and which channel it is on — the fact that
  #    tells this rung's row apart from the foot's in the one place both could
  #    appear. The content is not asked for: the platform redacts it, and the
  #    words are `crate::pocket::attending`'s, tested at the coverage floor.
  if grep -q "foregroundNoti=Notification(channel=yog.attention.held" <<<"$held"; then
    verdict pass "lane: the hold carries its standing row on the held-attention channel"
  else
    verdict fail "lane: the hold's notification is not on yog.attention.held (see $OUT/lane-services.txt)"
  fi

  # 4. THE ACT REVERSED. Taking the exemption back is the operator's own stop,
  #    and the next resume is where it lands — there is no in-app switch, for
  #    the reason there is no foot toggle (DESIGN §18.2).
  "${ADB[@]}" shell cmd deviceidle whitelist "-$PKG" >/dev/null 2>&1 || true
  relaunch
  local stopped=""
  for i in 1 2 3 4 5 6 7 8 9 10; do
    stopped=$("${ADB[@]}" shell dumpsys activity services "$PKG" 2>/dev/null | tr -d '\r')
    if ! grep -q "$svc" <<<"$stopped"; then break; fi
    sleep 2
  done
  if grep -q "$svc" <<<"$stopped"; then
    verdict fail "lane: the hold outlived the exemption that authorized it"
  else
    verdict pass "lane: taking the battery exemption back stopped the hold"
  fi
}
