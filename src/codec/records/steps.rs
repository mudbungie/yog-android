//! **The step census** (`steps`): one row per step of the conversation, in
//! sequence order, with the view-level orphaned-tail state above them.
//!
//! **Two class tokens and neither is a boolean.** Upstream carries `orphan`
//! and `wound` as discriminants with an optional reason beside each, because
//! the pair *(bool, Option<reason>)* stopped being a bijection the moment a
//! third arm arrived. They are read here as the same discriminants — a table,
//! never a derivation — so a token this build has not heard of refuses by
//! name rather than folding into the nearest one it knows.
//!
//! **The two timestamps are not read.** A census answers *what happened and
//! how it ended*; `started_at` and `ended_at` are a ledger whose only use
//! here would be a duration this seat would have to compute, and computing it
//! is what `codec::balls` already refuses to do with money.

use serde_json::{Map, Value};

use super::super::fields::{opt, pick, str_of, u64_of};
use super::agent::object;

/// The whole `steps` answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Steps {
    pub rows: Vec<StepRow>,
    /// Which tail the conversation left orphaned, and — when the engine had
    /// words for it — why.
    pub orphan: Orphan,
    pub orphan_reason: Option<String>,
}

/// The orphaned-tail classes. `None` is the ordinary conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orphan {
    None,
    Mail,
    ToolWindow,
}

const ORPHANS: [(&str, Orphan); 3] = [
    ("none", Orphan::None),
    ("mail", Orphan::Mail),
    ("tool_window", Orphan::ToolWindow),
];

/// One step of the conversation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepRow {
    pub seq: String,
    /// The §4.4 terminal classification, in the engine's three words.
    pub framing: String,
    /// The §7.3 wound's class, and the adapter's own last words when the
    /// no-response class left any.
    pub wound: String,
    pub wound_reason: Option<String>,
    /// How many times the step was attempted.
    pub attempts: u64,
    /// The read-state commit the step recorded, empty where it recorded none
    /// — which is exactly what makes a step unpinnable.
    pub commit: String,
    /// The four counters' own total, as the engine derived it.
    pub tokens: u64,
}

/// Read the `steps` answer.
pub(in super::super) fn steps_of(o: &Map<String, Value>) -> Result<Steps, String> {
    Ok(Steps {
        rows: super::super::fields::arr_of(o, "rows")?
            .iter()
            .map(row)
            .collect::<Result<Vec<StepRow>, String>>()?,
        orphan: pick(o, "orphan", &ORPHANS)?,
        orphan_reason: opt(o, "orphan_reason", str_of)?,
    })
}

/// One census row. `framing` and `wound` are the engine's tokens carried
/// whole: this screen paints the word, and a table here would be a second
/// vocabulary for one that already has an authority.
fn row(v: &Value) -> Result<StepRow, String> {
    let o = object(v, "steps")?;
    Ok(StepRow {
        seq: str_of(&o, "seq")?,
        framing: str_of(&o, "framing")?,
        wound: str_of(&o, "wound")?,
        wound_reason: opt(&o, "wound_reason", str_of)?,
        attempts: u64_of(&o, "attempts")?,
        commit: opt(&o, "commit", str_of)?.unwrap_or_default(),
        tokens: total(&o)?,
    })
}

/// The four counters' own total. The counters themselves ride through unread:
/// a phone paints one number against one ceiling, and four beside it would be
/// the ledger `codec::balls` already declines to hold.
fn total(o: &Map<String, Value>) -> Result<u64, String> {
    let tokens = o.get("tokens").ok_or("steps: a row states no tokens")?;
    u64_of(&object(tokens, "tokens")?, "total")
}
