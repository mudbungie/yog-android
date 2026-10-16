+++
title = "the settings pane and the knob table: scale, help, bubbles, fold, full, steps, dock; settings.json beside the cache"
created = 1792122061
updated = 1792122683
claimant = "Surveying"
parent = "bl-d489"
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
DESIGN 23 item 11 (step 5). src/settings.rs is the table and the typed Settings with the two projections, writing only knobs off their default; shell/settings.rs is the overlay named settings, a row per knob over its sentence, defaults at the foot; /settings is a sheet row; /full and /steps leave the sheet, their state read off the knobs; scale installs egui's zoom factor; dock reads its knob for its first frame. New walk beat tuned.