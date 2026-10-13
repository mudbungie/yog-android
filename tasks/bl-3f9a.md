+++
title = "phone hermes 1/7: the overlay frame — every bar-painting depth becomes one framed sheet over the chat, closed by its name line or back"
created = 1791867720
updated = 1791867720
parent = "bl-92d3"
priority = 1
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
tags = ["ui"]
+++
DESIGN §22 item 6 and landing step 1. Fold Shell's opened/records/files into one Overlay value (one stands at a time). Re-home the world screens (queue, trail, balls, science, diff, fleet, clients, config, login, help), search hits, records and files into a new shell/overlay.rs frame: SURFACE box, HAIRLINE frame, full width less S, a TOUCH-tall name line (name, subject, close) that closes it, body scrolling inside; the platform back closes it (shell/back.rs rule unchanged). Each screen drops its own bar() call. Rule and composer stand down under an overlay; the field is hidden unless the overlay holds an egui field of its own. shell::act records a tag for a control laid inside the overlay's scroll (reachable by scrolling), mirroring the desktop's bl-eb29. Reached by today's entries; nothing else moves. Files: src/shell/app.rs, app/fields.rs, screens.rs, screens/world.rs + world/*.rs, screens/records.rs, screens/files.rs, screens/search.rs, act.rs, new overlay.rs (pre-split under 200 lines). Tests: make check green; make screens walk beats waiting/trail unchanged by name and capturing the framed sheet; parity gate green.