+++
title = "conversation acts reach the phone: interrupt, fork, retarget, flag"
created = 1788399000
updated = 1788405832
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
Ops: interrupt, fork, retarget, flag. The controls row already owns every conversation-level act (DESIGN §13.2) — these four join it or its overflow. interrupt is the chat staple (cut off mid-work and send this text — the composer's send while a flight stands is a natural home, decide at the glass); fork wants a picking surface over history (a transcript row's act); retarget and flag are one-tap acts with a reason field. Full-seat re-scope, bl-eac2 / DESIGN §16.2 (operator ruling 2026-09-03: the phone is a full seat, not a chat-first companion). Closing this ball: grow the codec for exactly these ops (decision-table rows in tests/conformance/expect.rs move from Refuses to spelled — the codec grows per consumer, never speculatively), paint the controls with act: tags, extend the make screens walk where a new screen appears, and DELETE this group's parity.toml lines (the stale-exemption assertion enforces it). Serialize against the other §16.2 group balls — they share expect.rs, parity.toml and the shell surfaces (see the store's serialize-shared-surface guidance).

---

OPERATOR RULING (2026-09-03): the surface for the conversation acts is a LONG-PRESS context menu on the conversation row — egui synthesizes a secondary click from a touch long-press, so this is the same context-menu design the desktop seat is landing on its rows (lernie store, filed the same day; read its DESIGN menu-idiom section once landed for wording/idiom parity, but derive the implementation from THIS tree). The acts to surface are this ball's roster as filed; the menu is the gesture path.
