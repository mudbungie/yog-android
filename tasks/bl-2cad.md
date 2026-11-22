+++
title = "An act leaves at once: the phone's sends go on their own thread and never wait behind a read"
created = 1795322480
updated = 1795322480
priority = 3
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
Things happen on the server; a seat reads its own replica and may say something at any moment. On the wire (yog REMOTE) a seat's reads and acts are free to run in parallel — only a tool host's invocations read is exclusive.

The defect: every wire crossing this seat makes except the three held lanes runs on the model's one worker thread (src/seat/worker.rs). A command waits in the worker's queue while the worker is inside a pass (up to four reads, each its own connection), a lane's dial, a gesture's read (the records screen opens with five) or the read that follows another act. So a message the operator sends leaves only when whatever read was in flight has finished, and on a slow network that is the connect bound times the reads ahead of it.

The fix, in the desktop seat's shape (its poster beside its asker): a second thread, the poster, owns the acts. The frame's handle sends an act straight to it; it posts at once on its own connection and hands what the act earned back to the worker as a command, where the fold and the read that makes the act visible run as they do today. Reads never wait behind an act either. The focus an act is addressed at is the handle's ordered stream of focus gestures, so a tap followed by a send lands where the tap aimed even while the worker is still reading.

No new dependency, no wire change, no lock.