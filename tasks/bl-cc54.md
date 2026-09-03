+++
title = "re-vendor the wire at PROTOCOL 8, and say it out loud when a re-presentation actually restored the set: reply/advertised gained a required `wrote` boolean"
created = 1788399978
updated = 1788401636
claimant = "Droidvendor"
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
**Filed from the engine side**: yog bl-66d4 raised yog's `PROTOCOL` from 7 to 8,
so this build's pin no longer matches and the preface is refused by exact
equality before any gesture is sent.

## What moved

One shape. `reply/advertised` gained **`wrote`**, a required boolean:

    {"kind": "advertised", "ok": true, "wrote": true|false}

`false` means the engine found the presented set identical to what it had stored
and wrote nothing — the ordinary answer on every reconnect and every re-assertion.
`true` means it wrote the document.

It is required rather than optional-absent-reads-false on purpose: absent would
read as *"nothing was restored"*, the reassuring answer, on exactly the build too
old to tell.

Nothing else changed at 8.

## Why it exists

The advertised set is keyed on the client identity and any connection bearing
the certificate may replace it. The engine refuses a replacement while that
client holds a parked read (yog bl-1462), which covers an idle host — but a host
executing a tool holds no parked read, so a rival can blank its set in that
window. Re-presenting restores it, and until now the receipt was identical
either way, so the restoration was silent and two processes claiming one
machine's name reached no log on either side.

## The act

Raise the pin to 8, re-vendor `corpus/` from a yog checkout at that number, and
replay it. yog's REMOTE §5.1 carries the reasoning verbatim.

## This component is a foot, so the field is not just decode work

`foot.rs`'s `advertise` presents this machine's set. Once it decodes `wrote`,
a `true` arriving on any presentation **after** the channel's first is this
machine learning its set was blanked or replaced while it was absent, and
saying so is the whole remedy. A `true` on the first presentation of a channel
is ordinary and says nothing.