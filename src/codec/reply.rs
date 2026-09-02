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

use super::fields::{arr_of, bool_of, i64_of, opt, opt_val, str_of};
use super::pick::{self, ProviderRow};
use super::start::{self, Prepared};
use super::tools::{Capture, Invocation, capture_of, invocation_of};
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
    /// The receipt an advertisement earns. It carries nothing: what was
    /// presented is what was sent, and echoing it back would be a second
    /// spelling of a fact the sender holds.
    Advertised,
    /// The follow-class read's rows — this machine's work.
    Invocations(Vec<Invocation>),
    /// **A conversation that is now running** (§8.1) — what firing a staged
    /// conversation earns, carrying the name the engine gave it. This is the
    /// answer to `prompt` and the last frame of the two-gesture start, so a
    /// client that could not read it could stage a conversation and never
    /// learn that it started.
    Started { conversation: String },
    /// A staged conversation, as the engine stated it (§8.1). It is carried
    /// whole into the firing gesture; nothing here is re-derived.
    Prepared(Prepared),
    /// One workspace's provider rows, each carrying its own credential fact
    /// (bl-0267).
    Providers(Vec<ProviderRow>),
    /// One provider's model names, in the engine's order.
    Models(Vec<String>),
    /// The receipt a config write earns — what a model pick answers with. It
    /// carries nothing: the act stated the assignment whole, so an echo would
    /// be a second spelling of what the sender already holds.
    Applied,
    /// One invocation's standing after a call (REMOTE §5.3). `capture` is
    /// **absent** rather than empty while the far side still runs it, so a
    /// reader never has to tell "not finished" from "finished saying
    /// nothing".
    Routed {
        invocation: String,
        capture: Option<Capture>,
    },
}

impl Reply {
    /// This answer's `kind` token — the word the engine wrote. Named here
    /// rather than at each caller because two readers already need it (the
    /// seat model and the tool host both say "the engine answered X instead")
    /// and a second table of these words would drift from the decoder's own.
    pub fn kind(&self) -> String {
        match self {
            Self::Outcome { .. } => "outcome",
            Self::Workspaces { .. } => "workspaces",
            Self::Conversations(_) => "conversations",
            Self::Transcript(_) => "transcript",
            Self::Advertised => "advertised",
            Self::Invocations(_) => "invocations",
            Self::Routed { .. } => "routed",
            Self::Prepared(_) => "prepared",
            Self::Started { .. } => "started",
            Self::Providers(_) => "providers",
            Self::Models(_) => "models",
            Self::Applied => "applied",
        }
        .to_owned()
    }
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
        "advertised" => Reply::Advertised,
        "prepared" => Reply::Prepared(start::reply_of(o)?),
        "started" => Reply::Started {
            conversation: str_of(o, "conversation")?,
        },
        "invocations" => Reply::Invocations(rows(o, invocation_of)?),
        "providers" => Reply::Providers(rows(o, pick::row)?),
        "models" => Reply::Models(pick::names(o)?),
        "applied" => Reply::Applied,
        "routed" => Reply::Routed {
            invocation: str_of(o, "invocation")?,
            capture: opt_val(o, "capture", capture_of)?,
        },
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
