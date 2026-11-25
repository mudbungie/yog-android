+++
title = "Drop the hard-coded server entry at the head of the machine picker once the server makes no model call of its own"
created = 1795582752
updated = 1795582752
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
tags = ["feature"]
+++
yog bl-782f deletes the server's own inference path: a model call runs only on a registered machine's row. The machine picker still begins with a server entry, unconditionally. Remove the entry and its variant. Land after the server release that carries bl-782f.

Tested by: the picker lists only machines.