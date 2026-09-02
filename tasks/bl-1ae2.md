+++
title = "the redial test pins the sentence to two of the three transport verbs: a dial into a closing listener's backlog dies at the write, not the connect"
created = 1788310987
updated = 1788310988
claimant = "Patch"
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
Flaked in a close gate one commit after bl-8641 landed: a_dead_engine_is_redialled_with_the_dial_that_failed_standing asserts the standing sentence starts with 'connect ' or 'receive', and one run got 'send: Connection reset by peer'. All three are the transport class, and the class is what the test is about — a redial that lands in the backlog of a listener whose thread has just ended completes the connect and dies on the write instead. Fix: accept every prefix the transport can produce for a channel that would not carry the gesture, and say in the test why the verb is not the assertion.