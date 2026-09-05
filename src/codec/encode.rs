//! **What a gesture looks like on the wire** — the encoder, and the mirror of
//! `codec::request`'s decode. Split from `codec.rs` when the ball pane's reads
//! took that file to the 300 wall (bl-d587), on the seam the decode side had
//! already drawn: what a gesture IS is the vocabulary next door, and what it
//! is SPELLED as is here. The two are proved against each other by the
//! conformance corpus's round trip, which is the whole reason both exist.

use serde_json::{Value, json};

use super::{Act, Ask, Gesture, balls, candidates, hold, pick, row, start, tools};

/// Encode a gesture to its deposit envelope — the request frame's whole body.
/// Total over the slice; the spellings are the server codec's, byte for byte.
/// Encode a gesture to its deposit envelope — the request frame's whole body.
/// Total over the slice; the spellings are the server codec's, byte for byte.
///
/// **Two tables, on the boundary's own seam** — `codec::request`'s decode side
/// is split the same way and says why: *"the grammar is asks and acts, and the
/// reader is split the same way — a table that reads a place and a table that
/// names a change."* One table, one direction, and the two never share an arm.
pub fn encode(gesture: &Gesture) -> Value {
    match gesture {
        Gesture::Ask(ask) => asked(ask),
        Gesture::Act(act) => acted(act),
    }
}

/// The reads.
fn asked(ask: &Ask) -> Value {
    match ask {
        Ask::Workspaces => json!({ "op": "workspaces" }),
        Ask::Conversations { workspace } => {
            json!({ "op": "conversations", "workspace": workspace })
        }
        Ask::Transcript { workspace, agent } => {
            json!({ "op": "transcript", "workspace": workspace, "agent": agent })
        }
        Ask::Invocations => json!({ "op": "invocations" }),
        Ask::Search { text } => json!({ "op": "search", "text": text }),
        Ask::Attention => json!({ "op": "attention" }),
        Ask::Ops { max } => json!({ "op": "ops", "max": max }),
        Ask::Balls => json!({ "op": "balls" }),
        Ask::WorkspaceBalls { workspace } => {
            json!({ "op": "workspace-balls", "workspace": workspace })
        }
        Ask::Board => json!({ "op": "board" }),
        Ask::Science { workspace } => {
            json!({ "op": "science", "workspace": workspace })
        }
        // **The records screen's six** (DESIGN §13.11). Five name a
        // conversation and nothing else; `step` names the row inside it.
        Ask::Agent { workspace, agent } => aimed("agent", workspace, agent),
        Ask::Steps { workspace, agent } => aimed("steps", workspace, agent),
        Ask::Rail { workspace, agent } => aimed("rail", workspace, agent),
        Ask::Governing { workspace, agent } => aimed("governing", workspace, agent),
        Ask::Inbox { workspace, agent } => aimed("inbox", workspace, agent),
        Ask::Step {
            workspace,
            agent,
            seq,
        } => json!({ "op": "step", "workspace": workspace,
                      "agent": agent, "seq": seq }),
        Ask::Follow { workspace, agent } => {
            json!({ "op": "follow", "workspace": workspace, "agent": agent })
        }
        Ask::Roles { workspace } => {
            json!({ "op": "roles", "workspace": workspace })
        }
        Ask::Providers { workspace } => {
            json!({ "op": "providers", "workspace": workspace })
        }
        Ask::Models {
            workspace,
            provider,
        } => json!({ "op": "models", "workspace": workspace, "provider": provider }),
    }
}

/// The writes.
fn acted(act: &Act) -> Value {
    match act {
        Act::Message {
            workspace,
            agent,
            content,
        } => json!({ "op": "message", "workspace": workspace,
                      "agent": agent, "content": content }),
        Act::Ack => json!({ "op": "ack" }),
        Act::ClearTrail => json!({ "op": "clear-trail" }),
        Act::Seen { workspace, agent } => {
            json!({ "op": "seen", "workspace": workspace, "agent": agent })
        }
        Act::Answer {
            workspace,
            agent,
            verdict,
        } => hold::encode(workspace, agent, *verdict),
        Act::Stop {
            workspace,
            agent,
            children,
        } => json!({ "op": "stop", "workspace": workspace,
                      "agent": agent, "children": children }),
        Act::Nudge { workspace, agent } => {
            json!({ "op": "nudge", "workspace": workspace, "agent": agent })
        }
        Act::Ball { project, name, act } => balls::act::encode(project, name, act),
        Act::Candidate { project, ball, act } => candidates::act::encode(project, ball, act),
        Act::Fan {
            project,
            ball,
            prepared,
            n,
        } => candidates::act::encode_fan(project, ball, prepared, *n),
        Act::Row {
            workspace,
            agent,
            act,
        } => row::encode(workspace, agent, act),
        Act::Effort {
            workspace,
            role,
            level,
        } => pick::encode_effort(workspace, role, *level),
        Act::Priority {
            workspace,
            role,
            on,
        } => pick::encode_priority(workspace, role, *on),
        Act::PickModel {
            workspace,
            role,
            provider,
            model,
        } => pick::encode_pick(workspace, role, provider, model),
        Act::Advertise { tools } => {
            json!({ "op": "advertise", "tools": tools::encode_tools(tools) })
        }
        Act::Complete {
            invocation,
            capture,
        } => json!({ "op": "complete", "invocation": invocation,
                      "capture": tools::capture_value(capture) }),
        Act::Prepare { workspace } => start::encode_prepare(workspace),
        Act::Prompt { prepared, goal } => start::encode_prompt(prepared, goal),
    }
}

/// **A read addressed at one conversation and carrying nothing else** — five
/// of the records screen's six (DESIGN §13.11), and one spelling rather than
/// five, because the op token is the only thing that differs between them.
fn aimed(op: &str, workspace: &str, agent: &str) -> Value {
    json!({ "op": op, "workspace": workspace, "agent": agent })
}
