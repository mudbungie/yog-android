+++
title = "release-plz never releases this crate: is_publishable() reads the MANIFEST's publish key, so git_only cannot lift publish = false"
created = 1788751280
updated = 1788751280
priority = 1
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
tags = ["usability-r1"]
+++
`main` carries 0.0.2 and no `v0.0.2` tag exists: every run of the release job answers `nothing to release`, so no Release is cut, `releases_created` is false, and `release-apk` — the job that signs the APK a phone installs — is skipped. The channel has published exactly one asset (0.0.1) and cannot publish a second.

## The root cause, measured

bl-ca8f read release-plz's warning that `git_only` and `publish` cannot both be true and set `publish = false` in `release-plz.toml`. That is not the gate. release-plz 0.3.162's release command does:

    let packages = project.publishable_packages();
    if packages.is_empty() { info!("nothing to release"); ...

and `publishable_packages` filters on `Package::is_publishable()`, which is cargo metadata's reading of the **manifest's** `publish` key. `Cargo.toml` says `publish = false`, so the package is filtered out before any tag, version or config is consulted. Confirmed by running release-plz 0.3.162 locally against this tree with four configurations — `git_only` alone, `publish = false` alone, both plus explicit `git_tag_enable`/`git_release_enable`, and a per-package block — all four answer `should release: Yes` and then `nothing to release`.

So no configuration reaches it: the only lever release-plz offers is the manifest key, and that key is the one AGENTS.md rule 6 and DESIGN §20 make an invariant.

## The fix

Own the tag and the Release in the workflow. The release job's single step becomes: read the manifest version, do nothing if `v<version>` already exists, else `gh release create` at this run's sha — writing the same two outputs (`releases_created`, `releases`) the `read-tag` and `release-apk` jobs already read, so nothing downstream moves. release-plz keeps the half it does correctly: opening and refreshing the version-bump PR.