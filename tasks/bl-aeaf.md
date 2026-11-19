+++
title = "The self-healing exit of the advisories workflow is green; RUSTSEC-2026-0543 (chrono, unmaintained) is ignored"
created = 1795071710
updated = 1795071710
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
When main moves while the gate runs on the mend branch, the fast-forward push is refused and the next run begins from the new main. That costs a day and no hand, so the landing step now prints its line and ends successfully with nothing landed (the branch is still deleted), and no alarm issue opens. Only the refused push is forgiven: a red ci.yml on the branch, an advisory with no fixed version and every other failure stay red.

Also: RUSTSEC-2026-0543 marks chrono unmaintained, with no fixed version; it arrives through rxing. It is an unmaintained notice, not a vulnerability, so it joins the ignore list in deny.toml with its reason beside it, to be dropped when cargo-deny warns advisory-not-detected.