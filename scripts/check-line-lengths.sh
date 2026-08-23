#!/usr/bin/env bash
# The 300-line source cap — delegates to `make line-cap`, the one home of the
# rule (`make lint` runs it too, so the gate path goes through lint). This
# file exists chiefly because bl-speculate's gate fingerprint hashes a fixed
# file list that names it (GATE_FILES, balls src/speculate.rs); keeping it a
# live delegator means the fingerprint names a real gate step, not a stub.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
exec make line-cap
