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
use super::expect::{
    ACT, ALREADY_HELD, ALWAYS_A_BALL, ASKING_SIDE, BARE_RUNG, NO_ANCHOR, NO_FORK_POINT,
    NO_SCHEDULING, NO_SEED, NO_TREE, NOT_THE_MINTER, READ,
};

pub const REQUESTS: &[(&str, Expect)] = &[
    ("ack", Reads),
    ("advertise", Reads),
    ("agent", Reads),
    ("answer", Reads),
    ("arm", Reads),
    ("assign", Reads),
    ("attention", Reads),
    ("balls", Reads),
    ("board", Reads),
    ("capture", Refuses(ASKING_SIDE)),
    ("clear-trail", Reads),
    ("clients", Reads),
    ("close", Reads),
    ("complete", Reads),
    ("config", Refuses(ACT)),
    ("conversations", Reads),
    (
        "create",
        Partial {
            reads: 2,
            reason: NO_SCHEDULING,
        },
    ),
    ("delete-agent", Refuses(ACT)),
    ("delete-workspace", Refuses(ACT)),
    (
        "deliver",
        Partial {
            reads: 1,
            reason: ALWAYS_A_BALL,
        },
    ),
    ("disarm", Reads),
    ("disband", Reads),
    ("effort", Reads),
    ("enroll", Refuses(NOT_THE_MINTER)),
    (
        "fan",
        Partial {
            reads: 3,
            reason: ALWAYS_A_BALL,
        },
    ),
    (
        "files",
        Partial {
            reads: 2,
            reason: NO_TREE,
        },
    ),
    ("flag", Reads),
    ("fleet", Reads),
    ("follow", Reads),
    ("fork", Refuses(NO_FORK_POINT)),
    (
        "governing",
        Partial {
            reads: 1,
            reason: NO_ANCHOR,
        },
    ),
    ("help", Refuses(ALREADY_HELD)),
    ("inbox", Reads),
    ("interrupt", Reads),
    ("invocations", Reads),
    ("invoke", Refuses(ASKING_SIDE)),
    ("lineages", Reads),
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
    ("rail", Reads),
    ("release", Reads),
    ("restore", Reads),
    ("retarget", Reads),
    (
        "retire",
        Partial {
            reads: 1,
            reason: ALWAYS_A_BALL,
        },
    ),
    ("roles", Reads),
    ("revoke", Reads),
    ("scan", Refuses(ACT)),
    ("science", Reads),
    ("search", Reads),
    ("seen", Reads),
    ("step", Reads),
    ("steps", Reads),
    ("stop", Reads),
    ("transcript", Reads),
    ("unpin", Refuses(ACT)),
    (
        "update",
        Partial {
            reads: 2,
            reason: NO_SCHEDULING,
        },
    ),
    ("work-diff", Reads),
    ("workspace-balls", Reads),
    ("workspaces", Reads),
];
