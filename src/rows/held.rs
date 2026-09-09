//! **The parked call as a transcript row** (DESIGN §13.7, bl-8c94): the one
//! row on the glass that is asking the operator for an answer.
//!
//! **The row is where the call already is.** The queue read says a call is
//! parked and names its `tool_use` (`codec::queue::held_at`); the transcript
//! already carries that call as a committed `ToolUse` block. So this marks the
//! row that block became rather than adding a row of its own — a second line
//! about one call is two things on the glass where there is one thing in the
//! world, and the operator would have to reconcile them.
//!
//! **What changes is the label, the hue and the payload.** The label says
//! *held for you* rather than *running*, because a call waiting on a person is
//! not work in flight; the hue is the attention accent, which STYLE.md §3
//! gives word for word to *"a parked call held for an answer"*; and the
//! payload leads with the control's decision, because that is what decides an
//! answer, with the call's own input under it a fold away.
//!
//! **One spelling for both sides of the commit.** The follow window paints the
//! same parked call before the record catches up ([`super::windowed`]) and
//! calls straight into here, so a hold reads one way on either side of the
//! cadence — §13.3's vocabulary rule at the site where two lanes describe one
//! call.
//!
//! **The sentence is split, never rewritten** (`codec::queue::folded`,
//! REMOTE §8.1). The control's sentence opens with the tool and a clip of its
//! JSON input; the row already has that input as a field, so printing the
//! clip too would be the same bytes twice, one of them truncated. The
//! decision half is what the payload leads with, and the input follows it
//! whole.

use super::build::{GEAR, key, row};
use super::{Row, RowClass, Tone};
use crate::codec::{Block, Entry, EntryKind, Held};

/// One parked call as a row: the attention label, the decision, and the call's
/// own input under the fold. An empty input is no fold at all — the empty body
/// IS the fact ([`super::build`]).
pub(super) fn held_row(key: String, tool: &str, reason: &str, input: &str) -> Row {
    let folded = crate::codec::folded(tool, reason);
    let payload = if input.is_empty() {
        folded.why
    } else {
        format!("{}\n{input}", folded.why)
    };
    row(
        key,
        format!("{GEAR} {tool} — held for you"),
        &payload,
        RowClass::Other,
        Tone::Held,
        None,
    )
}

/// Make the row carrying `held`'s call the held row. A queue that names a call
/// this transcript has not committed yet marks nothing — the honest reading,
/// and the same one `codec::queue::held_at` takes of a conversation with no
/// row.
pub(super) fn mark(entries: &[Entry], flat: &mut [Row], held: &Held) {
    let Some((at, tool, input)) = parked(entries, &held.tool_use) else {
        return;
    };
    for row in flat.iter_mut().filter(|row| row.key == at) {
        *row = held_row(at.clone(), &tool, &held.reason, &input);
    }
}

/// The key, the tool and the input of the committed block carrying `tool_use`.
/// The block's own name and not the queue's, because the row's subject is the
/// call the transcript recorded: where the two ever disagreed, relabelling a
/// transcript row from another read is the wrong half to trust.
fn parked(entries: &[Entry], tool_use: &str) -> Option<(String, String, String)> {
    entries.iter().find_map(|entry| {
        let EntryKind::Model { blocks, .. } = &entry.kind else {
            return None;
        };
        blocks
            .iter()
            .enumerate()
            .find_map(|(at, block)| match block {
                Block::ToolUse { id, name, input } if id == tool_use => {
                    Some((key(&entry.name, at), name.clone(), input.clone()))
                }
                _ => None,
            })
    })
}
