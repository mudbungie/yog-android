//! **The n-candidate path** (yog VISION §4.10, DESIGN §13.12): spread one
//! obligation over n attempts, watch what each cost, accept one and release
//! the rest.
//!
//! **The row says which act it earns, and upstream's encoder is what decides.**
//! A science row's `diff` is a work-diff row, and a work-diff row carries a
//! `handle` or it does not: with one it is a **candidate** on
//! `attempt/<handle>` waiting to be accepted or released, without one it is
//! the ball's **own claim**, whose delivery obligation is the thing a fan
//! spreads. So the three acts are not a mode a screen holds; they are what the
//! row IS, and every value they take — the project, the ball, the handle — is
//! already on it (lernie DESIGN §4.36, whose ruling transfers whole).
//!
//! **One reader for one shape.** An attempt's `diff` is the same object the
//! `work-diff` answer spells, *"so an attempt's identity has one spelling
//! anywhere"* — so it is read by `codec::workdiff` and composed here, whole,
//! rather than by a second reader that would drift from it within a week
//! (DESIGN §13.14, bl-5a56). This screen paints three of its fields; the
//! churn and the refs are the work surface's, off the same value.
//!
//! **What is decoded is what these two screens paint.** Three columns ride
//! through unread and each is a decision: `usage`'s four counters (the ledger
//! `codec::balls` already declines to hold, and there is no total here to
//! carry instead), `pins` and `base`/`governing` (what an attempt was frozen
//! against, which is the config surface's question and unbuilt here), and
//! `conversation` (an address this screen offers no way to open — the day it
//! does is the day it earns a reader).

pub mod act;

use serde_json::{Map, Value};

use super::fields::{arr_of, str_of, u64_of};
use super::workdiff::{self, Diff};

/// **What the candidates screen is holding**, and the workspace it was read
/// for. `science` names a workspace, so a listing under another one is the
/// wrong claim — the same §14 pairing law `Records::about` keeps over a
/// conversation, one surface along.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spread {
    pub workspace: String,
    pub rows: Vec<Attempt>,
}

impl Spread {
    /// Whether this listing is about the workspace now focused.
    #[must_use]
    pub fn about(&self, workspace: &str) -> bool {
        self.workspace == workspace
    }
}

/// One attempt, as the engine measured it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attempt {
    /// **What this attempt changed**, whole, in the one spelling a diff row
    /// has anywhere (`codec::workdiff`). Its `handle` is the discriminant —
    /// the opaque handle of a candidate, or empty for the ball's own claim —
    /// and nothing else on the row says which of the three acts it earns.
    pub diff: Diff,
    /// What became of the attempt, and whatever that token can say: the
    /// acceptance's commit, the rejection's winner where there was one.
    pub outcome: String,
    pub commit: String,
    pub by: String,
    pub steps: u64,
    pub wall_secs: u64,
    /// The goal it was frozen with and the last thing it said — each absent
    /// upstream for an attempt that has neither, and absent paints as nothing.
    pub goal: String,
    pub response: String,
    pub verdicts: Vec<Judgement>,
}

/// One thing said about an attempt, by whoever said it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Judgement {
    pub sender: String,
    pub body: String,
}

/// What a delivery landed. **Two of the four identities are optional and each
/// absence is a fact** (lernie DESIGN §4.36): no `source` is a source ref that
/// was not there, no `commit` is a delivery that landed nothing. Painting
/// either as a blank would report a delivery that did not happen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delivered {
    pub base: String,
    pub target: String,
    pub source: String,
    pub commit: String,
}

/// Read the `science` answer's rows.
pub(super) fn science(o: &Map<String, Value>) -> Result<Vec<Attempt>, String> {
    arr_of(o, "rows")?.iter().map(row).collect()
}

/// One attempt row: the diff's identity, the outcome, and the figures beside
/// them.
fn row(v: &Value) -> Result<Attempt, String> {
    let o = object(v, "science")?;
    let diff = workdiff::diff(o.get("diff").ok_or("science: a row states no diff")?)?;
    let outcome = object(
        o.get("outcome").ok_or("science: a row states no outcome")?,
        "outcome",
    )?;
    Ok(Attempt {
        diff,
        outcome: str_of(&outcome, "state")?,
        commit: said(&outcome, "commit"),
        by: said(&outcome, "by"),
        steps: u64_of(&o, "steps")?,
        wall_secs: u64_of(&o, "wall_secs")?,
        goal: said(&o, "goal"),
        response: said(&o, "response"),
        verdicts: arr_of(&o, "verdicts")?
            .iter()
            .map(judgement)
            .collect::<Result<Vec<Judgement>, String>>()?,
    })
}

/// One verdict on an attempt.
fn judgement(v: &Value) -> Result<Judgement, String> {
    let o = object(v, "verdicts")?;
    Ok(Judgement {
        sender: str_of(&o, "sender")?,
        body: str_of(&o, "body")?,
    })
}

/// Read the `delivered` receipt.
pub(super) fn delivered(o: &Map<String, Value>) -> Result<Delivered, String> {
    Ok(Delivered {
        base: str_of(o, "base")?,
        target: str_of(o, "target")?,
        source: said(o, "source"),
        commit: said(o, "commit"),
    })
}

/// Read the `fanned` receipt: one prepared body per candidate, each already
/// rebound to its own attempt.
pub(super) fn fanned(o: &Map<String, Value>) -> Result<Vec<super::Prepared>, String> {
    arr_of(o, "rows")?
        .iter()
        .map(super::start::prepared_of)
        .collect()
}

/// A row, as an object, or the refusal naming the shape it was in.
fn object(v: &Value, kind: &str) -> Result<Map<String, Value>, String> {
    v.as_object()
        .cloned()
        .ok_or_else(|| format!("{kind}: row is not an object"))
}

/// A string the engine may not have written. Absence is a fact and never a
/// zero (`codec::balls`' rule): an attempt with no handle is the claim, and it
/// paints as nothing rather than as a value this seat invented.
fn said(o: &Map<String, Value>, key: &str) -> String {
    o.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

#[cfg(test)]
mod tests;
