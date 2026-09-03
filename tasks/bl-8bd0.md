+++
title = "teleop rung 3, the pocketed foot: a foreground service holds the host lane so invocations reach a phone in a pocket"
created = 1788398991
updated = 1788402480
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