+++
title = "dht::flight::land expires a query before reading the answer already in the socket: a stalled client thread turns a heard node into a silent one"
created = 1790736633
updated = 1790736633
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
Seen on the noodlezoo builder under load (bl-6bac): land() does flight.retain(deadline > now) and returns None before calling recv when the deadline passed while THIS thread was descheduled — even a stand-in transport whose recv errors at once was read as silence. On a phone the client thread is descheduled routinely (background throttling), so a walk on the live mainline can end dark with the answers sitting in the UDP buffer. Candidate: read what has already landed (recv with the 1 ms floor) before expiring, so a deadline bounds the wait, not the read. bl-6bac widened the tests' deadlines instead — this is the production half.