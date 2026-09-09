+++
title = "returning to the app flashes a DNS failure banner that clears itself: a redial that has not failed is not a failure"
created = 1788934192
updated = 1788934640
claimant = "Cantaloups-A8"
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
tags = ["usability-r3"]
+++
Operator report 2026-09-08: tabbing back into the app shows a DNS resolution failure that clears every time; the dial had not actually failed. On resume the first dial attempt races the network stack waking. Ship: a channel that was up before the pause redials silently; an error banner appears only when a dial has failed after the redial's own retries (or the engine refused), stated with what failed; a transient resolve error during the first N seconds after resume is a 'reconnecting…' state (Working ink), not an error (Error ink). Test on the emulator by toggling airplane mode / backgrounding the app.