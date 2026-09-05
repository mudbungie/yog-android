//! **What the conversation's history is anchored to** — the operable spine
//! (`rail`) and the config commit governing it (`governing`).
//!
//! **Two lists rather than a nesting** (upstream's own shape): a card names
//! its notch by index, and a notch with no card is still a place a gesture
//! can reach. A notch that recorded no commit is the one that cannot be
//! reached — absent, never the empty string, so *unpinnable* and *pinned at
//! nothing* stay apart.
//!
//! **`governing`'s `oid` is the first field this seat reads whose MEANING
//! moved under an unchanged spelling** (REMOTE §9.12, lernie DESIGN §4.29).
//! It named the fork commit at PROTOCOL 4 and names the followed lineage's
//! HEAD at 5, and no mechanical check this repository owns can see the
//! difference — a signature ledger records field paths and types. So the trap
//! is restated here, at the decoder, where whoever paints the number is
//! reading.

use serde_json::{Map, Value};

use super::super::fields::{arr_of, i64_of, opt, str_of, u64_of, usize_of};
use super::agent::{object, words};

/// The spine: the notches, and the children hanging off them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rail {
    pub notches: Vec<Notch>,
    pub cards: Vec<Card>,
}

/// One notch of the spine: the step it is, the commit it pins to, and the
/// budget as of it — the rollup, not this step's own figure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notch {
    pub seq: String,
    pub budget: u64,
    /// The commit this notch pins to, clipped by the engine. Empty is a notch
    /// no gesture can reach, which is upstream's own test for one.
    pub short: String,
}

/// One child forked at a notch: who it is, where it came from, and what it is
/// doing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub agent: String,
    pub name: String,
    /// The engine's own sentence about where the child came from.
    pub fork: String,
    /// The §5.1 state token, in the conversation list's own words.
    pub state: String,
    pub tokens: u64,
    /// Which notch it was born at.
    pub notch: usize,
    /// The last of its inference text. Empty is a child that has produced
    /// none — a different statement from having said the empty string, and
    /// the difference is only ever painted as *nothing*.
    pub tail: String,
}

/// The config commit governing this conversation, and what it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Governing {
    /// **The followed lineage's head since PROTOCOL 5** — see the module doc.
    pub short_oid: String,
    /// The lineage being followed, or none — in which case `diverged` is the
    /// count of lineages that held it. The two are one enum's faces.
    pub follows: Option<String>,
    pub diverged: i64,
    pub files: Vec<String>,
}

/// Read the `rail` answer.
pub(in super::super) fn rail_of(o: &Map<String, Value>) -> Result<Rail, String> {
    Ok(Rail {
        notches: arr_of(o, "rows")?
            .iter()
            .map(notch)
            .collect::<Result<Vec<Notch>, String>>()?,
        cards: arr_of(o, "cards")?
            .iter()
            .map(card)
            .collect::<Result<Vec<Card>, String>>()?,
    })
}

/// One notch. The full commit rides beside the clipped one upstream and is
/// not read: the pinning gesture that would spend it is `fork`, which
/// `parity.toml` cites to bl-99fd, and a phone paints the short form.
fn notch(v: &Value) -> Result<Notch, String> {
    let o = object(v, "rail")?;
    Ok(Notch {
        seq: str_of(&o, "seq")?,
        budget: u64_of(&o, "budget")?,
        short: opt(&o, "short", str_of)?.unwrap_or_default(),
    })
}

/// One child card.
fn card(v: &Value) -> Result<Card, String> {
    let o = object(v, "cards")?;
    Ok(Card {
        agent: str_of(&o, "agent")?,
        name: str_of(&o, "name")?,
        fork: str_of(&o, "fork")?,
        state: str_of(&o, "state")?,
        tokens: u64_of(&o, "tokens")?,
        notch: usize_of(&o, "notch")?,
        tail: opt(&o, "tail", str_of)?.unwrap_or_default(),
    })
}

/// Read the `governing` answer.
pub(in super::super) fn governing_of(o: &Map<String, Value>) -> Result<Governing, String> {
    Ok(Governing {
        short_oid: str_of(o, "short_oid")?,
        follows: opt(o, "follows", str_of)?,
        diverged: i64_of(o, "diverged_lineages")?,
        files: words(o, "files")?,
    })
}
