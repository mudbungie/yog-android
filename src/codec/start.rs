//! **The §8.1 start family**: staging a new conversation and firing it. The
//! mirror of the server's `boundary/codec/start.rs`, over the one rung this
//! device spends.
//!
//! Two gestures because starting is two acts: `prepare` does everything a new
//! conversation needs before it is prompted — the mint, the workspace if it
//! does not exist, the ball rung's steps — and answers a **prepared body**;
//! `prompt` carries that body back with the goal and fires it.
//!
//! **The prepared body rides through this client whole.** It is the engine's
//! own statement about a staged conversation, and every field is carried
//! rather than re-derived: a client that recomputed one would be inventing
//! world state it does not own, and the two would drift the first time the
//! engine's policy moved. `binding`, `lineage` and `role` are **real nulls** —
//! the field is present and its absence is the value — so a reply deposits
//! back as the gesture it came from.
//!
//! **`role` is the one field a SEAT is meant to write** (PROTOCOL 18, yog
//! bl-9ced): `prepare` answers `null` — litany's `worker`, spelled as an
//! absence exactly as `lineage`'s default is — and a seat that wants a
//! planning conversation deposits the same body back with `"role":
//! "planner"`. This client carries the null through faithfully and offers no
//! chooser: the honest list of roles a workspace declares is not a read on
//! this wire — `reply/roles` answers which roles are ASSIGNED a provider and
//! model, which is a different set from the ones the governing commit
//! declares — so a chooser built from it would silently omit exactly the role
//! somebody came here for. The follow-up is bl-045b; until it lands, plan mode
//! is started from a seat that has the list.
//!
//! **One rung, and the others are not omissions.** A phone is not where a
//! work directory is chosen or a ball is bound, so the bare rung is the whole
//! slice; the richer two grow here when a surface on this device needs them,
//! never speculatively.

use serde_json::{Map, Value, json};

use super::fields::{opt, str_of};

/// A staged conversation, exactly as the engine stated it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prepared {
    pub workspace: String,
    /// The §3.3 typed binding; `None` is the bare rung's "bind nothing".
    pub binding: Option<String>,
    /// The §8.7 birth policy; `None` is the default lineage.
    pub lineage: Option<String>,
    pub goal: String,
    /// The §7.3 origin token, carried as the word the engine wrote — this
    /// client never branches on it, and a token it cannot spell would
    /// otherwise refuse a staging it has no quarrel with.
    pub origin: String,
    /// The role the conversation is born on (PROTOCOL 18); `None` is
    /// litany's `worker`, spelled as the absence the engine writes.
    pub role: Option<String>,
}

/// The staging gesture, on the one rung this device spends.
pub(crate) fn encode_prepare(workspace: &str) -> Value {
    json!({ "op": "prepare", "workspace": workspace,
            "payload": { "rung": "bare" } })
}

/// The deferred fire. `seed` is the firing seat's own name prediction and a
/// phone predicts none, so it is null — stated rather than omitted, because
/// the field's absence is a value the server reads.
pub(crate) fn encode_prompt(prepared: &Prepared, goal: &str) -> Value {
    json!({ "op": "prompt", "prepared": body(prepared),
            "goal": goal, "seed": Value::Null })
}

/// The prepared body's one spelling, spent by the firing gesture. It is the
/// same shape the reply carries, because a body written twice would drift
/// from the body read.
pub(super) fn body(p: &Prepared) -> Value {
    json!({ "workspace": p.workspace, "binding": p.binding,
            "lineage": p.lineage, "goal": p.goal, "origin": p.origin,
            "role": p.role })
}

/// Read a prepared body back, strictly — every field required, the two
/// optional ones as the nulls they were written as.
pub(crate) fn prepared_of(v: &Value) -> Result<Prepared, String> {
    let o = v.as_object().ok_or("prepared: not an object")?;
    Ok(Prepared {
        workspace: str_of(o, "workspace")?,
        binding: opt(o, "binding", str_of)?,
        lineage: opt(o, "lineage", str_of)?,
        goal: str_of(o, "goal")?,
        origin: str_of(o, "origin")?,
        role: opt(o, "role", str_of)?,
    })
}

/// The body out of a `prepared` reply envelope.
pub(crate) fn reply_of(o: &Map<String, Value>) -> Result<Prepared, String> {
    let body = o
        .get("prepared")
        .ok_or("prepared: missing field \"prepared\"")?;
    prepared_of(body)
}

#[cfg(test)]
mod tests;
