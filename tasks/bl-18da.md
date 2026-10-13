+++
title = "phone hermes 6/7: the transcript idiom and the panels — gutter glyphs, the trail tree under one fold, the conversation and seat panels at the head"
created = 1791867723
updated = 1791869883
claimant = "PhoneChat"
parent = "bl-92d3"
priority = 1
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
tags = ["ui"]

[[blockers]]
id = "bl-79bb"
on = "claim"
+++
DESIGN §22 item 4, landing step 6. Gutter glyph replaces the RULE stripe (› brand operator, ¦ model, · peer/tool/machinery); a turn's calls under one fold head + tool calls (N), each a painted stem (reuse rows/thread.rs strokes) with • name(preview) and ✔/✖; response elbow; HAIRLINE between turns; fold toggles stay TOUCH targets. Projection rules of §7 unchanged (crate::rows stays pure; any new grouping is a host-tested rows function). At the head: the conversation panel (name, workspace, model, subtree path, long ones behind + folds) or, with nothing focused, the seat panel shared with the switcher's head. Files: src/shell/chat.rs, transcript.rs, src/rows/*.rs, new shell/chat/panel.rs. Tests: rows host tests for the trail grouping; walk beats transcript and running show the panel and gutter; parity green.