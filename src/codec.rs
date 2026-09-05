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
//! two recorded narrowings: a conversation row's `alignment` verdict and a
//! ball chip's `state` token ride through untyped (`Value` / `String`) until
//! a surface here paints them.

pub mod balls;
mod conv;
pub mod encode;
pub(crate) mod fields;
pub mod follow;
pub mod hold;
pub mod pick;
pub mod queue;
pub mod reply;
pub mod request;
mod row;
pub mod search;
pub mod start;
pub mod tools;
pub mod trail;
mod transcript;
mod ws;

pub use balls::{BallRow, Board, BoardRow, Pane, View, WsBallRow};
pub use conv::{AgentState, ConvBall, ConvRow, Flight, Tone};
pub use encode::encode;
pub use follow::Stream;
pub use hold::{Answered, Verdict};
pub use pick::{Effort, ProviderRow, RoleRow};
pub use queue::{Held, QueueRow};
pub use request::decode;
pub use row::RowAct;
pub use search::{Address, Found, Hit, HitField};
pub use start::Prepared;
pub use tools::{Capture, Invocation, Tool};
pub use trail::{OpRow, Standing};
pub use transcript::{Block, Entry, EntryKind};
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
    /// **The follow-class read**: this machine's next work, answered when
    /// there is some. The ask never inverts (REMOTE §3) — the engine speaks
    /// only into a stream this device asked for — so a tool host waits here
    /// rather than listening on a socket it would have to open.
    Invocations,
}

/// A gesture: act or ask, the boundary's whole grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gesture {
    Act(Act),
    Ask(Ask),
}

#[cfg(test)]
mod tests;
