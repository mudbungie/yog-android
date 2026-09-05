//! **The decision queue's rows** (yog DESIGN §8.5, VISION §5 V5.2) — every
//! conversation the engine says is waiting on the operator, and why.
//!
//! **This seat reads it for one fact: the parked tool call** (DESIGN §13.7,
//! bl-b39d). A held invocation is the only thing on the wire that an operator
//! must *answer* rather than merely notice, and the three facts the engine
//! puts in `held` — which tool, which call, and the control's own sentence
//! about it — are what makes an answer an informed one. REMOTE §8.1 is
//! explicit that this sentence must cross unrewritten: rewriting it *"would
//! put a different call in front of the operator"*.
//!
//! **The whole row is read even though one field is painted.** The rest of the
//! queue — the signals, the flag, the preview, the ages — is the whole-queue
//! surface's (bl-35bd), and decoding it now is what makes that surface a paint
//! rather than a second decode. It is also the difference between a client
//! that skipped a field and one that misread it: this decoder refuses a row
//! whose shape it does not recognize instead of reading past it.
//!
//! **One recorded narrowing**, the `alignment` verdict's precedent exactly:
//! `signals` rides as its tokens rather than as an enum. The vocabulary is the
//! queue surface's to paint, and a strict table here would refuse a whole
//! answer — the held call in it included — the day upstream names a ninth
//! signal.

use serde_json::Value;

use super::conv::{AgentState, STATES};
use super::fields::{arr_of, bool_of, i64_of, opt, opt_val, pick, str_of, usize_of};

/// One conversation waiting on the operator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueRow {
    /// The §3.1 workspace name and the agent id — the same two keys, in the
    /// same vocabulary, that every gesture takes, so answering a row is
    /// copying two values rather than translating between two vocabularies.
    pub workspace: String,
    pub agent: String,
    pub display: String,
    pub state: AgentState,
    pub uncertain: bool,
    /// Why this row fires, in the engine's own tokens (the narrowing above).
    pub signals: Vec<String>,
    pub preview: String,
    pub age_secs: i64,
    pub pending: usize,
    /// **The parked invocation**, when this conversation holds one. Absent is
    /// *nothing is parked here* — never *parked with nothing to say*.
    pub held: Option<Held>,
    pub failure: Option<String>,
    pub flag: Option<Flag>,
}

/// The invocation the capability boundary parked before it ran (yog §8.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Held {
    /// The call's own id. Carried, never spent: the answer gesture names the
    /// conversation and the engine reads the mark itself at fire time, so
    /// nothing here can answer a call that is no longer the one held.
    pub tool_use: String,
    pub tool: String,
    /// The control's sentence about this call — what it is about to do, in
    /// the words the engine chose. Painted verbatim (REMOTE §8.1).
    pub reason: String,
}

/// A raised flag, when somebody asked for a look at this conversation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flag {
    pub at: String,
    pub reason: String,
}

/// Read one queue row, strictly.
pub(crate) fn row(v: &Value) -> Result<QueueRow, String> {
    let o = v.as_object().ok_or("attention row: not an object")?;
    Ok(QueueRow {
        workspace: str_of(o, "workspace")?,
        agent: str_of(o, "agent")?,
        display: str_of(o, "display")?,
        state: pick(o, "state", &STATES)?,
        uncertain: bool_of(o, "uncertain")?,
        signals: arr_of(o, "signals")?
            .iter()
            .map(|v| {
                v.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| "attention row: non-string signal".to_owned())
            })
            .collect::<Result<Vec<String>, String>>()?,
        preview: str_of(o, "preview")?,
        age_secs: i64_of(o, "age_secs")?,
        pending: usize_of(o, "pending")?,
        held: opt_val(o, "held", held)?,
        failure: opt(o, "failure", str_of)?,
        flag: opt_val(o, "flag", flag)?,
    })
}

fn held(v: &Value) -> Result<Held, String> {
    let o = v.as_object().ok_or("held: not an object")?;
    Ok(Held {
        tool_use: str_of(o, "tool_use")?,
        tool: str_of(o, "tool")?,
        reason: str_of(o, "reason")?,
    })
}

fn flag(v: &Value) -> Result<Flag, String> {
    let o = v.as_object().ok_or("flag: not an object")?;
    Ok(Flag {
        at: str_of(o, "at")?,
        reason: str_of(o, "reason")?,
    })
}

/// **The call parked at one conversation**, out of a whole queue.
///
/// `pub`, like `shell::place`'s pair and `RowAct::wants`, and for their
/// reason: this is a pure reading the ANDROID paint spends, so a `pub(crate)`
/// here would be dead code on a host build and the assertion would go with it.
/// The pairing is both keys, never the agent alone — an agent id is unique
/// inside a workspace and this queue spans all of them.
#[must_use]
pub fn held_at(rows: &[QueueRow], workspace: &str, agent: &str) -> Option<Held> {
    rows.iter()
        .find(|row| row.workspace == workspace && row.agent == agent)
        .and_then(|row| row.held.clone())
}

#[cfg(test)]
mod tests;
