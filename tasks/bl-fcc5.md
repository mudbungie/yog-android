+++
title = "rung 1 of REMOTE §14: an OS-scheduled fetch and a local notification — attention reaches a pocketed phone with zero engine work"
created = 1788235096
updated = 1788401594
claimant = "Wakebell"
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
What the emulator proved and what it could not.

PROVEN on a headless emulator, as two new `make screens` beats (they live in the new `scripts/screens-platform.sh` — what the platform HOLDS, split from what the walk WENT THROUGH, because screens.sh crossed the 300-line cap and that was the real seam):

- the platform holds a registered periodic job for `dev.yog/.Watch` after a resume;
- forcing that job to run (`cmd jobscheduler run -f`) reaches the service, and the run ends in `STOP-P ... app called jobFinished`, which is our own thread calling it. That path binds the service, starts a process with no Activity in it, loads the library from `Watch`'s own static block, and resolves the native entry.

Also proven by readback of the built APK: `Java_dev_yog_Watch_probe` is exported from `libyog_android.so` on both ABIs (arm64-v8a, x86_64), `Ldev/yog/Watch;` and the `yog.attention` channel id are in the dex, and the merged manifest carries the service with BIND_JOB_SERVICE plus RECEIVE_BOOT_COMPLETED.

NOT REACHABLE without a real device and a live engine, and left as such:

- the OS actually firing the job on its own cadence, and what Doze does to it. The floor is the platform's 15 minutes; the walk forces a run rather than waiting for one, so nothing here measures real latency and nothing should claim to.
- a notification actually posted. Every emulator run answers silence, correctly: the walk's material points at a closed port, so the sweep finds no answer and says nothing. A posted row needs an engine whose roster carries a workspace whose attention rose.
- the battery cost in the field.
- survival across a reboot (setPersisted).

Two platform findings the walk paid for, both written into DESIGN 17.5:

- a job scheduled in the FIRST resume after a force-stop or a fresh install can be cancelled by the platform seconds after `schedule` returns RESULT_SUCCESS. Measured on a cold-booted emulator; a warm one does not show it. This is why arming happens on every resume rather than once at startup — the next resume re-arms, so the exposure is one period at worst.
- `dumpsys | grep -q` under `set -o pipefail` fails on the runs that MATCH: grep exits at the first hit, the writer takes SIGPIPE, the pipeline reports 141. It cost two full walks. Both beats hold the dump in a variable and match with a herestring.

bl-05b6 (on-device invocation proof) is untouched and unblocked by this.
