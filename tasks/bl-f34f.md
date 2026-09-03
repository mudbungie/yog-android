+++
title = "teleop rung 1, the paper tools: device state, clipboard set, notify, open"
created = 1788398990
updated = 1788399585
claimant = "Papertool"
priority = 2
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
bl-eac2 / DESIGN §16.1 (operator ruling 2026-09-03: working teleoperation tools on the phone). Four built-in tools join the src/tools.rs table, all runnable inside the app's current architecture (foreground-serving foot, no new platform service):

- `device` — battery level and charging state, network class, free storage. Plain reads, no permission.
- `clipboard_set` — write text to the clipboard. Write is believed unrestricted where read is background-blocked; PROBE on a current device before advertising (§6's own discipline: bounded by what an app uid can actually do, established by probe).
- `notify` — post a local notification (title, text). POST_NOTIFICATIONS is a runtime permission on API 33+; ride the bl-d815 permission-result hook. A denied grant refuses in band naming the settings act. This is a TOOL an agent invokes to reach the operator's pocket — distinct from REMOTE §14's attention rungs, which are the seat's own machinery (bl-fcc5/bl-b82d); say so in the tool description.
- `open` — fire a typed VIEW/SEND intent (a URL or shared text). No permission, but background activity launch is platform-refused since API 29: refuses in band when the app is not foreground, naming the fact. Typed, never a generic run-any-intent payload — REMOTE §5.2 refused the wrapper meta-tool twice and the reasoning binds here.

Every tool: three advertised facts (REMOTE §5.1), capture as text, host-testable table per DESIGN §6, refusal sentences that name the one operator act that fixes them (the bl-5710 editorial rule, §16.1). No wire change, no parity.toml line (tools are machine-side). Serialize with the other teleop balls — same files (src/tools.rs, android/).

---

What is left for a real device, exactly.

Three of the four ask the operator for nothing on any device: `device` reads
behind ACCESS_NETWORK_STATE, which the installer grants and no dialog ever
mentions; `clipboard_set` needs neither a permission nor focus (AOSP applies
the focus test only to the clipboard READ — see the DESIGN §16.1 paragraph);
`open` needs no grant at all.

The operator acts that remain are two, and both are ordinary:

1. **`notify` wants POST_NOTIFICATIONS on API 33+, and only a person can give
   it.** The emulator never asks, because `screens.sh` installs with `-g` and
   every runtime permission is granted outright — so the emulator cannot prove
   this path either way. On a real phone: the first `notify` while the app is
   in front raises Android's own dialog and refuses that invocation saying so;
   a second call after the grant posts. With the app backgrounded, or after a
   refusal, it names the settings act instead — Settings > Apps > yog >
   Notifications — because Android stops showing the dialog after one no and a
   sentence about a dialog that never appears would teach nothing.

2. **`open` needs yog to be the app in front.** Android has refused a
   background activity launch since API 29 and reports nothing when it does,
   so the tool asks before it acts and refuses in band rather than answering
   ok for a launch that never happened. Nothing an agent can do lifts this;
   the refusal names it, and points at `notify` as the way to ask the operator
   to bring the app forward.

No service enable, no appop, no restricted-settings trip: that is what makes
these the PAPER rung. bl-05b6 carries the on-device invocation proof none of
the corpus has yet.
