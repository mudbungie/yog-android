+++
title = "a fresh natives cache cannot build: OpenSSL 3.4.0's Configure refuses 'no-server' (Unsupported options), so make apk fails on any box or worktree without a populated target/natives; the main checkout's cache predates the flag"
created = 1792905875
updated = 1792905875
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
tags = ["build"]
+++
Seen 2026-10-25 in a bl worktree (empty target/natives): scripts/natives.sh runs ./Configure ... no-server ... and OpenSSL 3.4.0 answers 'Unsupported options: no-server', so natives, and so apk, fail. The main checkout's cache was built before that flag was added and is never rebuilt, which is why the hook never saw it. Either drop no-server (it is a 3.5+ option) or step OPENSSL_VERSION; either way a fresh cache must build, and a test of natives.sh against an empty cache would have caught it. Workaround used: NATIVES_CACHE=<the main checkout's target/natives>.