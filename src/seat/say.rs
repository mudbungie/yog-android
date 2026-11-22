//! **What the frame can SAY** — the act vocabulary, split from the worker's
//! commands (bl-2cad) when the acts were given a thread of their own. The
//! seam is the one this crate draws at five other sites and calls by the same
//! name: the boundary's grammar is asks and acts, `seat::cmd` is what the
//! frame asks the worker for, and this is what it tells the poster to post.
//!
//! **Not one of these is ever sent twice** (`seat::posted`), and none of them
//! waits behind a read: `seat::poster` is what spends every variant here, on a
//! connection of its own, the moment it arrives.

pub(super) enum Say {
    Deposit(String),
    Start(String),
    /// **Sign one provider in** (§13.19) — the act, fired at the focused
    /// workspace. Its answer is the run's standing rather than a receipt, so
    /// it seeds the tail as well as starting the run. With a code it hands
    /// back what the human copied instead, and the same tail is followed.
    Login(String, Option<String>),
    /// Assign the worker role's provider and model, stated whole.
    Pick(String, String),
    /// Set the worker's reasoning level, or remove it (`None` is off).
    Effort(Option<crate::codec::Effort>),
    /// Ask the worker's provider for its priority lane, or stop asking.
    Priority(bool),
    /// Acknowledge the trail's alarms.
    Ack,
    /// **Answer the attention queue at one conversation** (§13.8): the
    /// workspace and the agent the row named, both carried, because the queue
    /// spans workspaces and the focus is nobody's address here.
    Seen(String, String),
    /// **One act on one BALL** (§13.9): the project the row named, and which
    /// of the five was fired. One command for the group because they are one
    /// gesture — the pane has one home for the roster, and it is
    /// `codec::BallAct`. The `--as` stamp is not carried: it is the focused
    /// workspace and the worker is where the focus lives.
    Ball(String, crate::codec::BallAct),
    /// **Fork the focused conversation at a picked point** (§13.16), with the
    /// composer's text as the child's goal. Its own command for the same
    /// reason `codec::fork` is its own shape: the subject is the point.
    Fork {
        from: String,
        goal: String,
    },
    /// **Mint an offer for the next device** (§13.18): its own command rather
    /// than an arm of the admin act, because its answer IS the product.
    Offer(String, crate::leaf::Grade),
    /// **Approve or decline one pending offer** (§13.18), by its name.
    Settle(crate::codec::Settle, String),
    /// **One act of the admin surface** (§13.17): one command for the group
    /// because they are one surface, and the address is inside the choice
    /// because these five do not share one.
    Admin(crate::codec::AdminAct),
    /// **One handle act on one obligation** (§13.12): the project and the
    /// ball the science row named, and which of the two it fired.
    Candidate(String, String, crate::codec::CandidateAct),
    /// **The fan** — its own command, because it is a CHAIN rather than a
    /// gesture: stage, spread, then fire each candidate. What the glass knows
    /// is the count and the goal; the prepared body is the server's and never
    /// crosses this channel.
    Fan {
        project: String,
        ball: String,
        n: usize,
        goal: String,
    },
    /// **One of the two armings a workspace carries** (§13.13). One command
    /// for both families because they are one gesture with four spellings,
    /// and the roster has one home — `codec::FleetAct`.
    Fleet(crate::codec::FleetAct),
    /// Truncate the trail — the armed act (§13.8).
    ClearTrail,
    /// Stop the focused conversation's turn, optionally its subtree with it.
    StopTurn(bool),
    /// Re-prompt the focused conversation from where it stands.
    Nudge,
    /// **Answer the tool call parked at the focused conversation** (§13.7):
    /// release it, decline it, or keep it parked.
    Answer(crate::codec::Verdict, crate::codec::Scope),
    /// **One act on a NAMED conversation** (§13.5): the agent the row's menu
    /// was opened on, and which of the three it fired. One command for the
    /// group because they are one gesture — the roster has one home, and it
    /// is `codec::RowAct`.
    Row(String, crate::codec::RowAct),
}
