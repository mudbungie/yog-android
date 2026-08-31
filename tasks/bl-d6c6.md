+++
title = "the local server bootstrap: what running the engine on the phone actually needs"
created = 1788138444
updated = 1788139021
claimant = "OrderDroid"
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
The third component the bl-15bd ruling names: the yog **server** — holder of
the world, the balls, the conversations — running on the phone. Allowed,
deliberate, non-default.

This ball is the honest evaluation and whatever of it is landable. The
question is not "does rustls build for aarch64-linux-android" (it does, with
`ring`, which is the feature set this crate already pins for the same reason).
The question is the engine's substrate: yog composes a **nested world** and
hands it to every child it spawns, and the children are `git`, the task
tracker and the agent-loop engine. A phone has none of them, an app uid cannot
install them into a place a `PATH` lookup finds, and the engine's own boot
mints wire certificates by shelling to `openssl`.

So the chain to state, rung by rung, is: the crate cross-compiles → the world
founds → a child process can be spawned at all under an app uid → the specific
children exist on the device → the engine's boot-time provisioning finds its
one recipe. A component that cleared the first rung and failed the fourth
would be an app that launches a server which refuses every act it is given,
which is worse than an app that says what it needs.

Until the chain is walked, the bootstrap is a **gated surface that states its
dependency chain and starts nothing**. That is not a stub standing in for
work: an operator choosing this bootstrap is choosing to run an engine, and
the app telling them exactly what is missing is the honest answer to that
choice. The alternative — a button that starts a process which cannot serve —
is the failure mode §12's "ship inert" ruling exists to refuse.