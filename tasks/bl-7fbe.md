+++
title = "the identity rename deferred by bl-5331: applicationId dev.yog, Java package, crate yog_android, repo yog-android, release asset name — decide whether the id ever moves, and what an installed phone loses if it does"
created = 1791514343
updated = 1791514343
priority = 3
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
bl-5331 renamed the label and every operator-facing sentence and kept the id, on DESIGN §9's own reasoning: the release channel (§20) made `applicationId` a one-way door, and a phone keeps its update path only under the id it was installed with. What is left over, for a deliberate decision rather than a drift: the package id and Java package `dev.yog`, the JNI class names, the crate name `yog_android`, the repository name `yog-android`, the GitHub release asset `yog-android-<v>.apk`, the User-Agent the update fetch sends, and the thread/channel ids. Moving the id is a new app to Android (every installed phone re-enrols by hand); moving the repo name redirects on GitHub but changes the URL the installed app polls (`Release.java`). Either both move together with a migration note, or neither does and this ball closes with the decision written into DESIGN §9.