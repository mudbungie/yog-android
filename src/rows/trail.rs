//! **The transcript's idiom as data** (DESIGN §22 item 4; lernie §4.42 item
//! 4): what the glass groups the projected rows into, and the glyph each
//! stands behind. Pure, so the shape is asserted here and the shell only
//! paints it.
//!
//! Every block starts at one left edge with its glyph in the gutter — `›` for
//! the operator, `¦` for the model, `·` for a peer, a tool's answer or the
//! machinery. A run of steps — calls, results, reasoning — stands under one
//! fold head, `tool calls (N)`, each step on a stem; a `response` elbow parts
//! a trail from the prose after it, a hairline parts one turn from the next,
//! and a blank stands only where the kind of block changes.
//!
//! **A finished turn's rollup IS its trail's head** ([`super::turns`]): it
//! already folds the turn's machinery and counts it. A run that did not roll
//! up — a turn still running, or one that ended on machinery — gets a head of
//! its own here, **open** until the operator shuts it, because what is still
//! happening is the show; its key is the run's first entry with `#trail`
//! where the rollup's has `#turn`, so a flip on one never lands on the other.

use std::collections::BTreeSet;

use super::{Fold, Kind, Role, Row, RowClass, Tone};
use crate::theme::glyph;

/// The fold head's word, with the calls under it (the desktop's `TRAIL`).
pub(super) fn head_word(calls: usize) -> String {
    format!("tool calls ({calls})")
}

/// Key suffix of a head this pass makes, where the rollup's says `turn`.
const TRAIL_SUFFIX: &str = "trail";

/// What a line is on the glass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    /// Something said, at the gutter.
    Said,
    /// A trail's fold head.
    Head,
    /// A step on the trail's stem; `last` ends the stem in an elbow.
    Step { last: bool },
}

/// What stands between a line and the one above it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Before {
    Nothing,
    /// A blank: the kind of block changed.
    Gap,
    /// The hairline between two turns: the operator spoke again.
    Parting,
    /// The `response` elbow: prose after a trail.
    Response,
}

/// One painted line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub row: Row,
    pub part: Part,
    pub before: Before,
}

/// **The projected rows as the glass groups them.** `folds` is the same
/// override set [`super::rows`] reads, so a head made here flips exactly as a
/// rollup does.
pub fn lines(rows: &[Row], folds: &BTreeSet<String>) -> Vec<Line> {
    let mut out: Vec<Line> = Vec::new();
    let mut turns = 0;
    let mut at = 0;
    while let Some(row) = rows.get(at) {
        let headed = usize::from(row.kind == Kind::Trail);
        let rest = rows.get(at + headed..).unwrap_or_default();
        let run = rest.iter().take_while(|step| step.kind.is_step()).count();
        let steps = rest.get(..run).unwrap_or_default();
        let before = match out.last() {
            None => Before::Nothing,
            Some(above) => between(above, row, turns),
        };
        if headed == 1 || run > 0 {
            let head = if headed == 1 {
                row.clone()
            } else {
                made(steps, folds)
            };
            let open = head.expanded;
            out.push(Line {
                row: head,
                part: Part::Head,
                before,
            });
            if open {
                out.extend(steps.iter().enumerate().map(|(i, step)| Line {
                    row: step.clone(),
                    part: Part::Step { last: i + 1 == run },
                    before: Before::Nothing,
                }));
            }
            at += headed + run;
            continue;
        }
        turns += usize::from(row.role == Some(Role::User));
        out.push(Line {
            row: row.clone(),
            part: Part::Said,
            before,
        });
        at += 1;
    }
    out
}

/// What parts `row` from the line above it. `turns` counts the operator's
/// turns already painted.
fn between(above: &Line, row: &Row, turns: usize) -> Before {
    // A head only ever follows something said: a run of steps is maximal,
    // and a rollup stands after the boundary that opened its turn.
    if row.kind != Kind::Said {
        return Before::Gap;
    }
    let after_trail = above.part != Part::Said;
    if row.role == Some(Role::User) {
        if turns > 0 {
            Before::Parting
        } else {
            Before::Gap
        }
    } else if after_trail {
        Before::Response
    } else if glyph(&above.row) == glyph(row) {
        Before::Nothing
    } else {
        Before::Gap
    }
}

/// **The head over a run that did not roll up**: open until flipped.
fn made(steps: &[Row], folds: &BTreeSet<String>) -> Row {
    let first = steps.first().map_or("", |step| step.key.as_str());
    let entry = first.rsplit_once('#').map_or(first, |(head, _)| head);
    let key = format!("{entry}#{TRAIL_SUFFIX}");
    let calls = steps
        .iter()
        .filter(|step| matches!(step.kind, Kind::Call | Kind::Answered))
        .count();
    Row {
        expanded: !folds.contains(&key),
        key,
        prefix: head_word(calls),
        preview: String::new(),
        body: String::new(),
        hover: String::new(),
        class: RowClass::Other,
        tone: Tone::Weak,
        role: None,
        fold: Fold::Steps,
        kind: Kind::Trail,
    }
}

/// **The glyph a row stands behind**: the gutter's for something said, the
/// stem's for a step, the fold's for a head. A wound — the conversation's
/// own failure — stands behind `✖`, as on the desktop.
pub fn glyph(row: &Row) -> String {
    let failed = row.tone == Tone::Bad;
    let said = match row.kind {
        Kind::Trail if row.expanded => glyph::OPEN,
        Kind::Trail => glyph::SHUT,
        Kind::Call | Kind::Answered => glyph::CALL,
        Kind::Result | Kind::Said if failed => glyph::FAILED,
        Kind::Result => glyph::DONE,
        Kind::Remark => glyph::MODEL,
        Kind::Thinking => glyph::NOTE,
        Kind::Said => match row.role {
            Some(Role::User) => glyph::PROMPT,
            Some(Role::Model) => glyph::MODEL,
            _ => glyph::NOTE,
        },
    };
    said.to_owned()
}

/// **How a met call ended**, beside its head: `✔`, or `✖` where it failed.
pub fn mark(row: &Row) -> Option<String> {
    (row.kind == Kind::Answered).then(|| {
        if row.tone == Tone::Bad {
            glyph::FAILED
        } else {
            glyph::DONE
        }
        .to_owned()
    })
}
