+++
title = "the redial test pins the sentence to two of the three transport verbs: a dial into a closing listener's backlog dies at the write, not the connect"
created = 1788310987
updated = 1788311003
claimant = "Patch"
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
Flaked in a close gate one commit after bl-8641 landed. Two assertions in host/tests/redial.rs read a MOMENT rather than a fact, and both are the same defect: `Host::standing()` keeps only the latest published standing, so anything transient asserted after `settle` returns is a race.

1. a_dead_engine_is_redialled...: asserts the standing sentence starts with 'connect ' or 'receive'; one run got 'send: Connection reset by peer'. All three are the transport class and the class is the subject — a redial landing in the backlog of a listener whose thread has just ended completes the connect and dies at the write.
2. a_channel_that_dies_mid_answer...: asserts health == Serving after settling on served == 1; the host may lawfully have reached the script's last turn and stopped by the time the test reads.

Fix: accept every transport verb, and assert on what the settled standing CARRIES (the tool it ran) rather than on a state it has already left. A transient state is read by the settle predicate, never after it.