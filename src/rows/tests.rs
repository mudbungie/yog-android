//! The projection's suite. It drives the one public entry point ([`rows`])
//! over hand-built entry lists, because every seam below it is internal and
//! the labels are the contract: a spelling that drifts from the desktop's is a
//! bug here whatever the intermediate shapes look like.

use std::collections::BTreeSet;

use serde_json::Value;

use super::{AutoExpand, Row, rows};
use crate::codec::{Block, Entry, EntryKind};

mod census;
mod compaction;
mod folds;
mod labels;
mod split;
mod turns;
mod wound;

/// The conversation's display name in every fixture — an agent, never a model.
const SPEAKER: &str = "yog";

/// Project under the shipped defaults: the conversation open, machinery shut.
fn go(entries: &[Entry]) -> Vec<Row> {
    rows(entries, SPEAKER, AutoExpand::default(), &BTreeSet::new())
}

/// Project with both knobs open — every step row on screen, aggregates too.
fn go_open(entries: &[Entry]) -> Vec<Row> {
    let auto = AutoExpand {
        responses: true,
        others: true,
    };
    rows(entries, SPEAKER, auto, &BTreeSet::new())
}

/// The always-visible labels, in order — what the operator actually reads.
fn prefixes(rows: &[Row]) -> Vec<String> {
    rows.iter().map(|row| row.prefix.clone()).collect()
}

fn delivered(name: &str, sender: &str, body: &str) -> Entry {
    entry(
        name,
        EntryKind::Delivered {
            sender: sender.to_string(),
            epitaph: None,
            body: body.to_string(),
        },
    )
}

fn ended(name: &str, sender: &str, epitaph: &str, body: &str) -> Entry {
    entry(
        name,
        EntryKind::Delivered {
            sender: sender.to_string(),
            epitaph: Some(epitaph.to_string()),
            body: body.to_string(),
        },
    )
}

/// A model turn whose bytes carried no readable counters — the legacy shape.
fn model(name: &str, blocks: Vec<Block>) -> Entry {
    metered(name, blocks, Value::Null)
}

/// A model turn with a committed `usage` record.
fn metered(name: &str, blocks: Vec<Block>, usage: Value) -> Entry {
    entry(
        name,
        EntryKind::Model {
            model_id: "sonnet-9".to_string(),
            blocks,
            usage,
        },
    )
}

fn text(body: &str) -> Block {
    Block::Text(body.to_string())
}

fn thought(body: &str) -> Block {
    Block::Thinking(body.to_string())
}

fn call(id: &str, name: &str, input: &str) -> Block {
    Block::ToolUse {
        id: id.to_string(),
        name: name.to_string(),
        input: input.to_string(),
    }
}

fn result(name: &str, id: &str, content: &str, is_error: bool) -> Entry {
    entry(
        name,
        EntryKind::ToolResult {
            tool_use_id: id.to_string(),
            content: content.to_string(),
            is_error,
        },
    )
}

fn streaming(name: &str, thinking: &str, text: &str) -> Entry {
    entry(
        name,
        EntryKind::Streaming {
            thinking: thinking.to_string(),
            text: text.to_string(),
        },
    )
}

fn compacted(name: &str, first: usize, last: usize, summary: &str) -> Entry {
    entry(
        name,
        EntryKind::Compacted {
            first,
            last,
            summary: summary.to_string(),
        },
    )
}

fn wounded(name: &str, wound: &str, reason: Option<&str>, auth_row: Option<&str>) -> Entry {
    entry(
        name,
        EntryKind::Wounded {
            wound: wound.to_string(),
            reason: reason.map(str::to_string),
            auth_row: auth_row.map(str::to_string),
        },
    )
}

fn raw(name: &str, bytes: &str) -> Entry {
    Entry {
        name: name.to_string(),
        raw: bytes.to_string(),
        kind: EntryKind::Raw,
    }
}

fn entry(name: &str, kind: EntryKind) -> Entry {
    Entry {
        name: name.to_string(),
        raw: String::new(),
        kind,
    }
}
