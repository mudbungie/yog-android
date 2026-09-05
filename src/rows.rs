//! One-line transcript rows — the phone's projection of a decoded transcript,
//! mirroring yog DESIGN §11's density rule spelling for spelling.
//!
//! Vertical space is the scarce resource and a phone has the least of it:
//! **every** transcript line — a delivered message, one model text block, one
//! thinking block, one tool call, one tool result, the live tail — is exactly
//! ONE row that folds open onto its full payload. A row is therefore a
//! *block*, not an entry: a model message that says something and then calls
//! two tools is three rows, because it is three things.
//!
//! **Expansion is derived, never stored per row.** The auto-state is a pure
//! function of the row's class and the two durable knobs ([`AutoExpand`]); the
//! caller's fold set holds *explicit overrides only*, so `expanded = auto XOR
//! overridden`. That dissolves "state on arrival" as a special case — there is
//! no arrival event, and a row appearing mid-frame is already in its
//! auto-state without anyone having to notice it appeared. On a seat that
//! re-asks the whole transcript at cadence and keeps no durable state, that is
//! not a nicety: nothing else would survive the next answer.
//!
//! Keys are the row's identity — `tx/<entry name>#<block index>` — so they
//! survive that stateless re-read.
//!
//! **Pure**: no egui, no JNI, no clock, no I/O. This module hands the shell a
//! vocabulary and a `Vec<Row>` and knows nothing of how either is painted,
//! which is what lets the whole projection be host-tested under the 100%
//! floor while the paint layer stays android-only.
//!
//! Cut at the parent's own seams: **what a row is** (the vocabulary below),
//! **what an entry becomes** ([`project`] — the exhaustive per-variant match
//! and the preview/body split), and **what a finished turn becomes**
//! ([`turns`] — the rollup of a turn's machinery to one aggregate line).

use std::collections::BTreeSet;

use crate::codec::Entry;

mod build;
mod compacted;
mod project;
mod turns;
mod wounded;

/// The six-value §11 tone vocabulary, re-exported rather than restated: the
/// wire already spells it for a conversation row ([`crate::codec::Tone`]) and
/// two identical enums in one crate drift within a week. A locally derived
/// hue and a decoded one mean the same thing to the shell, so they are the
/// same type.
pub use crate::codec::Tone;

/// The two auto-state knobs: whether a class expands on its own. The defaults
/// are the parent's operator ruling — the conversation open, all else folded —
/// and both are knobs so the policy stays config rather than code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoExpand {
    /// The conversation itself: delivered messages, model text, the live tail.
    pub responses: bool,
    /// Everything else: thinking, tool calls/results, raw bytes, rollups.
    pub others: bool,
}

impl Default for AutoExpand {
    fn default() -> Self {
        Self {
            responses: true,
            others: false,
        }
    }
}

/// Which auto-knob a row answers to. The split is **conversation vs
/// machinery**, not model vs everyone: a message delivered *to* the agent is
/// the other half of the exchange the operator came to read, so it arrives
/// expanded beside the reply it provoked — a user turn folded shut is the
/// operator's own words hidden from them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowClass {
    /// Someone talking: a delivered message, a model text block, the live tail.
    Response,
    /// Machinery: thinking, tool calls, tool results, raw bytes, rollups.
    Other,
}

/// What a row's fold toggle opens onto — the two things a fold can reveal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fold {
    /// The row's own `body`, printed beneath it. A row with an empty body has
    /// nothing to reveal and shows no toggle at all.
    Payload,
    /// A finished turn's step rows, which follow this row while it is expanded
    /// and are absent from the projection while it is not — each of them
    /// folding on its own, all the way down.
    Steps,
}

/// Who a row speaks for — the closed §11 role vocabulary, derived from
/// committed bytes only (the sender token and the `epitaph` field). Machinery
/// rows have no role: nobody is speaking.
///
/// It lives here and not beside a palette because this crate's palette is
/// android-only and outside the host suite. The *derivation* is the half worth
/// testing, so it sits with the projection that performs it; the shell maps a
/// role to a hue exactly as it maps a [`Tone`], and neither mapping is this
/// module's business.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// The operator's own words: the reserved `user` sender.
    User,
    /// The agent speaking: model output, or the live streaming tail.
    Model,
    /// Any other sender — a peer agent's message into this inbox.
    Peer,
    /// A result deposit (`epitaph:` present): a dispatched child's ending,
    /// arriving as mail rather than being chosen speech.
    Ended,
}

/// One transcript line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// Stable identity across the stateless re-read: `tx/<entry name>#<n>`.
    pub key: String,
    /// The always-visible label — `sender:`, `⚙ Read`, `✔ tool result — ok`.
    pub prefix: String,
    /// The payload's first line, shown while contracted.
    pub preview: String,
    /// The full payload, shown while expanded and **empty when the payload
    /// already fits the one line** — such a row has nothing to fold and shows
    /// no toggle, so the empty body *is* the fact and no second flag exists.
    pub body: String,
    /// What the prefix stands for; empty when the label says everything.
    pub hover: String,
    /// Which auto-knob this row answers to.
    pub class: RowClass,
    /// The hue the row asks for — never an RGB, which is the shell's to know.
    pub tone: Tone,
    /// Who is speaking, `None` on machinery.
    pub role: Option<Role>,
    /// What the toggle reveals.
    pub fold: Fold,
    /// The derived answer, recomputed every projection — never stored.
    pub expanded: bool,
}

/// Project decoded transcript `entries` into one-line rows. `speaker` is the
/// conversation's display name — **who** the model turns are, since a speaker
/// is an agent and not a model id; `auto` is the durable knob pair; `folds` is
/// the caller's override set, where membership *flips* a row's auto-state, so
/// an empty set means "everything as configured".
///
/// A [`BTreeSet`] and not a hash set: the projection is re-run every frame and
/// its output is asserted line for line by the suite, so a deterministic
/// iteration order costs nothing and buys a test that cannot flake.
pub fn rows(
    entries: &[Entry],
    speaker: &str,
    auto: AutoExpand,
    folds: &BTreeSet<String>,
) -> Vec<Row> {
    let mut flat = Vec::new();
    let mut steps = Vec::new();
    let mut usage = Vec::new();
    for entry in entries {
        let before = flat.len();
        project::push_entry(entries, entry, speaker, &mut flat);
        for block in 0..flat.len().saturating_sub(before) {
            steps.push(turns::step_of(&entry.kind, block));
            usage.push(turns::usage_of(&entry.kind));
        }
    }
    let mut out = turns::group(&flat, &steps, &usage, auto, folds);
    for row in &mut out {
        row.expanded = expanded_for(row, auto, folds.contains(&row.key));
    }
    out
}

/// The auto-state, flipped by an explicit override — the whole expansion rule.
/// A row **in flight** auto-expands whatever its class knob says: while a step
/// is happening it is the show, and completion returns it to its class
/// auto-state with no event to notice and nothing to store.
fn expanded_for(row: &Row, auto: AutoExpand, overridden: bool) -> bool {
    let auto_on = in_flight(row)
        || match row.class {
            RowClass::Response => auto.responses,
            RowClass::Other => auto.others,
        };
    auto_on != overridden
}

/// Is this row a step happening **right now** — the live tail, or a tool call
/// no result has retired yet? Already said by the tone the projection gave it,
/// so in-flightness stays the query it always was rather than becoming a field.
fn in_flight(row: &Row) -> bool {
    matches!(row.tone, Tone::Live | Tone::InFlight)
}

#[cfg(test)]
mod tests;
