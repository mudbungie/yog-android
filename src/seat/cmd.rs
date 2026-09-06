//! **What the frame can say to the worker** — the command vocabulary the
//! model's channel carries, split from the handle that sends it (bl-f36e)
//! when the ball pane's act took `model.rs` to the 300 wall. The seam is the
//! one `codec.rs` and `codec/encode.rs` already draw one layer down: what a
//! gesture IS, and the thing that spells it. `seat::worker` is what spends
//! every variant here; `seat::model` is what posts them.

pub(super) enum Cmd {
    Workspace(Option<String>),
    Conversation(String, String),
    Deposit(String),
    Start(String),
    /// **List the focused workspace's providers** (bl-0267) — a gesture of
    /// the selectors' own, asked when one is opened rather than on every
    /// pass: a pass is the standing set, and these are options.
    Providers,
    /// List one provider's models.
    Models(String),
    /// **Sign one provider in** (§13.19) — the act, fired at the focused
    /// workspace. Its answer is the run's standing rather than a receipt, so
    /// it seeds the tail as well as starting the run.
    Login(String),
    /// **Follow one provider's sign-in, or stop** (§13.19). The lane's want,
    /// and the empty case is the second: leaving the screen crosses no wire,
    /// so a tail can be closed with the engine unreachable — `Cmd::Search`'s
    /// own shape at the other gesture that turns something off.
    Watch(Option<String>),
    /// Assign the worker role's provider and model, stated whole.
    Pick(String, String),
    /// Set the worker's reasoning level, or remove it (`None` is off).
    Effort(Option<crate::codec::Effort>),
    /// Ask the worker's provider for its priority lane, or stop asking.
    Priority(bool),
    /// **Search the world, or drop the answer being shown** (bl-4c2b). The
    /// empty needle is the second: it crosses no wire, so a search can be
    /// left with the engine unreachable.
    Search(String),
    /// **Read the ops trail** (§13.8) — what the engine last did. A gesture
    /// read and not a standing one: nothing paints it unless the surface is
    /// open, and a phone's radio is not free.
    Ops,
    /// **A held lane's frame, or its end** (§14.1) — the attention queue's
    /// or the live tail's. A command like any gesture, because the worker
    /// adopts it where it adopts everything else and no lock is needed for
    /// a thread to hand it over.
    Lane(super::lane::Framed),
    /// Acknowledge the trail's alarms.
    Ack,
    /// **Read the ball pane at one of its three views** (§13.9). One command
    /// because the pane holds one answer: which view is open is the shell's,
    /// and which read answered is the pane's own.
    Balls(crate::codec::View),
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
    /// **Read the conversation's machinery** (§13.11) — the five reads the
    /// records screen opens with. One command because they are one value: six
    /// questions about one conversation, retired together when it moves.
    Records,
    /// **Drill into one step** of it, by the sequence the census stated. Its
    /// own command because it is the one read of the six that is about a ROW
    /// rather than about the conversation.
    Step(String),
    /// **Which config governs a picked fork point** (§13.16) — the anchored
    /// form of a read the records opening already makes. Its own command for
    /// `Step`'s reason: it is about one POINT rather than about the
    /// conversation, so nothing standing carries it.
    Anchor(String),
    /// **Fork the focused conversation at a picked point** (§13.16), with the
    /// composer's text as the child's goal. Its own command for the same
    /// reason `codec::fork` is its own shape: the subject is the point.
    Fork {
        from: String,
        goal: String,
    },
    /// **Mint the next device's material** (§13.18), and — its own command
    /// rather than an arm of the admin act — because its answer IS the
    /// product rather than a receipt.
    Enroll(String, crate::leaf::Grade),
    /// **Forget the material a mint answered with** (§13.18). The one command
    /// in this vocabulary whose whole product is that something is gone: the
    /// key is held in the worker's own memory and nowhere else, and closing
    /// the surface that shows it is what drops it.
    Forget,
    /// **Read one config file** (§13.17). The destination is the gesture's
    /// own — two of the three name no workspace — so nothing about the focus
    /// decides what is read.
    Config(crate::codec::Destination),
    /// **Read which task branch this workspace is marked with** (§13.17).
    Marks,
    /// **One act of the admin surface** (§13.17): one command for the group
    /// because they are one surface, and the address is inside the choice
    /// because these five do not share one.
    Admin(crate::codec::AdminAct),
    /// **Read this workspace's attempts** (§13.12) — what the candidates
    /// screen opens with.
    Science,
    /// **One handle act on one obligation** (§13.12): the project and the
    /// ball the science row named, and which of the two it fired.
    Candidate(String, String, crate::codec::CandidateAct),
    /// **The fan** — its own command, because it is a CHAIN rather than a
    /// gesture: stage, spread, then fire each candidate. What the glass knows
    /// is the count and the goal; the prepared body is the engine's and never
    /// crosses this channel.
    Fan {
        project: String,
        ball: String,
        n: usize,
        goal: String,
    },
    /// **Read the focused conversation's worktree** (§13.15) — the listing,
    /// or one file's bytes when a path is carried. One command for both
    /// because they are one question at two depths, and the deeper answer
    /// carries the shallower one whole.
    Files(Option<String>),
    /// **Read what this workspace's attempts changed** (§13.15) — the same
    /// pair one subject along: the listing, or one changed file's patch.
    Work(Option<crate::codec::WorkFile>),
    /// **Read which machines may execute for this workspace** (§13.14) —
    /// what the roster screen opens with, and what it re-asks while it is
    /// open, because presence is true only at the instant it was answered.
    Clients,
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
    Answer(crate::codec::Verdict),
    /// **One act on a NAMED conversation** (§13.5): the agent the row's menu
    /// was opened on, and which of the three it fired. One command for the
    /// group because they are one gesture — the roster has one home, and it
    /// is `codec::RowAct`.
    Row(String, crate::codec::RowAct),
    Stop,
}
