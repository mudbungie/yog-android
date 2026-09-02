//! One transcript entry → its rows: the exhaustive per-variant match and the
//! labels each variant wears. The row *vocabulary* it builds with lives in
//! [`super`], which is the only caller; **how** a row is made of those parts
//! is [`super::build`], and the one arm that projects a *hole* rather than
//! something somebody said is [`super::compacted`].
//!
//! Every literal here is a user-visible spelling shared with the desktop seat.
//! Where the two disagree one of them is a bug, so they are pinned by the
//! suite rather than trusted to stay in step.

use super::build::{key, row, with_size};
use super::compacted::compacted_row;
use super::{Role, Row, RowClass, Tone};
use crate::codec::{Entry, EntryKind};
use blocks::{block_row, model_hover};

mod blocks;

/// The reserved sender token naming the operator.
const USER_SENDER: &str = "user";
/// A result deposit can assert an ending and no content, so its body is empty
/// once the envelope is off. Say so, rather than paint a blank line from a
/// stranger.
const NO_BODY: &str = "(no message body)";
/// A model entry that committed no content blocks at all — surfaced, because a
/// turn that said nothing is a fact about the run and not an absence of one.
const NO_BLOCKS: &str = "(no content blocks)";
/// The ok/error result seats. Per the glyph doctrine the glyph is never the
/// outcome's only carrier, so the phrase rides with it in this one home and no
/// renderer invents its own wording.
const OK_RESULT: &str = "✔ tool result — ok";
/// The error half of that same pair.
const ERR_RESULT: &str = "✖ tool result — error";

/// Rows for one entry: one per model content block, else one for the entry.
/// `entries` is the whole list because a tool call's in-flightness is a
/// question about the *rest* of the transcript, not about the call.
pub(super) fn push_entry(entries: &[Entry], entry: &Entry, speaker: &str, out: &mut Vec<Row>) {
    match &entry.kind {
        EntryKind::Delivered {
            sender,
            epitaph,
            body,
        } => {
            let (payload, tone) = if body.is_empty() {
                (NO_BODY, Tone::Weak)
            } else {
                (body.as_str(), Tone::Plain)
            };
            out.push(row(
                key(&entry.name, 0),
                delivered_prefix(sender, epitaph.as_deref()),
                payload,
                RowClass::Response,
                tone,
                Some(message_role(sender, epitaph.is_some())),
            ));
        }
        EntryKind::Model {
            model_id, blocks, ..
        } if blocks.is_empty() => out.push(Row {
            hover: model_hover(model_id),
            ..row(
                key(&entry.name, 0),
                format!("{speaker}:"),
                NO_BLOCKS,
                RowClass::Other,
                Tone::Weak,
                Some(Role::Model),
            )
        }),
        EntryKind::Model {
            model_id, blocks, ..
        } => {
            for (i, block) in blocks.iter().enumerate() {
                out.push(block_row(
                    entries,
                    key(&entry.name, i),
                    speaker,
                    model_id,
                    block,
                ));
            }
        }
        // The size hint takes the prefix seat because it has to be legible
        // *contracted*, which the hover is not (it needs a pointer this seat
        // does not have) and the preview is not (it is the payload's own first
        // line) — and it trails the outcome, so the row still leads with what
        // it is.
        EntryKind::ToolResult {
            content, is_error, ..
        } => {
            let (prefix, tone) = if *is_error {
                (ERR_RESULT, Tone::Bad)
            } else {
                (OK_RESULT, Tone::Good)
            };
            out.push(with_size(row(
                key(&entry.name, 0),
                prefix.to_string(),
                content,
                RowClass::Other,
                tone,
                None,
            )));
        }
        EntryKind::Streaming { thinking, text } => {
            push_streaming(&entry.name, speaker, thinking, text, out);
        }
        EntryKind::Compacted {
            first,
            last,
            summary,
        } => out.push(compacted_row(&entry.name, *first, *last, summary)),
        EntryKind::Raw => out.push(row(
            key(&entry.name, 0),
            entry.name.clone(),
            &entry.raw,
            RowClass::Other,
            Tone::Weak,
            None,
        )),
    }
}

/// The prefix seat of a delivered message: the sender, plus **how it ended**
/// when the envelope asserted an ending. An `epitaph:` marks the message as a
/// *result deposit* — a child's terminal, arriving because this agent
/// dispatched it, not because someone chose to speak — and on a `stopped` /
/// `died` one it is the entire message.
///
/// The token rides through verbatim, which is the desktop's mapping with the
/// table removed rather than a deviation from it: that table parses four known
/// spellings to themselves and any other value to itself, so it is the
/// identity, and the wire hands this seat the token already.
fn delivered_prefix(sender: &str, epitaph: Option<&str>) -> String {
    match epitaph {
        Some(epitaph) => format!("{sender} ended: {epitaph}"),
        None => format!("{sender}:"),
    }
}

/// The role of a delivered message, from the two facts its bytes assert: who
/// sent it and whether it carries an `epitaph:`. The epitaph wins — a result
/// deposit is a kind before it is a sender — then the reserved `user` token,
/// then the peer catch-all, which is the general path so an unknown sender
/// reads as third-party mail and never as the operator.
fn message_role(sender: &str, has_epitaph: bool) -> Role {
    if has_epitaph {
        Role::Ended
    } else if sender == USER_SENDER {
        Role::User
    } else {
        Role::Peer
    }
}

/// The live tail is up to **two** rows and they are the same two a committed
/// model turn has — reasoning, then the answer. Each keeps its committed
/// counterpart's class, so the fold knobs mean one thing on either side of the
/// commit; what differs is the tone, and [`Tone::Live`] is what auto-expands
/// them while the step is happening. An empty half is no row at all: a model
/// that has only thought so far shows one growing row, not one growing row and
/// one blank one.
///
/// **Both rows wear their committed counterpart's label** (operator ruling,
/// bl-e3d1): the growing text is the speaking agent's own row, so it says
/// `<speaker>:` exactly as the settled turn will, and the reasoning says
/// `thinking:` exactly as a committed thinking block does. It used to say
/// `live:`, which put a word that is not a speaker in the speaker's seat —
/// a §13.3 vocabulary break — and made a finished reply read as two
/// different people saying the same thing.
fn push_streaming(name: &str, speaker: &str, thinking: &str, text: &str, out: &mut Vec<Row>) {
    if !thinking.is_empty() {
        out.push(row(
            key(name, 0),
            "thinking:".to_string(),
            thinking,
            RowClass::Other,
            Tone::Live,
            None,
        ));
    }
    if !text.is_empty() {
        out.push(row(
            key(name, 1),
            format!("{speaker}:"),
            text,
            RowClass::Response,
            Tone::Live,
            Some(Role::Model),
        ));
    }
}
