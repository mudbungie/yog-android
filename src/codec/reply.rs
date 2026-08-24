//! The reply codec's decode side — the mirror of the server's
//! `boundary/reply/decode.rs`, over this seat's slice.
//!
//! **The refusal is the envelope with no `kind`.** `ok` cannot be the
//! discriminant, because an `outcome` reply spells a captured run's own
//! verdict there — a gate that failed is `ok: false` and is not a refusal. So
//! a body carrying a `kind` is an answer, and a body carrying none must be
//! `{"ok": false, "error": …}`.
//!
//! The outer `Err` is a malformed envelope or body — bytes this codec cannot
//! read — and the inner `Err` is the refusal the envelope faithfully carried.

use serde_json::{Map, Value};

use super::fields::{arr_of, bool_of, i64_of, opt, str_of};
use super::{ConvRow, Entry, WsRow, conv, transcript, ws};

/// The typed answer this seat's gestures earn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reply {
    /// A captured run's receipt — what a `message` deposit answers with.
    /// `ok` is carried, not derived: it is the server's own verdict.
    Outcome {
        ok: bool,
        exit: i64,
        stdout: String,
        stderr: String,
    },
    /// The workspace enumeration, with the two staleness notes when the
    /// engine states them — absent is "fresh", never "declined to say".
    Workspaces {
        rows: Vec<WsRow>,
        stale: Option<String>,
        growth: Option<String>,
    },
    /// One workspace's conversation rows.
    Conversations(Vec<ConvRow>),
    /// One conversation's transcript rows, in message order.
    Transcript(Vec<Entry>),
}

/// Read one reply body off the wire.
pub fn decode(v: &Value) -> Result<Result<Reply, String>, String> {
    let o = v.as_object().ok_or("reply: not a JSON object")?;
    let Some(kind) = o.get("kind") else {
        return refusal_of(o).map(Err);
    };
    let kind = kind.as_str().ok_or("reply: non-string field \"kind\"")?;
    let reply = match kind {
        "outcome" => Reply::Outcome {
            ok: bool_of(o, "ok")?,
            exit: i64_of(o, "exit")?,
            stdout: str_of(o, "stdout")?,
            stderr: str_of(o, "stderr")?,
        },
        "workspaces" => Reply::Workspaces {
            rows: rows(o, ws::row)?,
            stale: opt(o, "stale", str_of)?,
            growth: opt(o, "growth", str_of)?,
        },
        "conversations" => Reply::Conversations(rows(o, conv::row)?),
        "transcript" => Reply::Transcript(rows(o, transcript::entry)?),
        other => return Err(format!("unknown reply kind {other:?}")),
    };
    Ok(Ok(reply))
}

/// The kind-less envelope: a refusal, and nothing else may wear that shape.
fn refusal_of(o: &Map<String, Value>) -> Result<String, String> {
    if bool_of(o, "ok")? {
        return Err("reply: ok with no kind — not a spelling either end writes".to_owned());
    }
    str_of(o, "error")
}

/// The `rows` array read by one row reader — the shape every listing shares.
fn rows<T>(
    o: &Map<String, Value>,
    read: fn(&Value) -> Result<T, String>,
) -> Result<Vec<T>, String> {
    arr_of(o, "rows")?.iter().map(read).collect()
}

#[cfg(test)]
mod tests;
