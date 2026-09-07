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
//! **`framing` is the third, since PROTOCOL 18** (yog bl-ab53). It rode as a
//! bare string while the screen only ever printed it; the fourth word,
//! `in_flight`, is the one a surface has to BRANCH on — a step being written
//! right now against one an interrupt cut — and `crate::theme::framing` is
//! where that branch lives. A table here rather than a `match` on a `&str`
//! there, for this module's own reason: an unknown word must refuse by name at
//! the decode, not paint as the nearest colour at a paint site.
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

/// **The §4.4 terminal classification, in the engine's four words** (yog
/// `steps_view::wire::framing_token`). `InFlight` is PROTOCOL 18's addition:
/// the step being written right now, which `killed` used to have to cover.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Framing {
    Complete,
    Failed,
    Killed,
    InFlight,
}

impl Framing {
    /// The engine's own token, which is also what the census row is labelled
    /// with: the vocabulary rule (DESIGN §13.3) says a seat spells an op and a
    /// state in the words the engine does, and a second wording here would be
    /// a second vocabulary for one that already has an authority.
    ///
    /// Read out of [`FRAMINGS`] rather than matched a second time, so the
    /// table the wire is picked against is also the table the glass is
    /// labelled from — one home. `pub(crate)` for `Destination::file`'s reason
    /// exactly: it hands back a borrow (bootstrap rule 2's honest demotion).
    pub(crate) fn word(self) -> &'static str {
        FRAMINGS
            .iter()
            .find(|(_, framing)| *framing == self)
            .map_or("", |(word, _)| word)
    }
}

const FRAMINGS: [(&str, Framing); 4] = [
    ("complete", Framing::Complete),
    ("failed", Framing::Failed),
    ("killed", Framing::Killed),
    ("in_flight", Framing::InFlight),
];

/// One step of the conversation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepRow {
    pub seq: String,
    /// The §4.4 terminal classification, in the engine's four words.
    pub framing: Framing,
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

impl StepRow {
    /// **The census row as its two lines of words** — the seq, the framing,
    /// the wound and its reason, then the tokens, the attempts and the commit
    /// it recorded.
    ///
    /// It lives beside the decode and not in the paint file that spends it for
    /// `roster::spoke`'s reason (DESIGN §13.14): a derivation inside a paint
    /// file is one no host test can reach, and this one joins four fields and
    /// two optional ones. What the paint adds is the picked mark and the ink,
    /// which are facts about the screen and not about the step.
    #[must_use]
    pub fn line(&self) -> String {
        let wound = match &self.wound_reason {
            Some(why) => format!("{} — {why}", self.wound),
            None => self.wound.clone(),
        };
        let commit = if self.commit.is_empty() {
            String::new()
        } else {
            format!(" · {}", self.commit)
        };
        format!(
            "{} · {} · {wound}\n{} tokens · {} attempt(s){commit}",
            self.seq,
            self.framing.word(),
            self.tokens,
            self.attempts,
        )
    }
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

/// One census row. `wound` is the engine's token carried whole — nothing
/// branches on it, so a table for it would be a vocabulary with no reader —
/// while `framing` is picked, because `crate::theme::framing` branches on
/// exactly its four words.
fn row(v: &Value) -> Result<StepRow, String> {
    let o = object(v, "steps")?;
    Ok(StepRow {
        seq: str_of(&o, "seq")?,
        framing: pick(&o, "framing", &FRAMINGS)?,
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
