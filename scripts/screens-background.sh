#!/usr/bin/env bash
# **The pocketed foot's beats** (DESIGN §18), sourced by `scripts/screens.sh`.
# Split from `screens-platform.sh` at the seam that file names — these beats
# MOVE the device rather than reading a book it already keeps — and split again
# at bl-b82d, when the second attention rung took this file past the cap.
#
# The seam the second split draws is the SUBJECT, not the size. This file is
# about what reaches this device's HANDS: a foot-grade leaf, a service that
# holds the tool lane open, and the operator act reversed.
# `screens-attention.sh` is about what reaches the OPERATOR — REMOTE §14's two
# rungs, one on the platform's schedule and one on a held connection — and the
# two move for unrelated reasons.
#
# What they share is the question no host test can answer. Both lanes exist so
# that this app reaches an operator, or is reached, while nobody is looking at
# the phone, and both are decided in Rust at the coverage floor
# (`crate::attention`, `crate::pocket`). What no host test can reach is whether
# the PLATFORM accepts the declaration: a service the manifest does not carry,
# one without its binding permission, one whose type the OS rejects, or a
# native symbol the service process cannot resolve all fail identically and
# only on a device.
#
# They read `${ADB[@]}`, `$PKG`, `verdict`, `relaunch`, `capture` and
# `mint_material` from the walk that sources them.

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


boot_beats() {
  # THE DEVICE CAME BACK (bl-d22d, DESIGN §18.8), and it is §18.3's own limit
  # reversed. The decision — what a path alone names, and what a host is made
  # of — is `crate::pocket`, at the coverage floor. What only a device can
  # answer is whether the PLATFORM delivers BOOT_COMPLETED to this receiver,
  # lets a receiver start a `specialUse` foreground service, and lets a
  # process no Activity ever created resolve this app's own classes.
  #
  # IT REBOOTS THE EMULATOR, which is the one gesture in this harness that
  # costs a boot. It runs last for that reason and for one more: every beat
  # above wants a device it did not just restart.
  local svc="ServiceRecord{[^}]*$PKG/\\.Pocket"
  local i

  # 1. A foot leaf and one launch, which is the state a pocketed phone is left
  #    in — the app opened once, then put away. Nothing here asserts; it is
  #    the premise the reboot is judged against.
  mint_material foot; relaunch
  "${ADB[@]}" shell input keyevent KEYCODE_HOME

  # 2. THE REBOOT. `adb reboot` and then the same bounded wait the boot at the
  #    top of this walk uses — an emulator that died looks exactly like one
  #    still booting, so the wait watches for both.
  "${ADB[@]}" reboot >/dev/null 2>&1 || true
  sleep 5
  "${ADB[@]}" wait-for-device >/dev/null 2>&1 || true
  ADB_BIN="$ADB_BIN" SERIAL="$SERIAL" timeout 420 bash -c '
    until [ "$("$ADB_BIN" -s "$SERIAL" shell getprop sys.boot_completed 2>/dev/null | tr -d "\r")" = "1" ]; do
      sleep 3
    done' || { verdict fail "boot: the device did not come back from adb reboot"; return 0; }

  # 3. THE APP IS NEVER OPENED. That is the whole assertion: the receiver is
  #    what armed the service, and a host built in a process with no Activity
  #    in it is what the notification is evidence of. The wait is generous
  #    because BOOT_COMPLETED is delivered after the boot animation, on a
  #    software-GPU emulator, behind every other receiver on the device.
  # THE WAIT IS FOR THE PROMOTION, NOT FOR THE RECORD, and that distinction
  # cost a walk. `startForegroundService` creates the ServiceRecord before the
  # process even exists: a dump taken in that window shows the record with
  # `startForegroundCount=0` and `callStart=false` — the service had not been
  # started yet — so a loop that broke on the record alone judged the beat
  # about two seconds too early and failed on a device that was doing
  # everything right. The dump names the allowance in the same breath
  # (`tempAllowListReason: BOOT_COMPLETED, duration:20000`), which is how that
  # run was read.
  local held=""
  for i in $(seq 1 30); do
    held=$("${ADB[@]}" shell dumpsys activity services "$PKG" 2>/dev/null | tr -d '\r')
    if grep -q "$svc" <<<"$held" && grep -q "isForeground=true" <<<"$held"; then break; fi
    sleep 4
  done
  printf '%s\n' "$held" > "$OUT/boot-services.txt"
  if grep -q "$svc" <<<"$held" && grep -q "isForeground=true" <<<"$held"; then
    verdict pass "boot: the foot came back without the app being opened"
  else
    verdict fail "boot: no promoted service after a reboot (see $OUT/boot-services.txt) — is .Boot declared, and did it reach Pocket.arm?"
  fi

  # 4. AND IT IS A HOST, not just a notification. `foregroundNoti` on the foot
  #    channel is what says this process took a host up: `crate::pocket::line`
  #    answers a line at all only where the leaf is a foot's, and the words
  #    under it are the host's own standing.
  if grep -q "foregroundNoti=Notification(channel=yog.foot" <<<"$held"; then
    verdict pass "boot: the returned hold carries the foot channel's standing row"
  else
    verdict fail "boot: the hold's notification is not on yog.foot (see $OUT/boot-services.txt)"
  fi

  # 5. THE ACT REVERSED, at boot too: a seat leaf, and the device is left the
  #    way every other beat wants it.
  mint_material; relaunch
}
