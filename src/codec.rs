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

use serde_json::{Value, json};

mod conv;
pub(crate) mod fields;
pub mod reply;
pub mod request;
pub mod start;
pub mod tools;
mod transcript;
mod ws;

pub use conv::{AgentState, ConvBall, ConvRow, Flight, Tone};
pub use request::decode;
pub use start::Prepared;
pub use tools::{Capture, Invocation, Tool};
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

/// Encode a gesture to its deposit envelope — the request frame's whole body.
/// Total over the slice; the spellings are the server codec's, byte for byte.
pub fn encode(gesture: &Gesture) -> Value {
    match gesture {
        Gesture::Act(Act::Message {
            workspace,
            agent,
            content,
        }) => json!({ "op": "message", "workspace": workspace,
                      "agent": agent, "content": content }),
        Gesture::Ask(Ask::Workspaces) => json!({ "op": "workspaces" }),
        Gesture::Ask(Ask::Conversations { workspace }) => {
            json!({ "op": "conversations", "workspace": workspace })
        }
        Gesture::Ask(Ask::Transcript { workspace, agent }) => {
            json!({ "op": "transcript", "workspace": workspace, "agent": agent })
        }
        Gesture::Ask(Ask::Invocations) => json!({ "op": "invocations" }),
        Gesture::Act(Act::Advertise { tools }) => {
            json!({ "op": "advertise", "tools": tools::encode_tools(tools) })
        }
        Gesture::Act(Act::Complete {
            invocation,
            capture,
        }) => json!({ "op": "complete", "invocation": invocation,
                      "capture": tools::capture_value(capture) }),
        Gesture::Act(Act::Prepare { workspace }) => start::encode_prepare(workspace),
        Gesture::Act(Act::Prompt { prepared, goal }) => start::encode_prompt(prepared, goal),
    }
}

#[cfg(test)]
mod tests;
