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
mod transcript;
mod ws;

pub use conv::{AgentState, ConvBall, ConvRow, Flight, Tone};
pub use transcript::{Block, Entry, EntryKind};
pub use ws::{ConfigTip, WsKind, WsRow};

/// The mutating half this seat spends. One variant today: the §8.2 deposit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Act {
    /// The plain send: `{"op":"message", workspace, agent, content}`.
    Message {
        workspace: String,
        agent: String,
        content: String,
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
    }
}

#[cfg(test)]
mod tests;
