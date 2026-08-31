+++
title = "the first-run surface gets branded, tappable choices — and each opens a real screen"
created = 1788147577
updated = 1788147622
claimant = "OrderGreeter"
priority = 9
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
An operator opened the app and found "no buttons or anything to do either of the activities". DESIGN §9 says the first-run surface has **no button, deliberately**, reasoning from REMOTE §1.4 — but §1.4 forbids the app *dialling unauthenticated*, and it never forbade a control. The reading was too strong and the operator has ruled: **the buttons choose and inform; the material still arrives out of channel.**

**What ships.** The cold screen presents the three bootstraps as branded, tappable choices, each named for what it makes this device:

- **Lernie** — the seat. Operate your conversations.
- **Thrall** — the foot. Let conversations use this device's tools.
- **Yog** — the server. Run the engine here. Stays the gated stub with its recorded blockers (DESIGN §10, rungs 3 and 4).

A tap opens a real screen. Lernie and Thrall open the **enrollment screen**: what material is needed, where it goes, and (once the QR envelope lands) the scan/paste path. Yog opens the existing analysis surface — the sentence §10 already writes, now on its own screen rather than crammed into a list.

**Component-derived-from-material stays the law.** A tap stores nothing and chooses nothing durable: it opens the flow that acquires the matching material, and the component that comes up is still read off the leaf on disk (DESIGN §9, `src/bootstrap.rs`). There is no chosen-mode field and there must never be one — the screen is navigation, not state.

DESIGN §9's "no button, deliberately" paragraph is amended by this ball, not worked around.