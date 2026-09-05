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
use super::follow::{Stream, stream_of};
use super::hold::{self, Answered};
use super::pick::{self, ProviderRow, RoleRow};
use super::queue::{self, QueueRow};
use super::search::{self, Found};
use super::start::{self, Prepared};
use super::tools::{Capture, Invocation, capture_of, invocation_of};
use super::trail::{self, OpRow};
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
    /// The receipt an advertisement earns, carrying **whether the engine
    /// wrote** (REMOTE §5.1, PROTOCOL 8): `false` when it found the stored set
    /// identical to the one presented and compared, `true` when the document
    /// changed. The set itself is not echoed — what was presented is what was
    /// sent — and `wrote` is not an echo either: it is the one fact in this
    /// exchange the advertising box cannot compute for itself.
    ///
    /// **Required rather than absent-reads-false**, which is the engine's
    /// ruling and the right one: absent would read as *"nothing was restored"*
    /// — the reassuring answer — on exactly the build too old to tell. So a
    /// receipt in the pre-8 shape refuses here by name rather than decoding
    /// into a comfortable `false`.
    Advertised { wrote: bool },
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
    /// **What each role is set to** (bl-e9f9). An empty list is the answer
    /// for a workspace with nothing assigned — never a refusal.
    Roles(Vec<RoleRow>),
    /// **The answer in flight** (REMOTE §5.5): as much of it as has landed
    /// when the read was made. One shot per read, so this is the whole tail
    /// and not a delta to append — see `codec::follow`.
    Follow(Stream),
    /// The receipt a nudge earns (bl-d09e). It carries nothing: the act
    /// said everything, and what the nudge DID shows up in the next
    /// transcript read like any other work.
    Nudged,
    /// The receipt a config write earns — what a model pick answers with. It
    /// carries nothing: the act stated the assignment whole, so an echo would
    /// be a second spelling of what the sender already holds.
    Applied,
    /// **The receipt a flag earns** (DESIGN §13.5, bl-f97c). It carries
    /// nothing but its own `ok`: the act stated the reason, and what the flag
    /// DID is a row on the ops trail and a mark on the conversation's own
    /// row — which is the read this seat already makes every cadence, and so
    /// the read that settles a lost one.
    ///
    /// The other two row acts answer `outcome`, which this codec already
    /// reads; only the flag has a receipt of its own.
    Flagged,
    /// **The decision queue** (yog §8.5): every conversation waiting on the
    /// operator, each carrying the parked call this seat answers (§13.7).
    Attention(Vec<QueueRow>),
    /// **The ops trail's tail** (yog §4.2): what this engine last did, newest
    /// last, as many rows as the ask allowed.
    Ops(Vec<OpRow>),
    /// The receipt an acknowledgement earns. It carries nothing, and the read
    /// that says what it did is the trail itself.
    Acked,
    /// **The receipt `seen` earns** (yog §8.5): the queue that REMAINS after
    /// the acknowledged row was taken out of it — the engine's own words,
    /// *"the remainder alone reads as a plain `/attention`"*.
    ///
    /// **The rows are decoded and this seat adopts none of them**, which is
    /// deliberate rather than lazy. The queue here has exactly one writer —
    /// the held attention lane (DESIGN §14.1), which states the whole answer
    /// again the moment it changes — and a second writer would be a second
    /// authority for one fact, able to overwrite a newer frame with an older
    /// receipt. They are read anyway because reading a shape is not the same
    /// as spending it: a `rows` array this codec skipped would be a shape it
    /// could misread, which is exactly what REMOTE §3 forbids.
    Acknowledged(Vec<QueueRow>),
    /// The receipt a truncation earns — nothing again, for the same reason:
    /// the trail read after it is what says the trail is gone.
    TrailCleared,
    /// **What an answer landed on** (§8.6): the call, the verdict, and whether
    /// the branch was driven on after it.
    Answered(Answered),
    /// **The floor that stands over the conversation now** (§8.6) — re-derived
    /// by the engine after the write, never an echo of what was asked: a
    /// restore under a still-revoked ancestor leaves the conversation floored,
    /// and this is the field that says so.
    Floored { standing: bool },
    /// **What the needle found** (yog DESIGN §8.5). The answer carries its
    /// own question, so a search that matched nothing is told apart from no
    /// search at all — see `codec::search`.
    Search(Found),
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
            Self::Advertised { .. } => "advertised",
            Self::Invocations(_) => "invocations",
            Self::Routed { .. } => "routed",
            Self::Prepared(_) => "prepared",
            Self::Started { .. } => "started",
            Self::Providers(_) => "providers",
            Self::Models(_) => "models",
            Self::Roles(_) => "roles",
            Self::Applied => "applied",
            Self::Nudged => "nudged",
            Self::Flagged => "flagged",
            Self::Follow(_) => "follow",
            Self::Search(_) => "search",
            Self::Attention(_) => "attention",
            Self::Ops(_) => "ops",
            Self::Acked => "acked",
            Self::Acknowledged(_) => "acknowledged",
            Self::TrailCleared => "trail-cleared",
            Self::Answered(_) => "answered",
            Self::Floored { .. } => "floored",
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
        "advertised" => Reply::Advertised {
            wrote: bool_of(o, "wrote")?,
        },
        "prepared" => Reply::Prepared(start::reply_of(o)?),
        "started" => Reply::Started {
            conversation: str_of(o, "conversation")?,
        },
        "invocations" => Reply::Invocations(rows(o, invocation_of)?),
        "providers" => Reply::Providers(rows(o, pick::row)?),
        "models" => Reply::Models(pick::names(o)?),
        "roles" => Reply::Roles(rows(o, pick::role)?),
        "applied" => Reply::Applied,
        "nudged" => Reply::Nudged,
        "flagged" => Reply::Flagged,
        "follow" => Reply::Follow(stream_of(o)?),
        "search" => Reply::Search(search::found_of(o)?),
        "attention" => Reply::Attention(rows(o, queue::row)?),
        "ops" => Reply::Ops(rows(o, trail::row)?),
        "acked" => Reply::Acked,
        "acknowledged" => Reply::Acknowledged(rows(o, queue::row)?),
        "trail-cleared" => Reply::TrailCleared,
        "answered" => Reply::Answered(hold::answered_of(o)?),
        "floored" => Reply::Floored {
            standing: bool_of(o, "standing")?,
        },
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
