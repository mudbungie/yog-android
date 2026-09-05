//! **The gesture codec's decode side** — the inverse of [`encode`](super::encode),
//! and the mirror of the server's own `boundary::codec::decode`.
//!
//! This client never *reads* a request off a wire: it is always the asker
//! (REMOTE §3), so nothing in the app calls this at runtime. It exists because
//! REMOTE §3 says a client owes it to the conformance corpus:
//!
//! > *"decode every frame in both directories into its own types, and
//! > round-trip what it emits — decode then re-encode must return the frame
//! > exactly. A client that only sends requests still decodes the request
//! > fixtures; that is what catches a field it drops on the way out."*
//!
//! That is the whole argument for this module. An encoder alone can be proven
//! only against a fixture somebody wrote here; an encoder with an inverse can
//! be proven against a fixture the *server's own codec* wrote, and a field
//! this client silently omits shows up as a round trip that does not close.
//!
//! **It is exactly as narrow as the encoder, and refuses the rest by name.** A
//! shape outside this crate's slice is not decoded into an approximation of
//! itself — it refuses naming the op, because REMOTE §3's third rule is that
//! *"a shape a client does not implement is still one it must not misread."*
//! That reaches inside a shape as well as across shapes: `prepare` carries a
//! rung and `prompt` carries a name prediction, and this client spells one
//! rung and predicts no name (DESIGN §8), so a frame stating either of the
//! others is refused rather than flattened into the one this codec has.

use serde_json::{Map, Value};

use super::fields::{arr_of, str_of};
use super::start::{Prepared, prepared_of};
use super::tools::{Tool, capture_of, tool_of};
use super::{Act, Ask, Gesture};

/// Read one request envelope into this crate's gesture type.
///
/// **Two tables, on the boundary's own seam.** The grammar is asks and acts
/// (`Gesture`), and the reader is split the same way: a table that reads a
/// place and a table that names a change. It is a seam rather than a shave —
/// nothing about `ops` belongs beside `advertise` — and the refusal stays in
/// exactly one arm, so an op in neither table is still named once.
pub fn decode(v: &Value) -> Result<Gesture, String> {
    let o = v.as_object().ok_or("request: not a JSON object")?;
    let op = str_of(o, "op")?;
    match ask(&op, o)? {
        Some(ask) => Ok(Gesture::Ask(ask)),
        None => act(&op, o).map(Gesture::Act),
    }
}

/// The reads. `None` is *not one of mine* and never a refusal: [`act`] has the
/// other table and the one arm that names an op neither holds.
fn ask(op: &str, o: &Map<String, Value>) -> Result<Option<Ask>, String> {
    let ask = match op {
        "workspaces" => Ask::Workspaces,
        "conversations" => Ask::Conversations {
            workspace: str_of(o, "workspace")?,
        },
        "transcript" => Ask::Transcript {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
        },
        "invocations" => Ask::Invocations,
        "search" => Ask::Search {
            text: str_of(o, "text")?,
        },
        "attention" => Ask::Attention,
        "balls" => Ask::Balls,
        "workspace-balls" => Ask::WorkspaceBalls {
            workspace: str_of(o, "workspace")?,
        },
        "board" => Ask::Board,
        "science" => Ask::Science {
            workspace: str_of(o, "workspace")?,
        },
        // The records screen's six (DESIGN §13.11). Five read alike and one
        // names the row it is about; `governing` refuses its anchored form
        // below, which is why it is not in the aimed list.
        "agent" => Ask::Agent {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
        },
        "steps" => Ask::Steps {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
        },
        "rail" => Ask::Rail {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
        },
        "inbox" => Ask::Inbox {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
        },
        "step" => Ask::Step {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
            seq: str_of(o, "seq")?,
        },
        "governing" => Ask::Governing {
            workspace: unanchored(o.get("at"), str_of(o, "workspace")?)?,
            agent: str_of(o, "agent")?,
        },
        "ops" => Ask::Ops {
            max: super::fields::usize_of(o, "max")?,
        },
        "follow" => Ask::Follow {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
        },
        "roles" => Ask::Roles {
            workspace: str_of(o, "workspace")?,
        },
        "providers" => Ask::Providers {
            workspace: str_of(o, "workspace")?,
        },
        "models" => Ask::Models {
            workspace: str_of(o, "workspace")?,
            provider: str_of(o, "provider")?,
        },
        _ => return Ok(None),
    };
    Ok(Some(ask))
}

/// The writes, and the one refusal every unknown op earns.
fn act(op: &str, o: &Map<String, Value>) -> Result<Act, String> {
    let act = match op {
        "message" => Act::Message {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
            content: str_of(o, "content")?,
        },
        "ack" => Act::Ack,
        "clear-trail" => Act::ClearTrail,
        "seen" => Act::Seen {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
        },
        "answer" => {
            let (workspace, agent, verdict) = super::hold::decode(o)?;
            Act::Answer {
                workspace,
                agent,
                verdict,
            }
        }
        "stop" => Act::Stop {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
            children: super::fields::bool_of(o, "children")?,
        },
        "nudge" => Act::Nudge {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
        },
        // The five the row menu fires (DESIGN §13.5, §13.7). One arm,
        // because one gesture: `codec::row` states the subject once and the
        // choice is its own enum. `fork` is deliberately not among them and
        // refuses below by name — the reason is in that file's header.
        "interrupt" | "retarget" | "flag" | "revoke" | "restore" => super::row::decode(op, o)?,
        // The five the ball pane fires (DESIGN §13.9). One arm for the same
        // reason: one family, one address, and the choice is `BallAct`.
        "assign" | "release" | "close" | "create" | "update" => super::balls::act::decode(op, o)?,
        // The three the candidates screen fires (DESIGN §13.12). One arm
        // again: one family, one obligation, and the choice is
        // `CandidateAct`. A frame naming no ball refuses inside — the reason
        // is in that file's header.
        "fan" | "deliver" | "retire" => super::candidates::act::decode(op, o)?,
        "effort" => Act::Effort {
            workspace: str_of(o, "workspace")?,
            role: str_of(o, "role")?,
            level: super::pick::level_of(o)?,
        },
        "priority" => Act::Priority {
            workspace: str_of(o, "workspace")?,
            role: str_of(o, "role")?,
            on: super::fields::bool_of(o, "on")?,
        },
        "model" => Act::PickModel {
            workspace: str_of(o, "workspace")?,
            role: str_of(o, "role")?,
            provider: str_of(o, "provider")?,
            model: str_of(o, "model")?,
        },
        "advertise" => Act::Advertise {
            tools: arr_of(o, "tools")?
                .iter()
                .map(tool_of)
                .collect::<Result<Vec<Tool>, String>>()?,
        },
        "complete" => Act::Complete {
            invocation: str_of(o, "invocation")?,
            capture: capture_of(
                o.get("capture")
                    .ok_or("complete: missing field \"capture\"")?,
            )?,
        },
        "prepare" => Act::Prepare {
            workspace: bare_rung(o.get("payload"), str_of(o, "workspace")?)?,
        },
        "prompt" => Act::Prompt {
            prepared: unseeded(o.get("seed"), prepared(o.get("prepared"))?)?,
            goal: str_of(o, "goal")?,
        },
        other => return Err(format!("unknown op {other:?}")),
    };
    Ok(act)
}

/// The staging payload, on the one rung this device spends. The workspace
/// rides through so the caller reads one expression rather than two bindings.
///
/// A `path` or `ball` rung is **refused by name**: this codec has no field to
/// put a work directory or a ball in, and answering the bare rung to a frame
/// that asked for either would be the silent misread §3's third rule forbids.
fn bare_rung(payload: Option<&Value>, workspace: String) -> Result<String, String> {
    let payload = payload.ok_or("prepare: missing field \"payload\"")?;
    let o = payload
        .as_object()
        .ok_or("prepare: payload is not an object")?;
    match str_of(o, "rung")?.as_str() {
        "bare" => Ok(workspace),
        rung => Err(format!("prepare: unimplemented rung {rung:?}")),
    }
}

/// **`governing` asked ABOUT a commit is a different question**, and this
/// codec has no field to put the anchor in. `at` names a fork point — a
/// commit of the conversation's own history — and the surface that picks one
/// is bl-99fd's, cited in `parity.toml` for `fork` in the same words. So the
/// anchored frame is refused **by name** rather than answered as the standing
/// read, which is the silent misread REMOTE §3's third rule forbids; the
/// workspace rides through so the caller reads one expression.
fn unanchored(at: Option<&Value>, workspace: String) -> Result<String, String> {
    match at {
        None => Ok(workspace),
        Some(at) => Err(format!("governing: unimplemented anchor {at}")),
    }
}

/// The prepared body out of a firing gesture — required, and read whole.
fn prepared(body: Option<&Value>) -> Result<Prepared, String> {
    prepared_of(body.ok_or("prompt: missing field \"prepared\"")?)
}

/// `seed` is the firing seat's own name prediction, and a phone predicts none
/// (DESIGN §8) — so this codec writes the null and reads only the null. A
/// stated seed is a field this client would drop on the way back out, which is
/// the exact class of miss the corpus exists to catch, so it refuses instead.
fn unseeded(seed: Option<&Value>, prepared: Prepared) -> Result<Prepared, String> {
    match seed {
        Some(Value::Null) => Ok(prepared),
        Some(v) => Err(format!("prompt: unimplemented seed {v}")),
        None => Err("prompt: missing field \"seed\"".to_owned()),
    }
}

#[cfg(test)]
mod tests;
