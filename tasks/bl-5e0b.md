+++
title = "The rule gains a machine selector: providers are listed per machine without the qualification, each pick opens the next list, a list is as tall as its rows, and a start shows the conversation it founded"
created = 1795072363
updated = 1795072363
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
Since yog REMOTE 5.7 a workspace's providers listing carries, after the server's own rows, one row per provider per registered machine offering its model access, named row@client. The rule painted that as one flat provider list.

Ruling: seat -> servers/workspaces -> machines (the server itself, and each machine offering its model access) -> providers -> models, and each level is its own menu.

- A machine selector stands before the provider. Its choices are the server itself plus each client that qualifies a row of the listing. The provider list shows only the chosen machine's rows, without the qualification. The name the assignment writes stays the qualified one; the machine is derived from the provider in force and the listing, and the only new state is the machine a menu is browsing before a provider is picked.
- The split is the server's own rule: the suffix is matched against the workspace's registered clients (the clients read), longest wins, a row must stand before the joiner. The listing does not carry the client as a field, so with no roster in hand every name reads as the server's own, whole.
- Folds bl-c64c: picking a machine opens the provider list, picking a provider opens the model list, the role is written at the model pick.
- Folds bl-64e1: a list takes the height its rows need, up to the room above the rule.
- Folds bl-82d7: a start whose receipt names the conversation focuses it once the list carries that name.

DESIGN amended in the same ball.