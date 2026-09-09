//! **What a projected row is to the grouping** — the boundary it must not
//! cross and the provenance its aggregate counts — plus the committed counters
//! that provenance points back to. Split from [`super`] on the seam between
//! *classifying a row* and *deciding where a turn is*.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::codec::{Block, EntryKind};

/// One entry's committed token counters, summed under their own names.
///
/// The wire carries `usage` as raw JSON because the counter vocabulary is the
/// provider's and is deliberately unpinned. It is narrowed to counted integers
/// exactly here, at the one place a number is wanted: a counter whose value is
/// not a whole number is not a count, and a census may not report what it
/// cannot read.
pub(in crate::rows) type Usage = BTreeMap<String, u64>;

/// What a projected row is *to the grouping*. Derived from the entry the row
/// came from ([`step_of`]) — nothing new is stored on the row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::rows) enum Step {
    /// A delivered message, or a compaction marker: a turn boundary.
    Boundary,
    /// Model-authored output with no term of its own — an intermediate text
    /// block, or the one row an entry that committed no blocks gets. It still
    /// witnesses the inference call its entry stands for.
    Model,
    /// One thinking block.
    Thinking,
    /// One tool call.
    ToolCall,
    /// Not model-authored: a tool result, raw bytes, the live tail.
    Plain,
}

/// Which [`Step`] the `block`-th row of an entry of this kind is — the
/// one-row-per-block correspondence [`super::super::project`] emits.
pub(in crate::rows) fn step_of(kind: &EntryKind, block: usize) -> Step {
    match kind {
        // A compaction marker is a **boundary** for the same reason a
        // delivered message is: it is not something the agent did between two
        // things it said. It says the record was rewritten here, and a turn
        // that swallowed it into its aggregate would hide the one row saying
        // so behind a fold. The wound is one for the same reason: it says the
        // conversation is not coming back, and it must not be swallowed into
        // the last turn's census as if it were a step of it.
        EntryKind::Delivered { .. } | EntryKind::Compacted { .. } | EntryKind::Wounded { .. } => {
            Step::Boundary
        }
        EntryKind::Model { blocks, .. } => match blocks.get(block) {
            Some(Block::Thinking(_)) => Step::Thinking,
            Some(Block::ToolUse { .. }) => Step::ToolCall,
            // A block kind this build has not heard of (REMOTE §3.2) counts
            // as model-authored prose, which is what every block that is
            // neither a thought nor a call has ever been.
            Some(Block::Text(_) | Block::Unknown(_)) | None => Step::Model,
        },
        // The follow window's own row is a tool call — it is the same call
        // the block will be once the record catches up (REMOTE §5.5), and a
        // census that counted it as anything else would say a different number
        // about one turn depending on which side of the commit the read landed.
        EntryKind::Windowed { .. } => Step::ToolCall,
        // `Unknown` sits with `Raw` for the reason the two are one shape: an
        // entry whose kind this build cannot read is not something the agent
        // is known to have DONE, so it counts as nothing in a turn's census.
        EntryKind::ToolResult { .. }
        | EntryKind::Streaming { .. }
        | EntryKind::Raw
        | EntryKind::Unknown(_) => Step::Plain,
    }
}

/// The committed usage record every row of a model entry points back to — the
/// third parallel projection [`super::super::rows`] builds beside the rows and
/// their steps. Empty for anything not model-authored, and empty too for a
/// model entry whose bytes carried no readable report: the census words a
/// mixed turn for exactly that case, so the two need not be told apart here.
pub(in crate::rows) fn usage_of(kind: &EntryKind) -> Usage {
    match kind {
        EntryKind::Model { usage, .. } => counters(usage),
        EntryKind::Delivered { .. }
        | EntryKind::ToolResult { .. }
        | EntryKind::Streaming { .. }
        | EntryKind::Windowed { .. }
        | EntryKind::Compacted { .. }
        | EntryKind::Wounded { .. }
        | EntryKind::Raw
        | EntryKind::Unknown(_) => Usage::new(),
    }
}

/// The readable counters of one raw `usage` value. Anything that is not an
/// object of whole numbers contributes nothing rather than a guess.
fn counters(usage: &Value) -> Usage {
    usage
        .as_object()
        .map(|fields| {
            fields
                .iter()
                .filter_map(|(name, value)| Some((name.clone(), value.as_u64()?)))
                .collect()
        })
        .unwrap_or_default()
}
