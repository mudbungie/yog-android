+++
title = "make screens should say when the APK is older than the source it is meant to show"
created = 1788330296
updated = 1788583630
claimant = "Animations-W"
priority = 3
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
bl-243b's own first run was driven against a stale APK: the walk showed a screen with a heading no source in this tree emits, and the wrong conclusion was nearly drawn about the app rather than about the artifact. DESIGN §15.4 records that, and argues a loop which cannot tell you which tree you are looking at will eventually lie to you — then leaves the guard to a sentence in the README saying to build first.

Close the gap: before the walk, compare the APK's mtime against the newest file under src/ and android/ (tracked files only) and say so when the APK is older. A WARNING, not a refusal — the loop is often run deliberately against a known-good artifact while the tree is mid-edit, and a refusal there would be wrong. What must not happen is a green verdict on pictures of a build nobody meant to look at.

Scope it to src/ and android/ on purpose: a docs edit does not change what the APK paints, and a guard that fires on every prose commit is one that gets ignored — which is the failure mode this is trying to avoid, wearing the other face.

Three lines in scripts/screens.sh, beside the existing preflight, whose whole discipline is already that every refusal names the one command that fixes it.