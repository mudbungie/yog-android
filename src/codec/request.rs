//! **The gesture codec's decode side** — the inverse of [`encode`](super::encode),
//! and the mirror of the server's own `boundary::codec::decode`.
//!
//! This client never *reads* a request off a wire: it is always the asker
//! (REMOTE §3), so nothing in the app calls this at runtime. It exists because
//! REMOTE §3 says a client owes it to the conformance corpus:
//!
//! > *"decode every frame in both directories into its own types, and
//! > round-trip what it emits — decode then re-encode must return the frame
//! > exactly. A client that only sends requests still decodes the request
//! > fixtures; that is what catches a field it drops on the way out."*
//!
//! That is the whole argument for this module. An encoder alone can be proven
//! only against a fixture somebody wrote here; an encoder with an inverse can
//! be proven against a fixture the *server's own codec* wrote, and a field
//! this client silently omits shows up as a round trip that does not close.
//!
//! **It is exactly as narrow as the encoder, and refuses the rest by name.** A
//! shape outside this crate's slice is not decoded into an approximation of
//! itself — it refuses naming the op, because REMOTE §3's third rule is that
//! *"a shape a client does not implement is still one it must not misread."*
//! That reaches inside a shape as well as across shapes: `prepare` carries a
//! rung and `prompt` carries a name prediction, and this client spells one
//! rung and predicts no name (DESIGN §8), so a frame stating either of the
//! others is refused rather than flattened into the one this codec has.

use serde_json::{Map, Value};

use super::fields::str_of;
use super::{Ask, Gesture};

mod acts;

/// Read one request envelope into this crate's gesture type.
///
/// **Two tables, on the boundary's own seam.** The grammar is asks and acts
/// (`Gesture`), and the reader is split the same way: a table that reads a
/// place and a table that names a change. It is a seam rather than a shave —
/// nothing about `ops` belongs beside `advertise` — and the refusal stays in
/// exactly one arm, so an op in neither table is still named once.
pub fn decode(v: &Value) -> Result<Gesture, String> {
    let o = v.as_object().ok_or("request: not a JSON object")?;
    let op = str_of(o, "op")?;
    match ask(&op, o)? {
        Some(ask) => Ok(Gesture::Ask(ask)),
        None => acts::act(&op, o).map(Gesture::Act),
    }
}

/// The reads. `None` is *not one of mine* and never a refusal: [`act`] has the
/// other table and the one arm that names an op neither holds.
fn ask(op: &str, o: &Map<String, Value>) -> Result<Option<Ask>, String> {
    let ask = match op {
        "workspaces" => Ask::Workspaces,
        "conversations" => Ask::Conversations {
            workspace: str_of(o, "workspace")?,
        },
        "transcript" => Ask::Transcript {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
        },
        "invocations" => Ask::Invocations,
        "search" => Ask::Search {
            text: str_of(o, "text")?,
        },
        "attention" => Ask::Attention,
        "balls" => Ask::Balls,
        "workspace-balls" => Ask::WorkspaceBalls {
            workspace: str_of(o, "workspace")?,
        },
        "board" => Ask::Board,
        "science" => Ask::Science {
            workspace: str_of(o, "workspace")?,
        },
        "clients" => Ask::Clients {
            workspace: str_of(o, "workspace")?,
        },
        "lineages" => Ask::Lineages {
            workspace: str_of(o, "workspace")?,
        },
        // The records screen's six (DESIGN §13.11). Five read alike and one
        // names the row it is about; `governing` refuses its anchored form
        // below, which is why it is not in the aimed list.
        "agent" => Ask::Agent {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
        },
        "steps" => Ask::Steps {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
        },
        "rail" => Ask::Rail {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
        },
        "inbox" => Ask::Inbox {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
        },
        "step" => Ask::Step {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
            seq: str_of(o, "seq")?,
        },
        // **The work-review pair** (DESIGN §13.15). `files` refuses the
        // anchored frame for `governing`'s reason one noun along — `at` names
        // a tree and this seat has no pin — and reads the optional `path`,
        // which is the depth it does have.
        "files" => Ask::Files {
            workspace: unpinned(o.get("at"), str_of(o, "workspace")?)?,
            agent: str_of(o, "agent")?,
            path: super::fields::opt(o, "path", str_of)?,
        },
        "work-diff" => Ask::WorkDiff {
            workspace: str_of(o, "workspace")?,
            file: super::fields::opt_val(o, "file", super::workdiff::file::decode)?,
        },
        // **The admin reads** (DESIGN §13.17): each is its write's op token
        // with the written half absent, so the two tables split on exactly
        // that — a frame carrying the write's field is not a read.
        "config" if !o.contains_key("text") => Ask::Config {
            at: super::admin::destination(o.get("target"))?,
        },
        "marks" if !o.contains_key("branch") => Ask::Marks {
            workspace: str_of(o, "workspace")?,
        },
        "governing" => Ask::Governing {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
            at: super::fields::opt(o, "at", str_of)?,
        },
        "ops" => Ask::Ops {
            max: super::fields::usize_of(o, "max")?,
        },
        "follow" => Ask::Follow {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
        },
        "roles" => Ask::Roles {
            workspace: str_of(o, "workspace")?,
        },
        "providers" => Ask::Providers {
            workspace: str_of(o, "workspace")?,
        },
        "models" => Ask::Models {
            workspace: str_of(o, "workspace")?,
            provider: str_of(o, "provider")?,
        },
        "login-tail" => Ask::LoginTail {
            workspace: str_of(o, "workspace")?,
            provider: str_of(o, "provider")?,
        },
        _ => return Ok(None),
    };
    Ok(Some(ask))
}

/// **`files` asked AT a commit is a different tree**, and this codec has no
/// field to put the anchor in. `at` is VISION V1.2's pin — an assertion about
/// which commit is being read — and the controls that would make one are
/// `pin` and `unpin`, both `parity.toml` lines. So the anchored frame is
/// refused **by name** rather than answered off the live worktree, which is
/// the silent misread REMOTE §3's third rule forbids; the workspace rides
/// through so the caller reads one expression. `unanchored`'s shape, at the
/// other read that takes a commit.
fn unpinned(at: Option<&Value>, workspace: String) -> Result<String, String> {
    match at {
        None => Ok(workspace),
        Some(at) => Err(format!("files: unimplemented tree {at}")),
    }
}

#[cfg(test)]
mod tests;
