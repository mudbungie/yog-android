+++
title = "A sign-in waiting for a code is finished on the phone: the lane's authorize key is a door, and the pasted code goes back as login's code (edition 31)"
created = 1795235463
updated = 1795235463
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
yog bl-0127 (REMOTE §8.3, edition 31) lets any seat finish a provider sign-in whose credential lives on another box. This seat's half:

- re-vendor the corpus at edition 31;
- reply `login` reads the optional `authorize` key, a standing fact replaced by every frame;
- request `login` writes the optional `code` key, omitted when starting;
- while a lane states `authorize` and no `outcome`, the sign-in screen shows a door that opens the address in the phone's browser and a paste field with a send;
- the send is `login` for the same pair with the pasted text verbatim, and the same tail is followed;
- the sentences saying a browser flow is finished at the far machine are deleted, because it no longer is.

DESIGN §13.19 and §24 item 5 are amended in the same delivery.