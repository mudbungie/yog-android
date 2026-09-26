+++
title = "the phone's DHT door re-asks every bootstrap address when the frontier runs dry, silent ones included, burning the query cap; and a put that reached no token holder says no node stored the item (port yog bl-f519)"
created = 1790395787
updated = 1790395787
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
Port of yog bl-f519 (yog REMOTE §13.2, §13.7 ruling 3), as lernie bl-4168 and thrall bl-47bc did. A door re-ask knocks only at bootstrap addresses that answered this walk; a router silent past its first deadline leaves the door for the walk, as a silent node leaves the frontier. A put whose walk found no token holder fails with 'no DHT node near T offered a write token' and sends nothing; 'no DHT node stored the item' stays for holders that were sent the item and none stored it. Tests: a_silent_router_leaves_the_door_after_its_first_deadline (fails under the old rule), put_to_holders_that_never_answer_is_an_error. Amend DESIGN §21.2, which still calls bl-f519 the engine's open question.