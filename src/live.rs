//! **The streaming tail's one rule** (bl-e3d1): the transcript a frame paints
//! carries exactly one tail, freshened while a turn is in flight and gone
//! when it is not.
//!
//! **Why there was ever more than one.** The engine writes the growing answer
//! into the transcript itself as an `EntryKind::Streaming`, so a cadence read
//! already carries a tail; bl-4822 then added the follow lane, which reads
//! the same answer four times a rest. Painting the lane's fold *beside* the
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
//! **And at rest there is no tail at all.** `in_flight` is the row's own
//! `flight` (REMOTE §9.4 puts the gate on the row), so a conversation the
//! engine says is not writing shows no growing text — whatever the response
//! file still holds. That is the flight-end half of the same defect: a tail
//! left standing under a settled reply is stale by the engine's own statement.

use crate::codec::{Entry, EntryKind, Stream};

/// The name the engine gives its streaming entry, and therefore the name a
/// replacement wears: the row keys a fold override is remembered by are built
/// from it, so a tail that changed its name mid-turn would drop the
/// operator's own flips.
const NAME: &str = "streaming";

/// The transcript as it should paint.
pub fn settled(transcript: Vec<Entry>, live: Option<&Stream>, in_flight: bool) -> Vec<Entry> {
    let Some(fresh) = live.filter(|_| in_flight).and_then(entry_of) else {
        // Nothing to replace it with, so there are only two answers: at rest
        // no tail at all, and in flight the read's own tail — which is the
        // freshest thing there is until the lane's first read lands, and
        // whose absence for half a second would be a row that flickers.
        return if in_flight {
            transcript
        } else {
            transcript
                .into_iter()
                .filter(|entry| !matches!(entry.kind, EntryKind::Streaming { .. }))
                .collect()
        };
    };
    let mut out: Vec<Entry> = transcript
        .into_iter()
        .filter(|entry| !matches!(entry.kind, EntryKind::Streaming { .. }))
        .collect();
    // The tail goes last because it IS the tail: the engine writes it at the
    // end of the response file, and everything before it has committed.
    out.push(fresh);
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

#[cfg(test)]
mod tests;
