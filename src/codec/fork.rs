//! **The attempt** (yog VISION §5 V2, DESIGN §13.16): start a child of this
//! conversation from a point in its history, with a goal.
//!
//! **It is not a [`RowAct`](super::RowAct), and that was attacked before it
//! was built.** The other three acts of that group address the conversation —
//! same two facts, one parameter apiece — and this one does not: its frame
//! names `parent` rather than `agent`, it carries a `role` and a `skills`
//! list, and its SUBJECT is a point in a history rather than the history. A
//! variant folded into that enum would have been three fields nothing else in
//! it has and a subject that is not the group's, so the grouping's own
//! argument — *"they arrive together, differing only in the one parameter"* —
//! refuses it.
//!
//! **`from` is a ref and empty is not a value.** The engine's own
//! `fork::Attempt` says so: *"a fork with no ref is a different gesture."*
//! The refs this seat can name are the operable notches of the conversation's
//! spine and the workspace's `config/<name>` heads — both read by the records
//! screen (`rail`, `lineages`), which is why the picking surface is that
//! screen and this act hangs on its foot.
//!
//! **`skills` is empty, and a frame naming any refuses by name.** A skill set
//! is a choice off the same config a role is, and the read that lists one does
//! not exist on this wire at all — so a control offering names would be
//! inventing them. The empty list is the honest gesture, and the shape this
//! codec spells; `prompt`'s `seed` is the same narrowing one op along.

use serde_json::{Map, Value, json};

use super::Act;
use super::fields::str_of;

/// Encode one attempt. `skills` is written always and always empty — the
/// engine reads an absent field as the empty list, but a client that omitted
/// it could not read its own frame back, which is the round trip REMOTE §3
/// asks for.
pub(crate) fn encode(workspace: &str, parent: &str, from: &str, role: &str, goal: &str) -> Value {
    json!({ "op": "fork", "workspace": workspace, "parent": parent,
            "from": from, "role": role, "skills": [], "goal": goal })
}

/// Read one back.
pub(crate) fn decode(o: &Map<String, Value>) -> Result<Act, String> {
    unskilled(o.get("skills"))?;
    Ok(Act::Fork {
        workspace: str_of(o, "workspace")?,
        parent: str_of(o, "parent")?,
        from: str_of(o, "from")?,
        role: str_of(o, "role")?,
        goal: str_of(o, "goal")?,
    })
}

/// **A pinned skill is a field this codec has nowhere to put**, so a frame
/// stating one is refused rather than read as the attempt without it — the
/// silent misread REMOTE §3's third rule forbids. Required rather than
/// absent-reads-empty for the round trip's sake: this encoder writes the key,
/// so a decoder that accepted its absence would hand back a frame the sender
/// did not write.
fn unskilled(skills: Option<&Value>) -> Result<(), String> {
    match skills.map(Value::as_array) {
        Some(Some(pinned)) if pinned.is_empty() => Ok(()),
        Some(Some(pinned)) => Err(format!("fork: unimplemented skills {}", json!(pinned))),
        Some(None) => Err("fork: field \"skills\" is not an array".to_owned()),
        None => Err("fork: missing field \"skills\"".to_owned()),
    }
}

#[cfg(test)]
mod tests;
