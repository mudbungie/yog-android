+++
title = "the app's rendezvous path logs nothing: no ladder rung, no punch, no held-line event reaches logcat, so a live dial can only be observed from socket state"
created = 1790657358
updated = 1790657358
priority = 3
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
Same defect yog fixed on the engine side in bl-355c (one line per event, never an address, key, salt or sealed byte): log through android_logger at the rungs — direct rung tried/refused/timed out, presence read (seq), call written (nonce, endpoint count, families, acks), punch started/landed (peer family)/expired, held line kept/dropped (reason class), ping discarded. Families and counts only. Tests on the fake-DHT bench with a captured sink. lernie and thrall need the same; filed there.