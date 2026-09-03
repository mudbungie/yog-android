+++
title = "teleop rung 2, the notification listener: the shade as text — the SMS-adjacent surface without the SMS permissions"
created = 1788398990
updated = 1788402438
claimant = "Shaderead"
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
bl-eac2 / DESIGN §16.1. A `notifications` tool reads the device's notification shade as text (app, title, text, when). Needs a NotificationListenerService — the same enable class as dev.yog.InterfaceService: the operator turns it on in system settings or over the debug bridge, and sideloads meet the restricted-settings block a second time (the trap ledger, DESIGN §6 — the failure presents as a setting that will not stick). Advertised whether or not the service is on; a disabled service refuses in band naming the enable (§6's two-tables argument).

This IS the SMS-adjacent surface: 2FA codes and messages arrive as the messaging app's notification text. READ_SMS/SEND_SMS are hard-restricted permissions and are REFUSED as a design shape (§16.1) — the listener answers the read want at one settings act instead of a hard-restricted grant, and agent-sent SMS is the operator's voice on a channel with no undo, not built without an explicit operator ruling.

Scope hard: read-only rung — no dismiss, no reply action. Serialize with the other teleop balls.

---

Landed. What the emulator proved, and what only a phone can: the walk asserts the fresh-install state (no notification access — what the tool refuses from), the platform's Live listener list after the enable over the debug bridge, and a posted notification standing in the shade while the listener is bound. A component that does not exist is Allowed just as readily and never appears among the live ones, which is why the beat reads Live and not Allowed — that is the discrimination, measured. Static readback beside it: aapt2 xmltree shows the service carrying BIND_NOTIFICATION_LISTENER_SERVICE and the listener action, dexdump shows Ldev/yog/ShadeService;.

Real-device residue, none of it reachable here: (1) the restricted-settings block, which bites a sideload and presents as a toggle that will not stick — over the bridge the enable is unconditional, so only a phone meets it, and the refusal names the appops act anyway; (2) that a model actually READS the shade through an invocation, which needs an engine, a foot leaf and something to fire /invoke — bl-05b6's ball, and the retention ruling means no other trace exists to look for; (3) real notification shapes from a messaging app (grouped rows, a summary with no text, a code in the expanded body rather than the collapsed line) — the formatter takes EXTRA_TEXT and falls back to EXTRA_BIG_TEXT, unverified against a real messaging app; (4) whether the operator-facing settings path reads the same on a skin that is not AOSP.

Two harness findings paid for: cmd notification post splits on whitespace with no quoting of its own, so a quoted argument silently becomes two and the row lands under an unexpected tag; and dumpsys notification redacts the content it prints, which is the platform agreeing that a shade is not material to leave lying in a dump.
