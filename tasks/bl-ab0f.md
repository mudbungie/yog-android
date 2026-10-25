+++
title = "a fresh natives cache cannot build: OpenSSL 3.4.0's Configure refuses 'no-server' (Unsupported options), so make apk fails on any box or worktree without a populated target/natives; the main checkout's cache predates the flag"
created = 1792905875
updated = 1792906430
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
tags = ["build"]
+++
Root cause (found 2026-10-25): bl-a672's noun rename (engine → server) rewrote OpenSSL's Configure flag no-engine into no-server in scripts/natives.sh; OpenSSL 3.4.0 answers 'Unsupported options: no-server'. The main checkout's target/natives predates the rename, so the hook never saw it; every GitHub release run since 2026-10-23 has failed at 'Build the APK' on a fresh runner, so no release since v0.0.45 has shipped an APK. Fix: restore no-engine; prove by building one ABI into an empty NATIVES_CACHE.