+++
title = "the tab strip from the second open conversation, and the drawer swipe that opens the switcher"
created = 1792122061
updated = 1792122468
claimant = "Surveying"
parent = "bl-d489"
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
DESIGN 23 items 1 and 2 (step 4). crate::tabs holds the open set per run; every focus fills it; shell/tabs.rs paints the sideways-scrolling strip over the transcript from the second tab; a tap focuses, × closes to the left neighbour. crate::drawer decides a left-edge drag; shell::back opens the switcher on it. New walk beat tabbed.