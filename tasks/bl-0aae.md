+++
title = "the leaf key survives an uninstall: allowBackup restores wire/ from the platform's cloud backup, so a reinstalled app comes back enrolled with material that left the box"
created = 1791514342
updated = 1791514342
priority = 1
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
Observed 2026-10-08 while swapping the app on a phone (ops bl-13cb): uninstall, install the new build, launch — and the app came up ENROLLED, painting a cached transcript, with the old leaf's name refused by the engine for re-issue because "the device took it up". The manifest declares no `android:allowBackup`, so the default (true) holds and the platform backs the app's private files up to the operator's cloud account and restores them on reinstall. That puts `files/wire/client.key` — the one copy the design says exists (`enroll` shreds the server side) — in a third party's backup, and makes an uninstall not a revocation.

Decide and land: `allowBackup=false`, or a backup-rules exclusion of `files/wire` (and the cache, which is reconstructible anyway). DESIGN §5 (delivery channels) and §14 (the one cache) should say which files may leave the device and that a backup is not one of the channels.