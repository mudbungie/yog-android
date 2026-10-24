//! **What one model content block becomes** — split from [`super`] at the
//! design-time budget on the seam that module's match already draws: up there
//! an *entry* picks its arm, and exactly one of those arms fans out over the
//! blocks a turn committed. The two change for unrelated reasons — a new entry
//! kind on the wire touches only the match, a new block kind only this file.

use super::super::build::{as_, row};
use super::super::call::{head, result_of};
use super::super::{Kind, Role, Row, RowClass, Tone};
use crate::codec::{Block, Entry};

/// One model content block as a row. A tool call still awaiting its result
/// says so in words beside the pulse — the hue is never the only carrier —
/// and its payload is the input the wire already capped. **A call its result
/// has met is one row** ([`super::super::call`]): its head names the call and
/// the result is its payload, folded to its first line and unfolded in place;
/// a failed one wears the error accent and nothing else does, because how a
/// call ended is the trail's mark, not a hue over the whole step. **The
/// model's prose and reasoning say the model id** (`<model id>`, `<model id>
/// (thinking)`; DESIGN §23 item 15), as the desktop's do.
pub(super) fn block_row(entries: &[Entry], key: String, model_id: &str, block: &Block) -> Row {
    match block {
        Block::Text(text) => row(
            key,
            model_id.to_owned(),
            text,
            RowClass::Response,
            Tone::Plain,
            Some(Role::Model),
        ),
        Block::Thinking(text) => as_(
            row(
                key,
                format!("{model_id} ({})", super::THINKING),
                text,
                RowClass::Other,
                Tone::Weak,
                None,
            ),
            Kind::Thinking,
        ),
        Block::ToolUse { id, name, input } => match result_of(entries, id) {
            None => as_(
                row(
                    key,
                    format!("{} — running", head(name, input)),
                    input,
                    RowClass::Other,
                    Tone::InFlight,
                    None,
                ),
                Kind::Call,
            ),
            Some((content, failed)) => {
                let tone = if failed { Tone::Bad } else { Tone::Plain };
                let said = row(
                    key,
                    head(name, input),
                    &content,
                    RowClass::Other,
                    tone,
                    None,
                );
                as_(said, Kind::Answered)
            }
        },
        // A block kind a newer server of this major spells (REMOTE §3.2). The
        // word is the whole of what this build knows about it, and it is said
        // rather than dropped: a turn missing a row is a turn misread.
        Block::Unknown(word) => row(
            key,
            crate::codec::unknown_label("block", word),
            "",
            RowClass::Other,
            Tone::Weak,
            None,
        ),
    }
}
