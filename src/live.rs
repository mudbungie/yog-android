//! **The streaming tail's one rule** (bl-e3d1): the transcript a frame paints
//! carries exactly one tail, freshened while a turn is in flight and gone
//! when it is not — and, since the lane widened (REMOTE §5.5, PROTOCOL 15),
//! the tool window beside it.
//!
//! **Why there was ever more than one.** The engine writes the growing answer
//! into the transcript itself as an `EntryKind::Streaming`, so a cadence read
//! already carries a tail; bl-4822 then added the follow lane, which carries
//! the same answer as it is written. Painting the lane's fold *beside* the
//! transcript put the same words on the glass twice — and after the turn
//! committed, the settled row and a tail the read still carried made a
//! finished reply read as two different people saying the same thing.
//!
//! So the lane does not paint anything of its own. It **replaces** the
//! transcript's own tail with a fresher copy of the same entry, which leaves
//! one row on the glass with one label, and makes the dedupe structural
//! rather than a content match: when the engine's read stops carrying a tail,
//! there is nothing to replace and nothing to dissolve.
//!
//! **The prose is painted while a model call is streaming, and not merely
//! while the step is live** (REMOTE §5.5, bl-ee21). The gate used to be the
//! row's `flight` at all, which was the same reading back when the lane ENDED
//! the instant a call settled — the seat's fold was emptied by the stream's
//! end and there was nothing stale to hold. The lane now follows the STEP, so
//! it stays open through the tool phase with the settled call's words still in
//! the fold, and the engine's own tail (`live_tail`) is gated on `InFlight` —
//! it stops carrying one at exactly that moment. Painting the fold on any
//! flight would therefore put the committed answer on the glass a second time
//! for as long as the tools run. So this reads the same two liveness questions
//! §5.5 tells apart, off the field the engine already states on the row:
//! `Flight::Inference` opens the prose, any flight at all opens the window.
//!
//! **And at rest there is neither.** `flight` is the row's own (REMOTE §9.4
//! puts the gate on the row), so a conversation the engine says is not working
//! shows no growing text and no window — whatever the response file still
//! holds.
//!
//! **A held call is the one the window exists for** (PROTOCOL 18, yog
//! bl-58bb). It lands neither of the two files the pair of transitions above
//! is made of, so before the field the lane simply had nothing to carry — and
//! since the lane's subject is the STEP, what it reported was a conversation
//! at rest at the exact moment the operator was the thing it was waiting for.
//! It is a windowed entry like any other, and the row it becomes wears the
//! attention accent (`rows::windowed`), which is the one state on this glass
//! that means *asking for you*.
//!
//! **The window paints only what the record does not carry yet.** A call whose
//! `tool_use` a committed block already names is the transcript's to paint —
//! it is there with its input and its running mark — so the lane adds a row
//! only for a call the cadence read has not caught up with. That is the same
//! structural dedupe the tail has, keyed on the id the two sides share rather
//! than on what either says.

use crate::codec::{Block, Call, Entry, EntryKind, Flight, Stream};

/// The name the engine gives its streaming entry, and therefore the name a
/// replacement wears: the row keys a fold override is remembered by are built
/// from it, so a tail that changed its name mid-turn would drop the
/// operator's own flips.
const NAME: &str = "streaming";

/// The name space a windowed call's row is keyed under — `window/<tool_use>`,
/// so the key is stable across the stateless re-read for exactly as long as
/// the call is, and cannot collide with a transcript filename.
const WINDOW: &str = "window";

/// The transcript as it should paint.
pub fn settled(
    transcript: Vec<Entry>,
    live: Option<&Stream>,
    flight: Option<Flight>,
) -> Vec<Entry> {
    let streaming = flight == Some(Flight::Inference);
    let fresh = live.filter(|_| streaming).and_then(entry_of);
    let mut out: Vec<Entry> = if streaming && fresh.is_none() {
        // The read's own tail is the freshest thing there is until the lane's
        // first frame lands, and dropping it for half a second is a row that
        // flickers.
        transcript
    } else {
        transcript
            .into_iter()
            .filter(|entry| !matches!(entry.kind, EntryKind::Streaming { .. }))
            .collect()
    };
    let windowed = windowed(live.filter(|_| flight.is_some()), &out);
    out.extend(windowed);
    // The tail goes last because it IS the tail: the engine writes it at the
    // end of the response file, and everything before it has committed.
    out.extend(fresh);
    out
}

/// The lane's fold as the entry it replaces. An answer that has begun and
/// said nothing is no entry: a growing row with nothing in it is a row.
fn entry_of(stream: &Stream) -> Option<Entry> {
    let thinking = stream.thinking.clone().unwrap_or_default();
    let text = stream.text.clone().unwrap_or_default();
    if thinking.is_empty() && text.is_empty() {
        return None;
    }
    Some(Entry {
        name: NAME.to_owned(),
        raw: String::new(),
        kind: EntryKind::Streaming { thinking, text },
    })
}

/// The window's calls as entries, minus the ones `committed` already carries.
fn windowed(live: Option<&Stream>, committed: &[Entry]) -> Vec<Entry> {
    live.map(Stream::window)
        .unwrap_or_default()
        .iter()
        .filter(|call| !carried(committed, &call.tool_use))
        .map(window_entry)
        .collect()
}

/// One call as the entry the projection paints. A call whose record carried no
/// readable name is labelled by its own id — the operator is being told a
/// command is running on their machine, and the id is the least this lane can
/// say while still saying it.
fn window_entry(call: &Call) -> Entry {
    Entry {
        name: format!("{WINDOW}/{}", call.tool_use),
        raw: String::new(),
        kind: EntryKind::Windowed {
            tool: call.tool.clone().unwrap_or_else(|| call.tool_use.clone()),
            input: call.input.clone().unwrap_or_default(),
            exit_code: call.exit_code,
            held: call.held.clone(),
        },
    }
}

/// Does the record already carry this call? Byte equality on the provider's
/// opaque id, exactly as `rows::project::blocks` asks the neighbouring
/// question of a committed block: the shape is the provider's and this seat
/// assumes nothing about it.
fn carried(entries: &[Entry], tool_use: &str) -> bool {
    entries.iter().any(|entry| match &entry.kind {
        EntryKind::Model { blocks, .. } => blocks
            .iter()
            .any(|block| matches!(block, Block::ToolUse { id, .. } if id == tool_use)),
        _ => false,
    })
}

#[cfg(test)]
mod tests;
