+++
title = "A provider's machine is read off the row's own fields, and the seat's split of the name is deleted"
created = 1795157707
updated = 1795157707
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
The server now states, on every providers, catalog and roles row whose provider name selects a registered machine, the two halves of that name: client and row (yog REMOTE 5.7, edition 30, yog bl-471d). Both absent means the server's own row. This seat matched the name against the clients read to learn the machine, a second derivation of the server's own reading. Consume the two fields in the row decoders, build the machine and provider menus from them, label by row, send the whole name back unchanged. Delete the split, the roster it was read against, and everything only they needed. No fallback split: a row without the fields is the server's own under its whole name. Re-vendor the corpus and amend DESIGN 25.