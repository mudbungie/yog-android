# **What each capture has to say about the device it came from** (bl-05b6),
# sourced by `scripts/invoke.sh`. Its own file for the seam the seeds keep:
# out there is a world being stood up, in here is what an answer must contain.
#
# The seven are not seven samples of one thing. Each was chosen because its
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
#     at all, and the one this corpus's honesty rule is about;
#   * `http` fetches a page over TLS, which proves the one thing no part of
#     this harness stands in for: the DEVICE's own networking stack, verifying
#     against the DEVICE's own trust store. It is also the only beat here that
#     leaves this box, so a failure names the emulator's network before it
#     names the tool;
#   * `curl` and `busybox` are the packaged executables, spent BY NAME through
#     the shell tool — which is the only thing that can prove the whole chain
#     of them at once: legacy packaging put real files in `nativeLibraryDir`,
#     the installer left them executable, `dev.yog.Kit` linked them into the
#     app's storage, and the shell tool put that directory on the child's
#     PATH. Any one of those four wrong is "not found" here.
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
  for tool in shell device notify open http curl busybox; do
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
  # The dump is held and matched with a herestring, never piped into `grep -q`:
  # this file is sourced by one that sets `pipefail`, and a `grep -q` that exits
  # on its match SIGPIPEs the `dumpsys` still writing — so the beat would fail
  # on exactly the runs where the channel was there (bl-3627).
  local shade
  shade=$("${ADB[@]}" shell dumpsys notification 2>/dev/null || true)
  if [ "$(said notify exit_code)" = 0 ] \
    && grep -q "channel=yog.tools" <<<"$shade"; then
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

  # 5. The device's own network stack, and its own trust store with it: an
  # https page fetched, its status line first, its headers under it. Matched on
  # the status rather than on the body, because what is being proved is the
  # request — the body is example.com's to change.
  if [ "$(said http exit_code)" = 0 ] \
    && grep -q "^HTTP 200" <<<"$(said http stdout)"; then
    verdict pass "http: the device fetched an https page through the platform's own TLS"
  else
    verdict fail "http: no 200 came back (exit $(said http exit_code), said: $(said http stderr))"
  fi

  # 6. The packaged curl, resolved by NAME off the PATH the shell tool hands
  # its child, doing TLS against the device's own certificate directories —
  # which is what `SSL_CERT_DIR` buys, since this curl carries no CA bundle.
  if [ "$(said curl exit_code)" = 0 ] \
    && grep -q "^HTTP/[0-9.]* 200" <<<"$(said curl stdout)"; then
    verdict pass "curl: the packaged executable ran off the PATH and verified an https host"
  else
    verdict fail "curl: no 200 (exit $(said curl exit_code), said: $(said curl stderr))"
  fi

  # 7. And busybox, over plain http — the one it is built for, because its own
  # small TLS does not verify certificates and is deliberately not compiled in.
  if [ "$(said busybox exit_code)" = 0 ] \
    && grep -q "Example Domain" <<<"$(said busybox stdout)"; then
    verdict pass "busybox: the packaged applet ran off the PATH and fetched a page"
  else
    verdict fail "busybox: nothing came back (exit $(said busybox exit_code), said: $(said busybox stderr))"
  fi
}
