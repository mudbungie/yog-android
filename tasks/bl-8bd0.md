+++
title = "teleop rung 3, the pocketed foot: a foreground service holds the host lane so invocations reach a phone in a pocket"
created = 1788398991
updated = 1788403442
claimant = "Pocketfoot"
priority = 3
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
bl-eac2 / DESIGN §16.1. Today the tool host serves only while the app process lives; backgrounded, the platform freezes sockets and the foot is absent until the next look — fine for a seat-with-hands, wrong for a device enrolled AS hands (W4). The rung: a foreground service holds the host loop (advertise, ride invocations, complete) so a routed call reaches a pocketed phone.

- Same platform grant shape as REMOTE §14 rung 2 / bl-b82d (a foreground service holding a lane): one service should hold BOTH lanes when both are wanted — coordinate with bl-b82d rather than founding a second service. Unlike b82d this rung is app-only: the invocations read is an existing follow-class wire read, no upstream gate.
- Costs stated per §14.2: a permanent notification, radio wakes, vendor task killers. Off by default; enabling it is an explicit operator act (§12.1's bootstrap discipline).
- Per-tool background reality goes in the tool descriptions the model reads (§6's honesty rule): camera refuses in background, clipboard read is impossible, location wants the separate background grant, ui_* tools work (the accessibility service is exempt from the freeze).
- The redial ladder (bl-8641) already fits a service; the three-state standing line moves into the notification text.

Order behind the paper tools and the listener; serialize on src/host.rs and android/.

---

**Real-device residue** — what the emulator walk cannot answer and a phone must:

1. **Doze for real.** The platform's own rule says a foreground service keeps
   network access in deep Doze (its UID sits above the foreground-service
   threshold), and that the service grants no wakelock — so the CPU still
   suspends and nothing fires on time. An emulator never enters deep Doze on
   its own. What a phone has to show: a still device, screen off, off charge,
   for hours, and whether an invocation routed to it is answered at all, and
   after how long.
2. **Days-long battery.** DESIGN §14.2 prices this rung as a permanent
   notification and radio wakes; that price has never been measured. The number
   wanted is what a foot-provisioned phone spends over a day of holding one
   idle mTLS connection against an engine that hands it no work.
3. **Vendor task killers.** An OEM power manager that stops a foreground
   service is doing what Android's own Active-apps switch does, and this app
   cannot tell them apart. Whether the hold survives overnight on a real
   handset is per-vendor and unanswerable here.
4. **The restricted-settings block does NOT apply to this rung** (unlike rung
   2's listener enable): a foreground service needs no settings act, only the
   two manifest permissions the installer grants outright.

The emulator proved, in seven beats: a seat-grade device holds no service; a
foot-grade leaf makes the platform hold the service; it is promoted
(isForeground=true) with types=0x40000000; it carries a standing ONGOING_EVENT
notification on the foot channel; the process survives `am kill` with the screen
away; the hold stands through an asserted airplane-mode cycle; and
re-provisioning a seat leaf stops it.

**Two beat-spelling traps the walk paid for**, recorded because the next lane
beat will meet them. A ServiceRecord names its component with the manifest's
leading-dot shorthand and continues past it, so a pattern ending in a closing
brace matches nothing — and it failed silently red while every other beat over
the same dump was green. And NO_CLEAR is on a foreground-service notification
only for its first seconds on API 33+, after which the row becomes dismissible;
asserting it makes a beat red or green by how fast the walk reached the dump.
ONGOING_EVENT is the stable half.
