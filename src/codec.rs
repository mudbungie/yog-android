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
pub mod follow;
pub mod pick;
pub mod reply;
pub mod request;
pub mod start;
pub mod tools;
mod transcript;
mod ws;

pub use conv::{AgentState, ConvBall, ConvRow, Flight, Tone};
pub use follow::Stream;
pub use pick::{Effort, ProviderRow, RoleRow};
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
    /// **The answer in flight** (REMOTE §5.5, bl-4822), read one shot at a
    /// time: every read starts holding nothing, so what comes back is the
    /// whole tail so far and this seat replaces rather than appends.
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
        Gesture::Ask(Ask::Follow { workspace, agent }) => {
            json!({ "op": "follow", "workspace": workspace, "agent": agent })
        }
        Gesture::Ask(Ask::Roles { workspace }) => {
            json!({ "op": "roles", "workspace": workspace })
        }
        Gesture::Ask(Ask::Providers { workspace }) => {
            json!({ "op": "providers", "workspace": workspace })
        }
        Gesture::Ask(Ask::Models {
            workspace,
            provider,
        }) => json!({ "op": "models", "workspace": workspace, "provider": provider }),
        Gesture::Act(Act::Stop {
            workspace,
            agent,
            children,
        }) => json!({ "op": "stop", "workspace": workspace,
                      "agent": agent, "children": children }),
        Gesture::Act(Act::Nudge { workspace, agent }) => {
            json!({ "op": "nudge", "workspace": workspace, "agent": agent })
        }
        Gesture::Act(Act::Effort {
            workspace,
            role,
            level,
        }) => pick::encode_effort(workspace, role, *level),
        Gesture::Act(Act::Priority {
            workspace,
            role,
            on,
        }) => pick::encode_priority(workspace, role, *on),
        Gesture::Act(Act::PickModel {
            workspace,
            role,
            provider,
            model,
        }) => pick::encode_pick(workspace, role, provider, model),
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
