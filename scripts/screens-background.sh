#!/usr/bin/env bash
# **The two background lanes**, sourced by `scripts/screens.sh`: the scheduled
# fetch (DESIGN §17) and the pocketed foot (DESIGN §18). Split from
# `screens-platform.sh` at the seam that file names — these beats MOVE the
# device rather than reading a book it already keeps, so they run last and each
# one states what it moved.
#
# What they have in common is the question. Both lanes exist so that this app
# reaches an operator, or is reached, while nobody is looking at the phone — and
# both are decided in Rust at the coverage floor (`crate::attention`,
# `crate::pocket`). What no host test can reach is whether the PLATFORM accepts
# the declaration: a service the manifest does not carry, one without its
# binding permission, one whose type the OS rejects, or a native symbol the
# service process cannot resolve all fail identically and only on a device.
#
# They read `${ADB[@]}`, `$PKG`, `verdict`, `relaunch`, `capture` and
# `mint_material` from the walk that sources them.

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

pocket_beats() {
  # THE POCKETED FOOT (bl-8bd0, DESIGN §18), and it is the half of the rung no
  # host test can reach. The decision — is this device hands, what does the
  # shade say in each state — is `crate::pocket`, pure and at the coverage
  # floor. What only a device can answer is whether the PLATFORM accepts this
  # service: a manifest without the FOREGROUND_SERVICE_SPECIAL_USE line throws
  # SecurityException out of `startForeground`, a missing type attribute throws
  # a different one, and a `<property>` the parser rejects fails the install —
  # each of them only on a device, and on one where the operator did everything
  # right.
  #
  # Every read holds its dump in a variable and matches with a herestring, for
  # `fetch_beats`' reason: a `dumpsys` piped into a `grep -q` reports 141 under
  # pipefail on exactly the runs that match.
  # A ServiceRecord names the component the way the INTENT did — with the
  # manifest's leading-dot shorthand, `dev.yog/.Pocket`. That is not the
  # spelling `shade_beats` matches next door: a notification-listener
  # ComponentInfo is written out in full, `dev.yog/dev.yog.ShadeService`,
  # because the settings string that enabled it was. Same package, two
  # spellings, and the walk paid a run to learn which is which.
  # It stops AT the component and does not close the brace: a record reads
  # `ServiceRecord{6461e49 u0 dev.yog/.Pocket c:dev.yog}`, so a pattern ending
  # in `}` demands one immediately after the class and matches nothing. That
  # cost a walk, and it failed in the one direction a beat must not — silently
  # red while every other beat over the same dump was green.
  local svc="ServiceRecord{[^}]*$PKG/\\.Pocket"

  # 1. THE STATE A SEAT DEVICE IS IN, which is the off-by-default half. The
  #    walk is still on the seat leaf it has carried since step 2, and a seat
  #    phone is a seat WITH hands beside it (§16.1): its host lives while the
  #    app does and nothing holds a lane pocketed. Without this beat, a device
  #    that held the service unconditionally would pass beat 2 while proving
  #    the opposite of the design.
  local services
  services=$("${ADB[@]}" shell dumpsys activity services "$PKG" 2>/dev/null | tr -d '\r')
  if grep -q "$svc" <<<"$services"; then
    verdict fail "pocket: a SEAT-grade device is holding a foreground service — the material gate is not gating"
  else
    verdict pass "pocket: a seat-grade device holds no foreground service"
  fi

  # 2. THE ENROLMENT, and it is a certificate rather than a setting. Minting
  #    this device a foot-grade leaf is the whole of the operator act (§16.1's
  #    consent gate 1, DESIGN §9's derivation): there is nothing else to seed.
  mint_material foot; relaunch
  capture pocketed foot

  local held=""
  local i
  for i in 1 2 3 4 5 6 7 8 9 10; do
    held=$("${ADB[@]}" shell dumpsys activity services "$PKG" 2>/dev/null | tr -d '\r')
    if grep -q "$svc" <<<"$held"; then break; fi
    sleep 2
  done
  if grep -q "$svc" <<<"$held"; then
    verdict pass "pocket: the platform holds $PKG/.Pocket for a foot-grade device"
  else
    verdict fail "pocket: no service for $PKG/.Pocket — is Pocket declared, and did onResume arm it?"
  fi

  # 3. THE PROMOTION AND THE TYPE. `isForeground=true` is the platform saying
  #    it accepted `startForeground`, which is what puts this process above the
  #    Doze network threshold; the type is what Android 15 judges the service
  #    by, and `specialUse` is the one with no six-hour clock on it.
  if grep -q "isForeground=true" <<<"$held"; then
    verdict pass "pocket: the platform promoted it to the foreground"
  else
    verdict fail "pocket: the service is not foreground — did startForeground throw? (see logcat)"
  fi
  # The type, in whichever of the platform's three spellings this build prints:
  # the constant's name, its bit (1 << 30), or that bit in hex. The dump itself
  # is kept beside the walk's other evidence, because a spelling that changes
  # under an SDK bump should be readable rather than guessed at twice.
  printf '%s\n' "$held" > "$OUT/pocket-services.txt"
  if grep -Eqi "SPECIAL_USE|1073741824|0x40000000" <<<"$held"; then
    verdict pass "pocket: the platform recorded the specialUse type"
  else
    verdict fail "pocket: no specialUse type in the service record (see $OUT/pocket-services.txt) — is FOREGROUND_SERVICE_SPECIAL_USE declared?"
  fi

  # 4. WHAT THE OPERATOR ACTUALLY READS, and it is read off the SERVICE record
  #    rather than the shade: `foregroundNoti=` is the notification this hold
  #    is carrying, which ties the two together — a row in the shade and a
  #    service holding one are different facts, and the second is the one that
  #    matters (a foreground service without a notification is a service the
  #    platform kills). The CONTENT is not asked for: the platform redacts it
  #    (`shade_beats`' third beat learned that) and the words are
  #    `crate::pocket`'s, tested at the coverage floor.
  #
  #    `ONGOING_EVENT` is the standing kind — what `setOngoing(true)` sets —
  #    and it is the half that says this notification is evidence rather than a
  #    message. It is asserted and `NO_CLEAR` is NOT: the platform adds that one
  #    only for the first seconds of a foreground service's life, after which an
  #    API 33+ device lets the operator dismiss the row, so a beat on it is red
  #    or green by how fast the walk got to the dump.
  if grep -q "foregroundNoti=Notification(channel=yog.foot" <<<"$held" \
    && grep -q "flags=[^ ]*ONGOING_EVENT" <<<"$held"; then
    verdict pass "pocket: the hold carries a standing notification on the foot channel"
  else
    verdict fail "pocket: the hold's notification is not a standing one on yog.foot (see $OUT/pocket-services.txt)"
  fi

  # 5. IT OUTLIVES THE SCREEN, WHICH IS THE WHOLE RUNG. Home sends the app to
  #    the background and `am kill` asks the platform to reap the package's
  #    BACKGROUND processes — a process running a foreground service is not
  #    one, so surviving it is the exact property this rung buys. The pid is
  #    read either side: a service that came back under a new process would
  #    look identical to one that never went away.
  local before after
  before=$("${ADB[@]}" shell pidof "$PKG" 2>/dev/null | tr -d '\r')
  "${ADB[@]}" shell input keyevent KEYCODE_HOME
  sleep 1
  "${ADB[@]}" shell am kill "$PKG" >/dev/null 2>&1 || true
  sleep 3
  after=$("${ADB[@]}" shell pidof "$PKG" 2>/dev/null | tr -d '\r')
  if [ -n "$before" ] && [ "$before" = "$after" ]; then
    verdict pass "pocket: the held process survived a background kill with the screen away"
  else
    verdict fail "pocket: the process did not survive backgrounding (was '$before', now '$after')"
  fi

  # 6. A NETWORK FLAP IS THE ORDINARY CASE ON A PHONE, so the hold must not be
  #    a casualty of one. The flap is asserted before it is judged: a beat that
  #    toggled nothing would report a survival of nothing (the walk's material
  #    points at a closed port either way, so the LANE's own answer to a flap
  #    is `host::tests`' subject, not this one's — what is judged here is that
  #    the platform kept the service across it).
  "${ADB[@]}" shell cmd connectivity airplane-mode enable >/dev/null 2>&1 || true
  sleep 3
  local mode
  mode=$("${ADB[@]}" shell settings get global airplane_mode_on 2>/dev/null | tr -d '\r')
  if [ "$mode" = "1" ]; then
    verdict pass "pocket: the walk put this device into airplane mode"
  else
    verdict fail "pocket: airplane mode did not take (got '$mode') — the flap beat below judges nothing"
  fi
  "${ADB[@]}" shell cmd connectivity airplane-mode disable >/dev/null 2>&1 || true
  sleep 5
  local flapped
  flapped=$("${ADB[@]}" shell dumpsys activity services "$PKG" 2>/dev/null | tr -d '\r')
  if grep -q "$svc" <<<"$flapped" && grep -q "isForeground=true" <<<"$flapped"; then
    verdict pass "pocket: the hold stood through a network flap"
  else
    verdict fail "pocket: the hold did not survive the flap"
  fi

  # 7. IT STOPS WHEN THE DEVICE STOPS BEING HANDS, which is the same act
  #    reversed: a seat-grade leaf, and the next resume takes the hold down.
  #    There is no in-app switch to press, and this is what its absence costs
  #    and buys — the material is the switch, everywhere.
  mint_material; relaunch
  local stopped=""
  for i in 1 2 3 4 5 6 7 8 9 10; do
    stopped=$("${ADB[@]}" shell dumpsys activity services "$PKG" 2>/dev/null | tr -d '\r')
    if ! grep -q "$svc" <<<"$stopped"; then break; fi
    sleep 2
  done
  if grep -q "$svc" <<<"$stopped"; then
    verdict fail "pocket: the hold outlived the foot leaf that authorized it"
  else
    verdict pass "pocket: re-provisioning a seat leaf stopped the hold"
  fi
}
