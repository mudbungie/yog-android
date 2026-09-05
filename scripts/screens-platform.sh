#!/usr/bin/env bash
# **What the PLATFORM granted and bound**, sourced by `scripts/screens.sh`. The
# walk next door judges where it went — a screen the app said it painted
# against the screen the walk asked for. No beat here is about a screen: they
# ask what the OS accepted at install and what it will bind, which is the half
# of this client no host test can reach at any coverage.
#
# **The two BACKGROUND lanes are next door again** (`screens-background.sh`),
# and the seam is what the beat has to do to ask its question. Everything here
# reads a book the platform already keeps — the package's grants, the listener
# registry — and changes nothing. A lane beat has to MOVE the device: send it
# home and bring it back, force a job, re-provision the leaf, cycle airplane
# mode. Two files, so the read-only half stays runnable in any order and the
# half with side effects states its own.
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
    # A herestring and not a pipe: `dumpsys package` is large, `grep -q` exits
    # the instant it matches, and under this harness's `pipefail` the SIGPIPEd
    # writer would report the read FAILED on exactly the runs where the grant
    # was held (bl-3627, reasoned at the head of `leak-selftest.sh`).
    if grep -q "android.permission.$want: granted=true" <<<"$held"; then
      verdict pass "grant: $want is declared, accepted and held"
    else
      verdict fail "grant: $want is not held — is it declared in AndroidManifest.xml?"
    fi
  done
}

shade_beats() {
  # THE NOTIFICATION LISTENER (bl-5cbd, DESIGN §16.1 rung 2), in three beats,
  # and after the screens because the last of them puts a row in the status bar
  # of every picture taken after it.
  #
  # WHAT ONLY A DEVICE CAN ANSWER HERE. The tool's pure half — the advertised
  # element, the cap, the refusal a build with no Android gives — is host-tested
  # at the coverage floor. What no host test can reach is whether the PLATFORM
  # accepts this service AS a notification listener: a manifest without the
  # `<service>`, one without BIND_NOTIFICATION_LISTENER_SERVICE, one without the
  # NotificationListenerService intent-filter action, or a class the dex does
  # not carry all fail identically and only on a device — the enable appears to
  # work and nothing ever binds.
  #
  # THE ENABLE OVER THE BRIDGE IS THE OPERATOR'S ACT, NOT THE APP'S. The app
  # never grants itself anything (§16.1's consent surface, and §6's trust model
  # unchanged); what the design names beside the settings toggle is "a trusted
  # device does it over the physically attached debug bridge", and this walk is
  # that device. `-g` at install cannot do it — notification access is not a
  # runtime permission, which is the whole reason this rung answers the read
  # want that READ_SMS would otherwise be asked for.
  local component="$PKG/$PKG.ShadeService"

  # 1. THE STATE EVERY DEVICE STARTS IN, and the one the tool refuses from. A
  #    fresh install holds no notification access, and this beat is what keeps
  #    the beat below honest: without it, an AVD that carried the enable in from
  #    somewhere else would pass beat 2 while proving nothing.
  local before
  before=$("${ADB[@]}" shell settings get secure enabled_notification_listeners 2>/dev/null | tr -d '\r')
  if grep -q "$component" <<<"$before"; then
    verdict fail "shade: this device already granted notification access before the walk asked"
  else
    verdict pass "shade: a fresh install holds no notification access — the refusal state"
  fi

  # 2. THE BIND, which is the enabled path as far as anything without an engine
  #    reaches. `Live notification listeners` is the platform's list of
  #    listeners it has actually CONNECTED — not `Allowed`, which is only the
  #    setting written back. Our component appearing there means the system
  #    resolved the class out of this APK's dex, accepted the permission on the
  #    declaration, and bound it; `ShadeService.live` is non-null at that moment,
  #    which is the branch the tool takes when it answers rather than refuses.
  #
  #    Held in a variable and matched with a herestring, never `dumpsys | grep
  #    -q`: the fetch beats' paragraph above is the reason, and it bites the
  #    same way here.
  "${ADB[@]}" shell cmd notification allow_listener "$component" >/dev/null 2>&1 || true
  local live=""
  for _ in 1 2 3 4 5 6 7 8 9 10; do
    live=$("${ADB[@]}" shell dumpsys notification 2>/dev/null | tr -d '\r')
    live=$(sed -n '/Live notification listeners/,/Snoozed notification listeners/p' <<<"$live")
    grep -q "ComponentInfo{$component}" <<<"$live" && break
    sleep 2
  done
  if grep -q "ComponentInfo{$component}" <<<"$live"; then
    verdict pass "shade: the platform bound $component as a live notification listener"
  else
    verdict fail "shade: the platform never bound $component — is the service declared with BIND_NOTIFICATION_LISTENER_SERVICE and the listener intent-filter?"
  fi

  # 3. THE SUBJECT OF THE READ, staged and standing while the listener is bound.
  #    A notification posted from the bridge is an ordinary row of the shade, so
  #    this is the material rung 2 exists to read, present at the same moment as
  #    the reader.
  #
  #    WHAT IT DOES NOT PROVE, said plainly: that THIS APP read it. Nothing here
  #    calls the tool — putting an invocation through the host channel needs an
  #    engine, a foot leaf and something to fire it, which is bl-05b6's ball and
  #    not this one's. The retention ruling is why no other evidence exists to
  #    look for: the listener keeps nothing and logs nothing, so a shade read
  #    leaves no trace anywhere for a walk to find, and one that did would be the
  #    defect rather than the proof.
  #
  #    TWO THINGS THE PLATFORM MADE THIS BEAT PAY FOR. `cmd notification post`
  #    takes `[-t <title>] <tag> <text>` and splits on whitespace with no
  #    quoting of its own, so an argument with a space in it silently becomes
  #    two and the row lands under a tag nobody expected — every word here is
  #    one token on purpose. And `dumpsys notification` REDACTS the content it
  #    prints (`android.title=String [length=3]`), which is why the match is on
  #    the record's `tag=`: the one part of a notification the dump states
  #    outright. That redaction is the platform agreeing with this rung's
  #    retention ruling — a shade is not material to leave lying in a dump.
  "${ADB[@]}" shell cmd notification post -S bigtext -t yog-walk shade-beat the-body-a-listener-reads >/dev/null 2>&1 || true
  sleep 2
  local shade
  shade=$("${ADB[@]}" shell dumpsys notification 2>/dev/null | tr -d '\r')
  if grep -q "tag=shade-beat" <<<"$shade" && grep -q "ComponentInfo{$component}" <<<"$shade"; then
    verdict pass "shade: a posted notification stands in the shade with yog's listener bound to it"
  else
    verdict fail "shade: the posted notification and the bound listener were not both there"
  fi
}

