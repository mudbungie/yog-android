# **What each capture has to say about the device it came from** (bl-05b6),
# sourced by `scripts/invoke.sh`. Its own file for the seam the seeds keep:
# out there is a world being stood up, in here is what an answer must contain.
#
# The four are not four samples of one thing. Each was chosen because its
# evidence is somewhere a host test cannot reach:
#
#   * `shell` answers a value this run minted, which is the whole route proved
#     end to end — engine, device, tool, capture — with nothing else in it;
#   * `device` answers a battery figure, and the platform is asked for the same
#     figure independently, so the answer is THIS device's rather than a
#     plausible sentence;
#   * `notify` leaves a row on the platform's own shade, read back out of
#     `dumpsys notification` on the tools channel;
#   * `open` is REFUSED, by Android and not by this app, because nothing of
#     yog's is on the screen — a refusal that cannot be produced off a device
#     at all, and the one this corpus's honesty rule is about.
#
# `dumpsys notification` redacts the content it prints, which is why the beat
# matches the CHANNEL rather than the title this run chose (DESIGN §15.4 paid
# for that finding once already).

# One field of one capture, read as text.
said() {               # said <tool> <field>
  python3 -c '
import json, sys
with open(sys.argv[1]) as fh:
    print(json.load(fh).get(sys.argv[2], ""), end="")' "$OUT/$1.json" "$2"
}

judge_captures() {
  local missing=0 tool
  for tool in shell device notify open; do
    [ -f "$OUT/$tool.json" ] || { verdict fail "$tool: no capture came back"; missing=1; }
  done
  [ "$missing" = 0 ] || return 0

  # 1. The route itself, proved with a value nothing else could have.
  if [ "$(said shell exit_code)" = 0 ] && grep -q "$NONCE" <<<"$(said shell stdout)"; then
    verdict pass "shell: the device ran it and this run's own value came back"
  else
    verdict fail "shell: the capture does not carry this run's value"
  fi

  # 2. A figure that is this device's, asked of the platform a second way.
  local level answered
  level=$("${ADB[@]}" shell dumpsys battery | sed -n 's/^ *level: *\([0-9]*\).*/\1/p' | head -1)
  answered=$(said device stdout)
  if [ -n "$level" ] && grep -q "battery $level%" <<<"$answered"; then
    verdict pass "device: the capture states this device's own battery level"
  else
    verdict fail "device: the capture says ${answered%%$'\n'*}, the platform says ${level:-nothing}"
  fi

  # 3. The shade, read back from the platform rather than from the answer.
  if [ "$(said notify exit_code)" = 0 ] \
    && "${ADB[@]}" shell dumpsys notification | grep -q "channel=yog.tools"; then
    verdict pass "notify: the post stands in the shade on the tools channel"
  else
    verdict fail "notify: nothing on the tools channel (said: $(said notify stderr))"
  fi

  # 4. The platform's own refusal, in band, naming the act that fixes it.
  if [ "$(said open exit_code)" != 0 ] \
    && grep -q "not in front" <<<"$(said open stderr)"; then
    verdict pass "open: the platform refused a background launch and said so in band"
  else
    verdict fail "open: expected the background refusal, got exit $(said open exit_code)"
  fi
}
