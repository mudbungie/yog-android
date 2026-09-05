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
