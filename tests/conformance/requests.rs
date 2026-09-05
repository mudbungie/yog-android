//! **Every request shape in the corpus, and this client's decision about it**
//! — the recorded half of REMOTE §3's third rule: *"a shape a client does not
//! implement is still one it must not misread, so skipping a fixture is a
//! decision recorded in the client, never a silent pass."*
//!
//! The table is exhaustive over `corpus/request/` in **both** directions
//! (`corpus.rs`): a row for a shape the corpus does not carry, and a shape the
//! corpus carries with no row, are each a red test. So a vocabulary that grows
//! upstream arrives here as a failure asking for a decision, rather than as
//! silence.
//!
//! `Reads` means every frame decodes **and re-encodes to the frame it came
//! from** — the round trip REMOTE §3 asks of what a client emits, and the only
//! thing that catches a field dropped on the way out. `Refuses` means no frame
//! decodes and each is refused naming the op. `Partial` is a shape this codec
//! spells in part: the count is what closes the round trip, and every other
//! frame must still be refused by name.

use super::expect::Expect::{self, Partial, Reads, Refuses};
use super::expect::{ACT, ASKING_SIDE, BARE_RUNG, NO_FORK_POINT, NO_SEED, NOT_THE_MINTER, READ};

pub const REQUESTS: &[(&str, Expect)] = &[
    ("ack", Reads),
    ("advertise", Reads),
    ("agent", Refuses(READ)),
    ("answer", Reads),
    ("arm", Refuses(ACT)),
    ("assign", Refuses(ACT)),
    ("attention", Reads),
    ("balls", Refuses(READ)),
    ("board", Refuses(READ)),
    ("capture", Refuses(ASKING_SIDE)),
    ("clear-trail", Reads),
    ("clients", Refuses(READ)),
    ("close", Refuses(ACT)),
    ("complete", Reads),
    ("config", Refuses(ACT)),
    ("conversations", Reads),
    ("create", Refuses(ACT)),
    ("delete-agent", Refuses(ACT)),
    ("delete-workspace", Refuses(ACT)),
    ("deliver", Refuses(ACT)),
    ("disarm", Refuses(ACT)),
    ("disband", Refuses(ACT)),
    ("effort", Reads),
    ("enroll", Refuses(NOT_THE_MINTER)),
    ("fan", Refuses(ACT)),
    ("files", Refuses(READ)),
    ("flag", Reads),
    ("fleet", Refuses(ACT)),
    ("follow", Reads),
    ("fork", Refuses(NO_FORK_POINT)),
    ("governing", Refuses(READ)),
    ("help", Refuses(READ)),
    ("inbox", Refuses(READ)),
    ("interrupt", Reads),
    ("invocations", Reads),
    ("invoke", Refuses(ASKING_SIDE)),
    ("lineages", Refuses(READ)),
    ("login", Refuses(ACT)),
    ("login-tail", Refuses(READ)),
    ("marks", Refuses(ACT)),
    ("message", Reads),
    ("model", Reads),
    ("models", Reads),
    ("nudge", Reads),
    ("ops", Reads),
    ("pin", Refuses(ACT)),
    (
        "prepare",
        Partial {
            reads: 1,
            reason: BARE_RUNG,
        },
    ),
    ("priority", Reads),
    (
        "prompt",
        Partial {
            reads: 6,
            reason: NO_SEED,
        },
    ),
    ("providers", Reads),
    ("rail", Refuses(READ)),
    ("release", Refuses(ACT)),
    ("restore", Reads),
    ("retarget", Reads),
    ("retire", Refuses(ACT)),
    ("roles", Reads),
    ("revoke", Reads),
    ("scan", Refuses(ACT)),
    ("science", Refuses(READ)),
    ("search", Reads),
    ("seen", Refuses(ACT)),
    ("step", Refuses(READ)),
    ("steps", Refuses(READ)),
    ("stop", Reads),
    ("transcript", Reads),
    ("unpin", Refuses(ACT)),
    ("update", Refuses(ACT)),
    ("work-diff", Refuses(READ)),
    ("workspace-balls", Refuses(READ)),
    ("workspaces", Reads),
];
