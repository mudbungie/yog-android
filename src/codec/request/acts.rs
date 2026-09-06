//! **The writes**, and the one refusal every unknown op earns — the act half
//! of `codec::request`'s two tables, split out (bl-5a56) on the seam that file
//! has always read the wire by: *"a table that reads a place and a table that
//! names a change."* The same seam `codec::encode`, `codec::ask` and
//! `seat::asks`/`seat::acts` are drawn on.
//!
//! The two narrowings that reach INSIDE a shape live here with the table they
//! narrow: `prepare` carries a rung and `prompt` carries a name prediction,
//! and this client spells one rung and predicts no name (DESIGN §8), so a
//! frame stating either of the others is refused rather than flattened into
//! the one this codec has.

use serde_json::{Map, Value};

use super::super::Act;
use super::super::fields::{arr_of, str_of};
use super::super::start::{Prepared, prepared_of};
use super::super::tools::{Tool, capture_of, tool_of};

/// The writes, and the one refusal every unknown op earns.
pub(super) fn act(op: &str, o: &Map<String, Value>) -> Result<Act, String> {
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
            let (workspace, agent, verdict) = super::super::hold::decode(o)?;
            Act::Answer {
                workspace,
                agent,
                verdict,
            }
        }
        "stop" => Act::Stop {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
            children: super::super::fields::bool_of(o, "children")?,
        },
        "nudge" => Act::Nudge {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
        },
        // The five the row menu fires (DESIGN §13.5, §13.7). One arm,
        // because one gesture: `codec::row` states the subject once and the
        // choice is its own enum.
        "interrupt" | "retarget" | "flag" | "revoke" | "restore" => {
            super::super::row::decode(op, o)?
        }
        // **The fourth act of that roster, and not one of them** (DESIGN
        // §13.16): its subject is a POINT in the conversation's history
        // rather than the conversation, so it has a shape of its own — and
        // it refuses a pinned skill inside itself.
        "fork" => super::super::fork::decode(o)?,
        // The five the admin surface fires (DESIGN §13.17). One arm again:
        // one surface, one roster, and the choice is `AdminAct`. `config` and
        // `marks` reach here only in their WRITTEN form — the read half of
        // each op is an ask, and the ask table takes it first.
        // **The mint** (DESIGN §13.18): its own shape, because its subject is
        // a device that does not exist yet and its answer is material rather
        // than a receipt.
        "enroll" => super::super::enroll::decode(o)?,
        "config" | "marks" | "scan" | "delete-agent" | "delete-workspace" => {
            super::super::admin::act::decode(op, o)?
        }
        // The five the ball pane fires (DESIGN §13.9). One arm for the same
        // reason: one family, one address, and the choice is `BallAct`.
        "assign" | "release" | "close" | "create" | "update" => {
            super::super::balls::act::decode(op, o)?
        }
        // The three the candidates screen fires (DESIGN §13.12). One arm
        // again: one family, one obligation, and the choice is
        // `CandidateAct`. A frame naming no ball refuses inside — the reason
        // is in that file's header.
        "fan" | "deliver" | "retire" => super::super::candidates::act::decode(op, o)?,
        // The four the fleet screen fires (DESIGN §13.13). One arm for two
        // FAMILIES, which is the trap that file's header names: they share a
        // receipt, so the op is the only thing that says which setting an
        // answer belongs to.
        "fleet" | "disband" | "arm" | "disarm" => super::super::fleet::decode(op, o)?,
        "effort" => Act::Effort {
            workspace: str_of(o, "workspace")?,
            role: str_of(o, "role")?,
            level: super::super::pick::level_of(o)?,
        },
        "priority" => Act::Priority {
            workspace: str_of(o, "workspace")?,
            role: str_of(o, "role")?,
            on: super::super::fields::bool_of(o, "on")?,
        },
        "login" => Act::Login {
            workspace: str_of(o, "workspace")?,
            provider: str_of(o, "provider")?,
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
