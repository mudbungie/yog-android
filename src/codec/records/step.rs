//! **One step's drill-in** (`step`): the record files a step wrote, the events
//! its response carried, and every tool call inside it.
//!
//! **A record is its class and its bytes.** Upstream keeps three states
//! distinct on the wire — parsed, absent, and bytes that are not JSON —
//! because rendered bare, malformed content is indistinguishable from a file
//! whose content happens to be that text. So the class token rides, and the
//! bytes ride beside it; what does NOT ride is the parsed tree, which is the
//! same file read by a parser this seat has no use for.
//!
//! **The answer says which row it belongs to.** `seq` is echoed back by the
//! engine, so a drill-in that lands after the operator tapped another row
//! cannot paint under the wrong one — and nothing in the model holds a second
//! name for *which step is open* to drift from it (lernie DESIGN §4.32).

use serde_json::{Map, Value};

use super::super::fields::{arr_of, bool_of, opt, str_of};
use super::agent::object;

/// One step, whole.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    /// The step this answer is about — the engine's own echo of the ask.
    pub seq: String,
    pub meta: Record,
    pub request: Record,
    pub staging: Record,
    /// The response file's events, in order.
    pub response: Vec<Record>,
    pub tools: Vec<ToolRecord>,
    /// The captured logs that had bytes. Absent keys, never empty texts: a
    /// log with nothing in it is not a log that said the empty string.
    pub stderr: Option<Log>,
    pub driver: Option<Log>,
}

/// One captured log: its class, and the bytes where the class carries any.
///
/// **The class is read and the byte count is not.** Upstream writes `text`,
/// `truncated` or `binary`; the first two carry the bytes and the third
/// carries only a size, so a reader that took `text` as required would refuse
/// a perfectly ordinary answer about a binary log. The count beside a
/// truncation is a ledger this width has no room for — `codec::balls`' rule
/// about a figure nothing paints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Log {
    pub kind: String,
    pub text: String,
}

/// One record file, as its class and the bytes behind it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// `json`, `absent` or `unparsed` — the engine's own token, carried whole.
    pub kind: String,
    /// What the file says, where the class carries bytes at all.
    pub raw: String,
    /// The engine's sentence about an unparseable one.
    pub note: String,
}

/// One tool call's records (litany's own `is_error` reading beside them).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolRecord {
    pub tool_id: String,
    pub is_error: bool,
    pub input: Record,
    pub output: Record,
}

/// Read the `step` answer.
pub(in super::super) fn step_of(o: &Map<String, Value>) -> Result<Step, String> {
    Ok(Step {
        seq: str_of(o, "seq")?,
        meta: doc(o, "meta")?,
        request: doc(o, "request")?,
        staging: doc(o, "staging")?,
        response: arr_of(o, "response")?
            .iter()
            .map(|v| record(v, "response"))
            .collect::<Result<Vec<Record>, String>>()?,
        tools: arr_of(o, "tools")?
            .iter()
            .map(tool)
            .collect::<Result<Vec<ToolRecord>, String>>()?,
        stderr: preview(o, "stderr")?,
        driver: preview(o, "driver")?,
    })
}

/// One named record file of the step.
fn doc(o: &Map<String, Value>, key: &str) -> Result<Record, String> {
    record(o.get(key).ok_or_else(|| format!("step: no {key:?}"))?, key)
}

/// A record, wherever it hangs. `raw` and `note` are absent for the classes
/// that carry neither, and absent reads as nothing rather than as a value.
fn record(v: &Value, key: &str) -> Result<Record, String> {
    let o = object(v, key)?;
    Ok(Record {
        kind: str_of(&o, "kind")?,
        raw: opt(&o, "raw", str_of)?.unwrap_or_default(),
        note: opt(&o, "note", str_of)?.unwrap_or_default(),
    })
}

/// One tool call.
fn tool(v: &Value) -> Result<ToolRecord, String> {
    let o = object(v, "tools")?;
    Ok(ToolRecord {
        tool_id: str_of(&o, "tool_id")?,
        is_error: bool_of(&o, "is_error")?,
        input: doc(&o, "input")?,
        output: doc(&o, "output")?,
    })
}

/// A captured log, when the engine wrote the key at all.
fn preview(o: &Map<String, Value>, key: &str) -> Result<Option<Log>, String> {
    let Some(log) = o.get(key) else {
        return Ok(None);
    };
    let log = object(log, key)?;
    Ok(Some(Log {
        kind: str_of(&log, "kind")?,
        text: opt(&log, "text", str_of)?.unwrap_or_default(),
    }))
}
