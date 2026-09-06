//! The client's half of the boundary codec (yog REMOTE §3): encode of the
//! gestures this seat sends, strict decode of the replies it is told. The
//! parent spelling is the server's `src/boundary/codec.rs` and its
//! `reply::encode` — **where the two disagree, one of them is a bug**, and the
//! tests here pin the exact envelope bytes so a disagreement is a red test,
//! not a runtime surprise.
//!
//! **This is a slice, not the surface.** The server's vocabulary is large;
//! this codec spells exactly what the phone seat spends — the chat loop:
//! enumerate workspaces, list a workspace's conversations, read a transcript,
//! deposit a message — and grows per consumer, never speculatively. Decode is
//! strict the way the parent is strict (an unknown `kind`, a missing field, a
//! mistyped value, an unknown token each refuse naming the offender), with
//! three recorded narrowings: a conversation row's `alignment` verdict and a
//! ball chip's `state` token ride through untyped (`Value` / `String`) until
//! a surface here paints them, and the records screen carries the engine's
//! state and flight WORDS rather than picking them, because nothing there
//! branches on either (`codec::records`). A diff row's `state` is the
//! counter-example and says why the rule is not "carry every token": it
//! DECIDES which fields the row has, so it is read (`codec::workdiff`).

mod ask;
pub mod balls;
pub mod candidates;
pub mod clients;
mod conv;
pub mod encode;
pub(crate) mod fields;
pub mod files;
pub mod fleet;
pub mod follow;
pub mod fork;
pub mod hold;
pub mod lineages;
pub mod pick;
pub mod queue;
pub mod records;
pub mod reply;
pub mod request;
mod row;
pub mod search;
pub mod start;
pub mod tools;
pub mod trail;
mod transcript;
pub mod workdiff;
mod ws;

pub use ask::Ask;
pub use balls::act::BallAct;
pub use balls::{BallRow, Board, BoardRow, Pane, View, WsBallRow};
pub use candidates::act::CandidateAct;
pub use candidates::{Attempt, Delivered, Judgement, Spread};
pub use clients::{ClientRow, Machines};
pub use conv::{AgentState, ConvBall, ConvRow, Flight, Tone};
pub use encode::encode;
pub use files::{FileRow, Files, Listing, Preview};
pub use fleet::FleetAct;
pub use follow::Stream;
pub use hold::{Answered, Verdict};
pub use lineages::Lineage;
pub use pick::{Effort, ProviderRow, RoleRow};
pub use queue::{Held, QueueRow};
pub use records::{
    Agent, Card, Context, Governing, Log, Mail, Notch, Orphan, Rail, Record, Records, SeatRow,
    Step, StepRow, Steps, ToolRecord,
};
pub use request::decode;
pub use row::RowAct;
pub use search::{Address, Found, Hit, HitField};
pub use start::Prepared;
pub use tools::{Capture, Invocation, Tool};
pub use trail::{OpRow, Standing};
pub use transcript::{Block, Entry, EntryKind};
pub use workdiff::{Churn, Churned, Diff, Work, WorkFile};
pub use ws::{ConfigTip, WsKind, WsRow};

/// The mutating half this device spends: the §8.2 deposit, and the two acts
/// a tool host owns (REMOTE §5.1, §5.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Act {
    /// The plain send: `{"op":"message", workspace, agent, content}`.
    Message {
        workspace: String,
        agent: String,
        content: String,
    },
    /// What this machine can run, presented on connect. **It names no
    /// client**, and that is the gesture (REMOTE §5.1): the identity a set
    /// lands under is the intake's — the connection's certificate common name
    /// — and a `client` field would let any connection overwrite any other's.
    Advertise { tools: Vec<Tool> },
    /// One invocation answered with what running it captured. Only the client
    /// it was addressed to may post one, so this too names no client.
    Complete {
        invocation: String,
        capture: Capture,
    },
    /// **Stage a new conversation** (§8.1): everything it needs before it is
    /// prompted. Answers a prepared body, which [`Act::Prompt`] carries back.
    Prepare { workspace: String },
    /// **Fire a staged conversation** with the goal it is being given.
    Prompt { prepared: Prepared, goal: String },
    /// **Stop an in-flight turn** (REMOTE §3.1, bl-48fa). The gesture is the
    /// op, never a deposited `/stop` line: a slash line is CONTENT, and
    /// content wakes the very driver it meant to kill. `children` stops the
    /// subtree as well as the conversation named.
    Stop {
        workspace: String,
        agent: String,
        children: bool,
    },
    /// **Re-prompt a conversation from where it stands** (§8.2's nudge,
    /// bl-d09e): the act for a branch that stopped advancing. It is not a
    /// message — nothing is added to the transcript — it is a detached
    /// `litany advance`, so it says nothing and asks the driver to go on.
    Nudge { workspace: String, agent: String },
    /// **An act addressed to one conversation's ROW** (DESIGN §13.5,
    /// bl-f97c): interrupt, retarget or flag, whichever the row's long-press
    /// menu fired. The subject is stated once here and the choice is
    /// [`RowAct`]; `codec::row` is where the three spellings and the reason
    /// for the grouping live.
    Row {
        workspace: String,
        agent: String,
        act: RowAct,
    },
    /// **Answer the tool call parked at a conversation** (yog §8.6, DESIGN
    /// §13.7). The subject is the conversation and never the call: the engine
    /// reads the held mark itself at fire time, so this gesture cannot be
    /// spent on a call that is no longer the one held.
    Answer {
        workspace: String,
        agent: String,
        verdict: Verdict,
    },
    /// **Set a role's reasoning level** (REMOTE §9.4, bl-dfbb) — how much
    /// reasoning its model calls request. `None` is `off`: the absence of a
    /// level rather than a fourth level, which is what the engine reads.
    Effort {
        workspace: String,
        role: String,
        level: Option<Effort>,
    },
    /// **Ask a role's provider for its priority lane**, or stop asking. A
    /// checkbox and not a tri-state: `off` removes the line, because asking
    /// for the standard lane is a different intent no config key expresses.
    Priority {
        workspace: String,
        role: String,
        on: bool,
    },
    /// **Acknowledge the trail's alarms** (yog DESIGN §4.2, §7.3). It names
    /// no row and takes no argument: the ack is a watermark over the trail as
    /// it stands, so there is nothing here for a client to select — and
    /// nothing for it to select wrongly.
    Ack,
    /// **Answer the attention queue at one conversation** (yog §8.5, DESIGN
    /// §13.8): records what that conversation is currently asking about as
    /// seen, which is what takes its row off the queue.
    ///
    /// **It names its own subject rather than taking the focus**, and that is
    /// the one structural difference from every other conversation-shaped act
    /// here: the queue spans every workspace this seat can see, so a row's
    /// workspace is the row's and not the operator's current depth.
    Seen { workspace: String, agent: String },
    /// **Truncate the trail** (yog §4.2). The one act this seat sends that
    /// DISCARDS a durable record, which is why the control that fires it is
    /// armed (DESIGN §13.8) and why nothing about the arming is on the wire:
    /// an arm is a property of the glass, and the gesture is the same one a
    /// slash line spells.
    ClearTrail,
    /// **An act on one BALL** (DESIGN §13.9, bl-f36e): claim it, give it
    /// back, close it, file a new one beside it, or amend it. The address is
    /// stated once — the project the row named, and the `--as` stamp, which is
    /// the ball's bound WORKSPACE name and never an operator's — and the
    /// choice is [`BallAct`]; `codec::balls::act` is where the five spellings
    /// and the reason for the grouping live.
    Ball {
        project: String,
        name: String,
        act: BallAct,
    },
    /// **An act on one candidate, or on the claim they spread from** (DESIGN
    /// §13.12): fan the obligation over n attempts, accept one, release one.
    /// The address is stated once — the project and the ball the science row
    /// named — and the choice is [`CandidateAct`]; `codec::candidates::act` is
    /// where the three spellings and the reason for the grouping live.
    Candidate {
        project: String,
        ball: String,
        act: CandidateAct,
    },
    /// **Spread one obligation over n candidates** (DESIGN §13.12) — *"the
    /// start with n in the middle, not a second start path"* (lernie DESIGN
    /// §4.36), which is why it sits beside [`Act::Prepare`] and
    /// [`Act::Prompt`] rather than beside the two handle acts above. The
    /// prepared body is the engine's own and rides through whole; what comes
    /// back is one body per candidate, and each is fired by the ordinary
    /// firing gesture.
    Fan {
        project: String,
        ball: String,
        prepared: Prepared,
        n: usize,
    },
    /// **One of the two armings a workspace carries** (DESIGN §13.13): the
    /// drone loop, or the alignment monitor watching what it commits. The
    /// address is the workspace, stated once, and the choice is [`FleetAct`];
    /// `codec::fleet` is where the four spellings, the naming trap and the one
    /// shared receipt live.
    Fleet { workspace: String, act: FleetAct },
    /// **Start a child of this conversation from a point in its history**
    /// (DESIGN §13.16): the attempt. Its own variant rather than a sixth
    /// [`RowAct`] — the frame names `parent` rather than `agent`, carries a
    /// role, and its subject is a POINT in a history rather than the history
    /// — and `codec::fork` is where that argument and the two narrowings live.
    Fork {
        workspace: String,
        parent: String,
        /// The fork point: an operable notch's commit, or a `config/<name>`
        /// head. Empty is not a value — the engine's own `fork::Attempt` says
        /// a fork with no ref is a different gesture — so nothing composes one
        /// without a picked point.
        from: String,
        role: String,
        goal: String,
    },
    /// **Assign a role's model** (bl-0267): one workspace, one role, and the
    /// provider/model pair stated whole. The seat spends `worker`; the field
    /// carries whatever the frame said so another role round-trips rather
    /// than being flattened into this device's one.
    PickModel {
        workspace: String,
        role: String,
        provider: String,
        model: String,
    },
}

/// A gesture: act or ask, the boundary's whole grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gesture {
    Act(Act),
    Ask(Ask),
}

#[cfg(test)]
mod tests;
