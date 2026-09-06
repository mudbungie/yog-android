//! **Reading one reply body off the wire** — the decode half, split from the
//! vocabulary beside it (bl-146b) when the conversation's machinery took
//! `reply.rs` past the 300 wall. The seam is the one `codec.rs` and
//! `codec::encode` already draw one layer up: what an answer IS, and how it
//! is READ.

use serde_json::{Map, Value};

use super::super::fields::{arr_of, bool_of, i64_of, opt, opt_val, str_of};
use super::super::follow::stream_of;
use super::super::pick;
use super::super::queue;
use super::super::search;
use super::super::start;
use super::super::tools::{capture_of, invocation_of};
use super::super::trail;
use super::super::{
    admin, balls, candidates, clients, conv, files, hold, lineages, records, transcript, workdiff,
    ws,
};
use super::Reply;

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
        "agent" => Reply::Agent(records::agent_of(o)?),
        "steps" => Reply::Steps(records::steps_of(o)?),
        "step" => Reply::Step(records::step_of(o)?),
        "rail" => Reply::Rail(records::rail_of(o)?),
        "governing" => Reply::Governing(records::governing_of(o)?),
        "inbox" => Reply::Inbox(records::mail(o)?),
        "clients" => Reply::Clients(clients::rows(o)?),
        "lineages" => Reply::Lineages(lineages::rows(o)?),
        "science" => Reply::Science(candidates::science(o)?),
        "config" => Reply::Config(admin::config(o)?),
        "marks" => Reply::Marks(admin::marks(o)?),
        "deleted" => Reply::Deleted,
        "files" => Reply::Files(files::listing(o)?),
        "work-diff" => Reply::WorkDiff(workdiff::churned(o)?),
        "fanned" => Reply::Fanned(candidates::fanned(o)?),
        "delivered" => Reply::Delivered(candidates::delivered(o)?),
        "armed" => Reply::Armed {
            armed: bool_of(o, "armed")?,
        },
        "retired" => Reply::Retired {
            discarded: bool_of(o, "discarded")?,
        },
        "balls" => Reply::Balls(rows(o, balls::row)?),
        "workspace-balls" => Reply::WorkspaceBalls(rows(o, balls::bound)?),
        "board" => Reply::Board(balls::board(o)?),
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
