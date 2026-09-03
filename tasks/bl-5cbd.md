+++
title = "teleop rung 2, the notification listener: the shade as text — the SMS-adjacent surface without the SMS permissions"
created = 1788398990
updated = 1788402059
claimant = "Shaderead"
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
bl-eac2 / DESIGN §16.1. A `notifications` tool reads the device's notification shade as text (app, title, text, when). Needs a NotificationListenerService — the same enable class as dev.yog.InterfaceService: the operator turns it on in system settings or over the debug bridge, and sideloads meet the restricted-settings block a second time (the trap ledger, DESIGN §6 — the failure presents as a setting that will not stick). Advertised whether or not the service is on; a disabled service refuses in band naming the enable (§6's two-tables argument).

This IS the SMS-adjacent surface: 2FA codes and messages arrive as the messaging app's notification text. READ_SMS/SEND_SMS are hard-restricted permissions and are REFUSED as a design shape (§16.1) — the listener answers the read want at one settings act instead of a hard-restricted grant, and agent-sent SMS is the operator's voice on a channel with no undo, not built without an explicit operator ruling.

Scope hard: read-only rung — no dismiss, no reply action. Serialize with the other teleop balls.