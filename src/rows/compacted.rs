//! **The row that stands for what is NOT in the record** — the compaction
//! marker, split off [`super::project`] on the seam that module's match
//! already draws: every other arm there projects something somebody said or a
//! tool answered, and this one projects a *hole*.

use super::build::{key, row};
use super::{Row, RowClass, Tone};

/// The compaction marker's glyph: this was **cut out**. Per the glyph doctrine
/// the words beside it carry the meaning and the glyph only recognizes it.
const GAP_GLYPH: &str = "✂";
/// What the marker stands for, on hover. It states the derivation's own limit
/// as plainly as its finding: nothing on disk links a summary to the span it
/// replaced, so the compactor's summaries ride the conversation's first cut
/// mark rather than being guessed onto a gap apiece.
const COMPACTED_HOVER: &str = "These entries were removed: lernie's compactor squashed them out of the record and \
     wrote a summary in their place. The counter proves which entries are gone; nothing \
     on disk says which summary replaced which span, so every summary this conversation \
     has opens from its first cut mark, in the order they were written.";
/// The payload of a marker carrying no part of the record — a later gap, or a
/// compaction that wrote no summary this pane can read.
const NO_SUMMARY: &str = "(no summary on this mark)";

/// **The record was rewritten here** — never another turn in the conversation.
/// The summary is the compactor model's own prose, not the operator's and not
/// this agent's, so the row wears the empty role seat (nobody is speaking),
/// the machinery knob's class and the weak tone: one faded line stating what
/// is missing, folding open onto what lernie put in its place. A mark carrying
/// no part of the record still says the entries are gone — what the counter
/// proves never depends on a summary existing.
pub(super) fn compacted_row(name: &str, first: usize, last: usize, summary: &str) -> Row {
    Row {
        hover: COMPACTED_HOVER.to_string(),
        ..row(
            key(name, 0),
            compacted_prefix(first, last),
            if summary.is_empty() {
                NO_SUMMARY
            } else {
                summary
            },
            RowClass::Other,
            Tone::Weak,
            None,
        )
    }
}

/// The prefix seat of a compaction marker: **how many** entries are gone and
/// **which** counter values they were, in the always-visible slot — the two
/// facts the surviving counter proves, and the whole of what this seat may
/// assert about a span it never saw. The span reads as one number when one
/// entry went.
fn compacted_prefix(first: usize, last: usize) -> String {
    let count = last.saturating_sub(first) + 1;
    let span = if first == last {
        format!("{first:03}")
    } else {
        format!("{first:03}–{last:03}")
    };
    let entries = if count == 1 { "entry" } else { "entries" };
    format!("{GAP_GLYPH} {count} {entries} compacted away — {span}")
}
