//! **The reads this seat spends** — the ask half of the boundary's grammar,
//! split from the act half (bl-146b) when the conversation's machinery took
//! `codec.rs` to the 300 band. The seam is the one `codec::request` already
//! reads the wire by: *"the grammar is asks and acts, and the reader is split
//! the same way — a table that reads a place and a table that names a
//! change."* What each is SPELLED as is `codec::encode`, one layer along.

/// The populating reads this seat spends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ask {
    /// The enumerated workspaces with their attention rollups.
    Workspaces,
    /// One workspace's conversation list, one row per subtree member.
    Conversations { workspace: String },
    /// One conversation's transcript, in message order.
    Transcript { workspace: String, agent: String },
    /// **The answer in flight** (REMOTE §5.5, bl-4822), held open (DESIGN
    /// §14.1): every read starts holding nothing, so the first frame is the
    /// whole tail so far and each later one is what landed since.
    Follow { workspace: String, agent: String },
    /// **What each role is set to** (bl-e9f9): the assignments the
    /// workspace's lineage tip holds, read from where the tuning gestures
    /// write. Per workspace, like the two reads beside it.
    Roles { workspace: String },
    /// One workspace's providers, with the credential fact each states about
    /// itself. Per workspace, because sign-ins are (bl-0267).
    Providers { workspace: String },
    /// One provider's models, in the engine's listing order.
    Models { workspace: String, provider: String },
    /// **Search the world this seat can see** (yog DESIGN §8.5): one needle,
    /// no scope. It is the only read here that names no place — every other
    /// one asks about a workspace or a conversation the operator is already
    /// looking at, and this one asks the engine where to look. The answer's
    /// addresses are the focuses this seat already takes, so a hit is fed
    /// straight back as one rather than resolved through anything.
    Search { text: String },
    /// **The decision queue** (yog §8.5): every conversation waiting on the
    /// operator. This seat reads it for the parked tool call each row may
    /// carry — the one thing on the wire that must be *answered* rather than
    /// noticed — and paints that where the answer is given (DESIGN §13.7).
    Attention,
    /// **Every ball this seat can see** (yog §8.5, DESIGN §13.9), with the
    /// workspace holding each. It names no workspace, which is the fact that
    /// puts it at the top depth beside the trail and the queue.
    Balls,
    /// **What one workspace holds** — the same table at the other width, and
    /// the only one of the three that names a place. Its rows are unpaintable
    /// under another focus, which is §14's pairing law and not a second rule.
    WorkspaceBalls { workspace: String },
    /// **The board**: the same balls folded into the engine's own columns,
    /// with a line per armed loop riding beside them.
    Board,
    /// **The ops trail's tail** (yog §4.2, REMOTE §9.17): the last `max`
    /// actions the engine took, newest last. `max` is the client's, not the
    /// engine's — a phone asks for a screenful — and it is carried rather
    /// than defaulted so the frame this codec writes is the frame the engine
    /// reads back.
    Ops { max: usize },
    /// **The conversation's own row** (DESIGN §13.11): what it is, what it is
    /// doing, and what it has spent. The header the records screen's other
    /// five halves are all about.
    Agent { workspace: String, agent: String },
    /// **Its step census**: one row per step, with the orphaned-tail state
    /// above them.
    Steps { workspace: String, agent: String },
    /// **One step's records**, addressed by the sequence the census stated.
    /// It is the one read of the six that is about a ROW rather than about
    /// the conversation, which is why nothing standing carries it: a standing
    /// read would have to invent a selection and then hold it (lernie DESIGN
    /// §4.32).
    Step {
        workspace: String,
        agent: String,
        seq: String,
    },
    /// **The operable spine**: the notches a gesture can reach, and the
    /// children forked at them.
    Rail { workspace: String, agent: String },
    /// **Which config commit governs it.** The engine also answers this
    /// question ABOUT a commit — an `at` this seat has no picker to name
    /// (`codec::request` refuses that frame by name) — so what is asked here
    /// is the standing one.
    Governing { workspace: String, agent: String },
    /// **The mail nothing has delivered yet**, one row per deposit.
    Inbox { workspace: String, agent: String },
    /// **The follow-class read**: this machine's next work, answered when
    /// there is some. The ask never inverts (REMOTE §3) — the engine speaks
    /// only into a stream this device asked for — so a tool host waits here
    /// rather than listening on a socket it would have to open.
    Invocations,
}
