+++
title = "A dropup still shows one row: the scroller inside is handed what the list took last time, and its floor is one row"
created = 1795072787
updated = 1795072787
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
Seen on the phone after bl-5e0b: a provider row answering ten models painted one. bl-5e0b removed the cap taken from the list's last measured height, which was half of it. The other half: the popup's area hands the scroller the height the list took the last time it was laid out, and the scroller never asks for more than it is handed unless that is under its own floor, which defaults to about one row. A list first laid out before its rows arrived is therefore one row tall for good. The floor is set to the room, so the scroller is the smaller of its rows and the room whatever it is handed. DESIGN 25 item 7 corrected in the same ball.