//! **What a gesture looks like on the wire** — the encoder, and the mirror of
//! `codec::request`'s decode. Split from `codec.rs` when the ball pane's reads
//! took that file to the 300 wall (bl-d587), on the seam the decode side had
//! already drawn: what a gesture IS is the vocabulary next door, and what it
//! is SPELLED as is here. The two are proved against each other by the
//! conformance corpus's round trip, which is the whole reason both exist.

use serde_json::{Value, json};

use super::{Act, Ask, Gesture, balls, hold, pick, row, start, tools};

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
        Gesture::Ask(Ask::Search { text }) => json!({ "op": "search", "text": text }),
        Gesture::Ask(Ask::Attention) => json!({ "op": "attention" }),
        Gesture::Ask(Ask::Ops { max }) => json!({ "op": "ops", "max": max }),
        Gesture::Ask(Ask::Balls) => json!({ "op": "balls" }),
        Gesture::Ask(Ask::WorkspaceBalls { workspace }) => {
            json!({ "op": "workspace-balls", "workspace": workspace })
        }
        Gesture::Ask(Ask::Board) => json!({ "op": "board" }),
        Gesture::Act(Act::Ack) => json!({ "op": "ack" }),
        Gesture::Act(Act::ClearTrail) => json!({ "op": "clear-trail" }),
        Gesture::Act(Act::Seen { workspace, agent }) => {
            json!({ "op": "seen", "workspace": workspace, "agent": agent })
        }
        Gesture::Act(Act::Answer {
            workspace,
            agent,
            verdict,
        }) => hold::encode(workspace, agent, *verdict),
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
        Gesture::Act(Act::Ball { project, name, act }) => balls::act::encode(project, name, act),
        Gesture::Act(Act::Row {
            workspace,
            agent,
            act,
        }) => row::encode(workspace, agent, act),
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
