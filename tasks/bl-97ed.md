+++
title = "the re-punch rung can never land against the engine: it binds a fresh punch port per climb and writes no call, while the engine punches only a new call's nonce toward the port that call named"
created = 1790731604
updated = 1790731604
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
Found on bl-2ba5 (DESIGN §21.8). Rung 3 of the ladder (ladder/climb.rs) re-punches at the RAM-cached engine endpoints from Punch::bind(0) and writes no call. The engine (yog src/wire/rendezvous) accepts on its punch listeners only inside the window of a call whose nonce it has not punched, and it connects toward the endpoints and port that call named. So a re-punch is answered only by coincidence: live, every one expired after 35 s with the engine saying 'call nonce N already punched — no punch'. Options to rule on: bind the punch port once per entry for the run (lernie's shape, already named as not done in §21.3) and have the engine re-punch a known caller's cached endpoints on some signal; or drop rung 3 and go direct to a fresh call, which costs one DHT round trip. Measure which the operator wants before building. bl-2ba5 already skips rung 3 for a dial beside a line still out.