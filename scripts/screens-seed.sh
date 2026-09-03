#!/usr/bin/env bash
# **The seeds**: everything that puts this device into the state a screen needs,
# sourced by `scripts/screens.sh`. The seam is what a screen IS against what
# the loop DOES with it — the walk boots, taps, captures and judges; nothing in
# here knows a screen was captured.
#
# Two seeds and no third, because the app derives everything else from them:
# the key material that makes this device a seat, and the paint-first cache
# whose stored FOCUS selects which of the seat's screens opens. Neither dials
# anything. Both are written into the build directory, never the tree.

# `run-as` is how anything reaches app-private storage, and it works because a
# debug APK is debuggable. Two hops: push into a world-readable staging path,
# then copy in as the app's own uid.
push_app() {           # push_app <local-dir> <files-relative-dest>
  "${ADB[@]}" shell "rm -rf /data/local/tmp/screens && mkdir -p /data/local/tmp/screens"
  "${ADB[@]}" push -q "$1"/. /data/local/tmp/screens >/dev/null
  "${ADB[@]}" shell "run-as $PKG sh -c 'mkdir -p $2 && cp /data/local/tmp/screens/* $2/'"
}
wipe_app() { "${ADB[@]}" shell "run-as $PKG sh -c 'rm -rf files/wire files/cache'"; }

# **Arm the parity inventory** (DESIGN §15.5, bl-fe4c). The app writes the
# `act:` tags it painted only where this directory exists, so creating it is
# the whole of the debug gate — a device nobody armed writes no file and a
# shipped app carries no flag. It is created once, after install, and survives
# `wipe_app`: the inventory is about which controls painted, never about what
# this device is provisioned with.
arm_parity() { "${ADB[@]}" shell "run-as $PKG sh -c 'mkdir -p files/parity && rm -f files/parity/acts.txt'"; }

# What this launch has painted so far, pulled out of app-private storage. Empty
# until the app has painted a tagged control, and `|| true` because a screen
# that carries no control at all is an ordinary state of this walk, not its
# failure.
pull_parity() {       # pull_parity <destination>
  "${ADB[@]}" shell "run-as $PKG cat files/parity/acts.txt 2>/dev/null" > "$1" || true
}

# A CA and one leaf under it. Nothing here is ever a secret: it is minted per
# run, into the build directory, and the engine it would authenticate does not
# exist. It is written where nothing tracked can reach it for the same reason
# the disclosure gate refuses a committed key — a fabricated one still reads
# like one.
# The GRADE is the argument, and it is the only one, because the grade is the
# only thing about a leaf this app reads (`src/leaf.rs`: REMOTE §4.2 puts it on
# the certificate as `OU=foot`, and everything else is a seat). `mint_material
# foot` is therefore the whole of "enrol this device as hands" — there is no
# setting to seed beside it, which is DESIGN §9's derivation doing its job.
mint_material() {      # mint_material [foot]
  local d="$OUT/material"; mkdir -p "$d"
  # An `if`, not `[ ... ] && x=y`: under `set -e` the second form takes the
  # whole list's status, so the ORDINARY seat call would end the walk.
  local subject="/CN=screens-seat"
  if [ "${1:-}" = foot ]; then subject="/OU=foot/CN=screens-foot"; fi
  openssl req -x509 -newkey rsa:2048 -nodes -days 1 -keyout "$d/ca.key" \
    -out "$d/ca.pem" -subj "/CN=screens-ca" >/dev/null 2>&1
  openssl req -newkey rsa:2048 -nodes -keyout "$d/client.key" -out "$d/client.csr" \
    -subj "$subject" >/dev/null 2>&1
  openssl x509 -req -in "$d/client.csr" -CA "$d/ca.pem" -CAkey "$d/ca.key" \
    -CAcreateserial -days 1 -out "$d/client.pem" >/dev/null 2>&1
  # A closed port on the device's OWN loopback. The dial fails fast, the
  # failure is painted, and that is a screen this walk wants a picture of
  # anyway — so the address only has to refuse, and the nearest thing that
  # refuses is the honest one. The emulator's host alias would also refuse and
  # is what a human reaches for; the disclosure gate refuses that literal as a
  # routable address, and it is right to, because no rule can tell one
  # routable quad from another by looking at it.
  printf '127.0.0.1:9' > "$d/address"
  rm -f "$d/ca.key" "$d/client.csr" "$d/ca.srl"
  push_app "$d" files/wire
}

# The cache seed. Its two version stamps are READ OUT OF THE SOURCE that
# defines them rather than restated here: one home per fact, and a bump that
# outruns this script discards the file, which the walk then reports as the
# wrong screen instead of a silent empty list.
seed_cache() {         # seed_cache <depth: roster|conversations|transcript>
  local d="$OUT/cache"; rm -rf "$d"; mkdir -p "$d"
  local version protocol
  version=$(sed -n 's/^const VERSION: u64 = \([0-9]*\);/\1/p' src/cache.rs)
  protocol=$(sed -n 's/^pub const PROTOCOL: u32 = \([0-9]*\);/\1/p' src/hello.rs)
  [ -n "$version" ] && [ -n "$protocol" ] || die "cannot read the cache/protocol versions from src/"
  python3 - "$d/seat.json" "$version" "$protocol" "$1" <<'PY'
import json, sys
out, version, protocol, depth = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), sys.argv[4]
def frame(name):
    with open(f"corpus/reply/{name}.json") as fh:
        return json.load(fh)["frames"][0]
workspaces = frame("workspaces")
body = {"yog-seat-cache": version, "protocol": protocol,
        "focus": {"workspace": None, "agent": None},
        "workspaces": workspaces, "conversations": None, "transcript": None,
        "options": {"workspace": None, "providers": None, "models": {}}}
# The pairing law `cache::read` enforces on the FILE: rows deeper than the
# focus they were asked at are unpaintable, and a file carrying them is
# discarded whole. So each depth carries exactly its own envelopes.
if depth in ("conversations", "transcript", "running"):
    conversations = frame("conversations")
    workspace = workspaces["rows"][0]["workspace"]
    body["focus"]["workspace"] = workspace
    body["conversations"] = conversations
    # **The options the controls row is made of** (DESIGN §14 stores them
    # beside the rows): the providers reply, what the workspace's roles are
    # actually set to, and one provider's models. Without them the row has no
    # provider, so the model selector is disabled and the §9.4 tuning band —
    # which paints only where the picked provider's own row says it takes
    # effort or priority — does not paint at all. They are the engine's own
    # envelopes like everything else here; the worker role's provider in the
    # roles fixture is the one the models map is keyed by.
    roles = frame("roles")
    worker = next(r for r in roles["rows"] if r["role"] == "worker")
    body["options"] = {"workspace": workspace, "providers": frame("providers"),
                       "roles": roles, "models": {worker["provider"]: frame("models")}}
    if depth in ("transcript", "running"):
        rows = conversations["rows"]
        # **Two transcript seeds, because three controls are gated by the
        # engine's own reading of the conversation** (REMOTE §3.1, §9.4): the
        # nudge is offered exactly while nothing is in flight, and the two
        # stop controls exactly while something is. A walk that only ever
        # sees one state cannot observe the other's controls, and unproven is
        # red (yog PARITY §5) — so the walk states which gate it wants and
        # visits both. The `running` seed sets the two booleans the engine
        # puts ON the row to their other lawful value; it invents no field
        # and reads no spelling this codec does not already decode.
        if depth == "running":
            row = rows[0]
            row["stoppable"] = True
            row["stop_children"] = True
        else:
            row = next(r for r in rows if r.get("flight") is None)
        body["focus"]["agent"] = row["root_id"]
        body["transcript"] = frame("transcript")
with open(out, "w") as fh:
    json.dump(body, fh)
PY
  push_app "$d" files/cache
}
