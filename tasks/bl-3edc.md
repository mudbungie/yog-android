+++
title = "phone hermes 3/7: the switcher — roster and conversation list fused into one overlay, the seat panel and the update row at its head, back from the chat opens it"
created = 1791867721
updated = 1791867721
parent = "bl-92d3"
priority = 1
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
tags = ["ui"]

[[blockers]]
id = "bl-0d15"
on = "claim"
+++
DESIGN §22 items 7 and 11, landing step 3. One overlay named conversations: seat panel lines at its head (identity, tools line, the §20.4 update offer — its once-per-process check now starts on the first seat paint), one heading row per workspace with the attention mark and agent count, the focused workspace open with its conversations as the §13.20 tree (roster.rs and rows/thread.rs unchanged) and + new under it (closes with the field in start mode); a tap on a heading focuses it; a tap on a conversation focuses it and closes. Long press on a conversation keeps the §13.5 row menu. Back rule: overlay or sheet open -> close; chat -> open the switcher; switcher -> leave the app. The roster's world entries are deleted (verbs since 2/7). Files: src/shell/screens.rs, screens/rows.rs, screens/rows/menu.rs, screens/world.rs, shell/back.rs, shell/update.rs, new shell/switcher.rs. Tests: walk beats roster+conversations replaced by switching (update row and threaded rows visible), row-menu long-presses inside the switcher; the probe names the first conversation row as before; parity gate green (act:conversations, act:seen, row acts).