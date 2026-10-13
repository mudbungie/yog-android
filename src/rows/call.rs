//! **A call and its result, met on one row** (DESIGN §22 item 4, lernie
//! bl-eb4e). The engine writes a call and its result as two entries; the
//! operator reads them as one act — `bash("cargo test") ✔`, with what came
//! back folded beneath it. The meeting happens at the projection, keyed on the
//! tool-use id both halves carry, and the id stops there: it is the wire's
//! bookkeeping.
//!
//! **Nothing is dropped.** A result whose call is not on this read stands as a
//! row of its own ([`super::project`]), and a call no result has met yet is a
//! call row that says `running` — the two cases the meeting cannot make one,
//! painted as what they are.

use crate::codec::{Block, Entry, EntryKind};
use crate::theme::glyph;

/// How much of a call's input its head carries, in characters — the desktop's.
const HEAD: usize = 60;

/// **A call's head**: its name and the head of its input, one line, elided.
/// A JSON object's first value stands for the whole — `bash("ls -la")` reads
/// where `bash({"command":"ls -la"})` was a record — and anything else is its
/// own first line. The desktop's spelling, byte for byte.
pub(super) fn head(name: &str, input: &str) -> String {
    if input.is_empty() {
        return name.to_owned();
    }
    let argument = match serde_json::from_str::<serde_json::Value>(input) {
        Ok(serde_json::Value::Object(map)) => map
            .values()
            .next()
            .map_or_else(String::new, ToString::to_string),
        Ok(value) => value.to_string(),
        Err(_) => input.to_owned(),
    };
    let line = argument.lines().next().unwrap_or_default();
    let mut said: String = line.chars().take(HEAD).collect();
    if said.len() < argument.len() {
        said.push_str(glyph::ELIDED);
    }
    format!("{name}({said})")
}

/// **The result that met this call on this read**: its content and whether it
/// failed. Byte equality on an opaque id — the shape is the provider's and
/// this seat assumes nothing about it.
pub(super) fn result_of(entries: &[Entry], tool_use_id: &str) -> Option<(String, bool)> {
    entries.iter().find_map(|entry| match &entry.kind {
        EntryKind::ToolResult {
            tool_use_id: id,
            content,
            is_error,
        } if id == tool_use_id => Some((content.clone(), *is_error)),
        _ => None,
    })
}

/// **Is this result's call on this read?** Where it is, the result stands on
/// the call's row and has none of its own.
pub(super) fn asked(entries: &[Entry], tool_use_id: &str) -> bool {
    entries.iter().any(|entry| match &entry.kind {
        EntryKind::Model { blocks, .. } => blocks
            .iter()
            .any(|block| matches!(block, Block::ToolUse { id, .. } if id == tool_use_id)),
        _ => false,
    })
}
