//! **What this workspace's agents actually changed** (yog §5.1 #32, VISION
//! §4.10; DESIGN §13.15): the pure git read `target..source` of every attempt
//! the workspace holds, and — when the ask named one changed file — that
//! file's bounded patch.
//!
//! **One diff row, two answers.** A science row's `diff` object IS a
//! work-diff row: upstream encodes both with one encoder *"so an attempt's
//! identity has one spelling anywhere"*, so this module is the one reader and
//! `codec::candidates` composes it rather than restating its fields (lernie
//! DESIGN §4.33, whose ruling transfers whole). Two readers of one shape drift
//! within a week (DESIGN §13.14).
//!
//! **The state token is READ here rather than carried**, which is the one
//! place this family differs from the tokens beside it (`codec::records`'
//! narrowing). A conversation's state and flight are words nothing branches
//! on; this one DECIDES which fields the row has — an unreadable project
//! states no refs at all, an absent one states no oids — so a decoder that
//! carried it whole would have to guess at every field under it. An unknown
//! state refuses naming it.
//!
//! **Binary is read off the SHAPE and not off a token.** Upstream writes
//! counts or it writes `binary`, never both, so there is nothing here to
//! match on and no third case to invent.
//!
//! **The patch is bytes** (lernie DESIGN §4.33's rule, taken the other way).
//! The desktop composes the bare listing and never asks for a patch at all;
//! this seat asks — a phone is where a review happens away from the desk — and
//! what it does with the answer is paint it. Nothing here parses a unified
//! diff into hunks: the engine bounded the bytes and the glass shows them.

use serde_json::{Map, Value};

use super::fields::{arr_of, bool_of, str_of, u64_of};
use super::files::Preview;

pub mod file;

pub use file::WorkFile;

/// One changed file's churn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Churn {
    pub path: String,
    pub added: u64,
    pub removed: u64,
    /// Bytes nothing counted lines in. The counts above are zero and mean
    /// nothing when this is set — which is why it is read off the shape.
    pub binary: bool,
}

/// **One attempt's diff**, in the engine's own spelling. Which of the fields
/// below are said is what `state` decides: `unreadable` says none of them,
/// `absent` says the two refs and what is missing, `diff` says the refs, both
/// oids, the churn and whether it was cut short.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diff {
    pub project: String,
    pub ball: String,
    /// **The discriminant of an attempt's identity**: the opaque handle of a
    /// candidate, or empty for the ball's own claim. Absence is a fact and
    /// never a zero (`codec::balls`' rule).
    pub handle: String,
    /// The acceptance mark, or empty for a candidate nothing has delivered.
    pub delivered: String,
    pub state: String,
    pub target: String,
    pub source: String,
    pub target_oid: String,
    pub source_oid: String,
    /// The refs the read could not find — `absent` alone.
    pub missing: Vec<String>,
    /// The churn — `diff` alone. Empty is a real answer there: two refs that
    /// hold the same tree.
    pub files: Vec<Churn>,
    pub truncated: bool,
}

/// One `work-diff` answer, as the engine spelled it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Churned {
    pub rows: Vec<Diff>,
    pub patch: Option<Preview>,
}

/// **What the work screen is holding**, and the workspace it was read for.
/// `work-diff` names a workspace, so a listing under another one is the wrong
/// claim — `Spread::about`'s law at a second site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Work {
    pub workspace: String,
    pub rows: Vec<Diff>,
    pub patch: Option<Preview>,
    /// **Which file this answer's patch was asked for**, `None` for the bare
    /// listing. The answer does not echo the address — a `patch` is a bounded
    /// file and nothing else — so it is named from the ask and the paint puts
    /// the bytes under exactly that row.
    pub opened: Option<WorkFile>,
}

impl Work {
    /// Whether this listing is about the workspace now focused.
    #[must_use]
    pub fn about(&self, workspace: &str) -> bool {
        self.workspace == workspace
    }
}

/// Read the `work-diff` answer.
pub(super) fn churned(o: &Map<String, Value>) -> Result<Churned, String> {
    Ok(Churned {
        rows: rows(o)?,
        patch: super::fields::opt_val(o, "patch", super::files::preview)?,
    })
}

/// The rows of a `work-diff` answer.
fn rows(o: &Map<String, Value>) -> Result<Vec<Diff>, String> {
    arr_of(o, "rows")?.iter().map(diff).collect()
}

/// **One diff row**, read wherever it arrives: on its own answer, or as a
/// science row's `diff` column.
pub(super) fn diff(v: &Value) -> Result<Diff, String> {
    let o = v
        .as_object()
        .ok_or("work-diff: a row is not an object")?
        .clone();
    let state = str_of(&o, "state")?;
    let mut row = Diff {
        project: str_of(&o, "project")?,
        ball: str_of(&o, "ball_id")?,
        handle: said(&o, "handle"),
        delivered: said(&o, "delivered"),
        state,
        target: String::new(),
        source: String::new(),
        target_oid: String::new(),
        source_oid: String::new(),
        missing: Vec::new(),
        files: Vec::new(),
        truncated: false,
    };
    match row.state.as_str() {
        "unreadable" => {}
        "absent" => {
            row.target = str_of(&o, "target")?;
            row.source = str_of(&o, "source")?;
            row.missing = super::fields::strings_of(&o, "missing", "work-diff")?;
        }
        "diff" => {
            row.target = str_of(&o, "target")?;
            row.source = str_of(&o, "source")?;
            row.target_oid = str_of(&o, "target_oid")?;
            row.source_oid = str_of(&o, "source_oid")?;
            row.files = arr_of(&o, "files")?
                .iter()
                .map(churn)
                .collect::<Result<Vec<Churn>, String>>()?;
            row.truncated = bool_of(&o, "truncated")?;
        }
        other => return Err(format!("work-diff: unknown state {other:?}")),
    }
    Ok(row)
}

/// One changed file. Binary is the presence of the flag, and the counts are
/// required exactly where it is absent.
fn churn(v: &Value) -> Result<Churn, String> {
    let o = v.as_object().ok_or("work-diff: a file is not an object")?;
    let path = str_of(o, "path")?;
    if o.contains_key("binary") {
        return Ok(Churn {
            path,
            added: 0,
            removed: 0,
            binary: bool_of(o, "binary")?,
        });
    }
    Ok(Churn {
        path,
        added: u64_of(o, "added")?,
        removed: u64_of(o, "removed")?,
        binary: false,
    })
}

/// A string the engine may not have written. Absence is a fact and never a
/// zero: an attempt with no handle is the claim, and it paints as nothing
/// rather than as a value this seat invented.
fn said(o: &Map<String, Value>, key: &str) -> String {
    o.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

#[cfg(test)]
mod tests;
