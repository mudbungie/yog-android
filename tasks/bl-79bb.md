+++
title = "phone hermes 4/7: the status rule — the bar leaves the chat; mark, state, waiting, model and name as tappable segments above the prompt; the controls band becomes the model overlay"
created = 1791867722
updated = 1791869476
claimant = "PhoneTwo"
parent = "bl-92d3"
priority = 1
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
tags = ["ui"]

[[blockers]]
id = "bl-3edc"
on = "claim"
+++
DESIGN §22 items 1, 2 and 10, landing step 4. A TOUCH-tall band above the prompt: the drawn mark (still the mark rectangle, toggles the configuration surface), state · elapsed in the state's ink (ready at rest; the banner's sentence in its accent while one stands, a tap puts it down), the waiting count in the attention accent (opens the queue), the model (opens the model overlay), spend absent until read, — name at the right (opens the switcher). Drop order spend, model, elapsed; state, count and name never drop; the name elides; each segment at least TOUCH wide. Put the pure segment/drop logic in a host-tested module. The bar is no longer painted on the chat (kept on the configuration, foot and enrollment surfaces). The controls band is deleted: provider, model, effort and priority move unchanged in rule into a /model overlay of four TOUCH rows whose lists still obey shell::place; the SMALL-type band exception goes. Files: src/shell/mark.rs, screens/banner.rs, transcript.rs, controls.rs + controls/*.rs, new shell/status.rs and pure src/status.rs (+tests), app/probe.rs (rule-name, rule-model, rule-waiting rectangles). Tests: host tests for the drop order at 320/360/420 points; walk beats settings/back-to-roster tap the mark on the rule, new beat modelled; place.rs sweep green; parity gate green (providers, models, model, effort, priority observed in the model overlay).