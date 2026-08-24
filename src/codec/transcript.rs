//! The transcript entry — the mirror of the server's `transcript::wire`
//! spelling: one row per entry, its filename, its kind token, whatever that
//! kind can say, and the raw text it was read from. An unparseable entry
//! stays distinguishable from a parsed one on the wire exactly as it does on
//! screen. `usage` is an open vocabulary by the parent's own ruling (the
//! provider's counters, unpinned), so it rides as raw JSON here too.

use serde_json::Value;

use super::fields::{arr_of, bool_of, opt, str_of, usize_of};

/// One transcript entry: the file, the parse, and the bytes as text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    /// The backing bytes, decoded lossily by the server before the wire.
    pub raw: String,
    pub kind: EntryKind,
}

/// What one entry is — the §4.4 canonical kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryKind {
    /// An operator/peer message delivered into the conversation.
    Delivered {
        sender: String,
        epitaph: Option<String>,
        body: String,
    },
    /// A model turn: its id, its content blocks, its committed counters.
    Model {
        model_id: String,
        blocks: Vec<Block>,
        usage: Value,
    },
    /// One tool call's result.
    ToolResult {
        tool_use_id: String,
        content: String,
        is_error: bool,
    },
    /// The live streaming tail of an in-flight call.
    Streaming { thinking: String, text: String },
    /// A compaction marker: which entries it folded, and the summary.
    Compacted {
        first: usize,
        last: usize,
        summary: String,
    },
    /// An entry the parser could not read — surfaced, never dropped.
    Raw,
}

/// One canonical content block. A tool call carries the summary the chip
/// renders, never a second parse of the input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Text(String),
    Thinking(String),
    ToolUse {
        id: String,
        name: String,
        input: String,
    },
}

/// Read one entry row, strictly.
pub(crate) fn entry(v: &Value) -> Result<Entry, String> {
    let o = v.as_object().ok_or("transcript entry: not an object")?;
    let kind = match str_of(o, "kind")?.as_str() {
        "delivered" => EntryKind::Delivered {
            sender: str_of(o, "sender")?,
            epitaph: opt(o, "epitaph", str_of)?,
            body: str_of(o, "body")?,
        },
        "model" => EntryKind::Model {
            model_id: str_of(o, "model_id")?,
            blocks: arr_of(o, "blocks")?
                .iter()
                .map(block)
                .collect::<Result<_, _>>()?,
            usage: o
                .get("usage")
                .cloned()
                .ok_or("model entry: missing field \"usage\"")?,
        },
        "tool-result" => EntryKind::ToolResult {
            tool_use_id: str_of(o, "tool_use_id")?,
            content: str_of(o, "content")?,
            is_error: bool_of(o, "is_error")?,
        },
        "streaming" => EntryKind::Streaming {
            thinking: str_of(o, "thinking")?,
            text: str_of(o, "text")?,
        },
        "compacted" => EntryKind::Compacted {
            first: usize_of(o, "first")?,
            last: usize_of(o, "last")?,
            summary: str_of(o, "summary")?,
        },
        "raw" => EntryKind::Raw,
        other => return Err(format!("transcript entry: unknown kind {other:?}")),
    };
    Ok(Entry {
        name: str_of(o, "name")?,
        raw: str_of(o, "raw")?,
        kind,
    })
}

/// Read one content block, strictly.
fn block(v: &Value) -> Result<Block, String> {
    let o = v.as_object().ok_or("content block: not an object")?;
    match str_of(o, "kind")?.as_str() {
        "text" => Ok(Block::Text(str_of(o, "text")?)),
        "thinking" => Ok(Block::Thinking(str_of(o, "text")?)),
        "tool-use" => Ok(Block::ToolUse {
            id: str_of(o, "id")?,
            name: str_of(o, "name")?,
            input: str_of(o, "input")?,
        }),
        other => Err(format!("content block: unknown kind {other:?}")),
    }
}

#[cfg(test)]
mod tests;
