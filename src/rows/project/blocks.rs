//! **What one model content block becomes** — split from [`super`] at the
//! design-time budget on the seam that module's match already draws: up there
//! an *entry* picks its arm, and exactly one of those arms fans out over the
//! blocks a turn committed. The two change for unrelated reasons — a new entry
//! kind on the wire touches only the match, a new block kind only this file.

use super::super::build::row;
use super::super::{Role, Row, RowClass, Tone};
use crate::codec::{Block, Entry, EntryKind};

/// One model content block as a row. A tool call still awaiting its result
/// says so in words beside the pulse — the hue is never the only carrier.
///
/// A tool call's payload is the input the wire already capped; this seat does
/// not summarize a second time, because two truncations of one string are two
/// answers to one question.
pub(super) fn block_row(
    entries: &[Entry],
    key: String,
    speaker: &str,
    model_id: &str,
    block: &Block,
) -> Row {
    match block {
        Block::Text(text) => Row {
            hover: model_hover(model_id),
            ..row(
                key,
                format!("{speaker}:"),
                text,
                RowClass::Response,
                Tone::Plain,
                Some(Role::Model),
            )
        },
        Block::Thinking(text) => row(
            key,
            "thinking:".to_string(),
            text,
            RowClass::Other,
            Tone::Weak,
            None,
        ),
        Block::ToolUse { id, name, input } => {
            let running = unresolved(entries, id);
            let prefix = if running {
                format!("⚙ {name} — running")
            } else {
                format!("⚙ {name}")
            };
            let tone = if running { Tone::InFlight } else { Tone::Plain };
            row(key, prefix, input, RowClass::Other, tone, None)
        }
    }
}

/// Has no result retired this call yet? A question about the *rest* of the
/// transcript, not about the call — which is why the whole list comes down
/// here. Byte equality on an opaque id: the shape is the provider's and this
/// seat assumes nothing about it.
fn unresolved(entries: &[Entry], tool_use_id: &str) -> bool {
    !entries.iter().any(|entry| {
        matches!(&entry.kind, EntryKind::ToolResult { tool_use_id: id, .. } if id == tool_use_id)
    })
}

/// What a model turn's speaker label stands for: the model that ran it. The
/// model id is a **config** fact — which model the conversation's governing
/// commit assigned — not a speaker, so it rides the hover while the label
/// names the agent. One turn can name a different model than the header's
/// current assignment, and that is the truth of that turn.
pub(super) fn model_hover(model_id: &str) -> String {
    format!("ran on {model_id} — the model is config, not the speaker")
}
