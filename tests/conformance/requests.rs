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
use super::expect::{ACT, ASKING_SIDE, BARE_RUNG, NO_SEED, NOT_THE_MINTER, READ};

pub const REQUESTS: &[(&str, Expect)] = &[
    ("ack", Refuses(ACT)),
    ("advertise", Reads),
    ("agent", Refuses(READ)),
    ("answer", Refuses(ACT)),
    ("arm", Refuses(ACT)),
    ("assign", Refuses(ACT)),
    ("attention", Refuses(READ)),
    ("balls", Refuses(READ)),
    ("board", Refuses(READ)),
    ("capture", Refuses(ASKING_SIDE)),
    ("clear-trail", Refuses(ACT)),
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
    ("enroll", Refuses(NOT_THE_MINTER)),
    ("fan", Refuses(ACT)),
    ("files", Refuses(READ)),
    ("flag", Refuses(ACT)),
    ("fleet", Refuses(ACT)),
    ("follow", Refuses(READ)),
    ("fork", Refuses(ACT)),
    ("governing", Refuses(READ)),
    ("help", Refuses(READ)),
    ("inbox", Refuses(READ)),
    ("interrupt", Refuses(ACT)),
    ("invocations", Reads),
    ("invoke", Refuses(ASKING_SIDE)),
    ("lineages", Refuses(READ)),
    ("marks", Refuses(ACT)),
    ("message", Reads),
    ("model", Refuses(ACT)),
    ("models", Refuses(READ)),
    ("nudge", Refuses(ACT)),
    ("ops", Refuses(READ)),
    (
        "prepare",
        Partial {
            reads: 1,
            reason: BARE_RUNG,
        },
    ),
    (
        "prompt",
        Partial {
            reads: 6,
            reason: NO_SEED,
        },
    ),
    ("providers", Refuses(READ)),
    ("rail", Refuses(READ)),
    ("release", Refuses(ACT)),
    ("restore", Refuses(ACT)),
    ("retarget", Refuses(ACT)),
    ("retire", Refuses(ACT)),
    ("revoke", Refuses(ACT)),
    ("scan", Refuses(ACT)),
    ("science", Refuses(READ)),
    ("search", Refuses(READ)),
    ("seen", Refuses(ACT)),
    ("step", Refuses(READ)),
    ("steps", Refuses(READ)),
    ("stop", Refuses(ACT)),
    ("transcript", Reads),
    ("update", Refuses(ACT)),
    ("work-diff", Refuses(READ)),
    ("workspace-balls", Refuses(READ)),
    ("workspaces", Reads),
];
