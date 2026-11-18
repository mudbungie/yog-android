+++
title = "A red scheduled workflow opens one issue per streak; the next green run closes it"
created = 1794985640
updated = 1794985640
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
Scheduled workflows (advisories.yml, store-scan.yml) fail silently: a red daily run tells nobody.

Add scripts/alarm.sh red|green (POSIX sh). The open issue titled exactly "unattended run red: <workflow name>" IS the state: queried with gh, never stored. red with none open files one issue that mentions the repository owner and links the run; red with one open does nothing, so a streak notifies once, not once per run. green closes the open one with a comment linking the green run; green with none does nothing. No labels, no config, no state file.

Each scheduled job gains a final if: failure() step (alarm.sh red), an if: success() step (alarm.sh green), GH_TOKEN from github.token, and issues: write at job level.

A self-test drives the real script under a fake gh on PATH, all four cases, wired into make lint.