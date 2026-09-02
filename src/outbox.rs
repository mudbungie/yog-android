//! **The outbox's one rule** (bl-66fb): has the message this seat just
//! deposited come back in a transcript read?
//!
//! The local echo itself is the shell's — it is the composer's own state and
//! dies with the screen — but *whether it is still an echo* is a fact about
//! the engine's transcript, which is exactly the kind of reading that belongs
//! under the coverage floor rather than in a paint file.
//!
//! **It matches on content, and that is a known weakness with a named
//! remedy.** A deposit's receipt is an `outcome` and carries no id this codec
//! reads, so there is nothing to match ON but the text — and two identical
//! consecutive messages ("ok", then "ok") are indistinguishable: the first
//! one's row dissolves the second one's echo, and the second message paints
//! as taken a read early. The honest fix is upstream, not here: a deposit
//! receipt that named the entry it wrote would make this exact
//! (`Reply::Outcome` has no field for it, so it is a REMOTE ask, not a shim).
//! Until then this reads the TAIL rather than the whole transcript, so an
//! identical message far up the conversation cannot dissolve anything.

use crate::codec::{Entry, EntryKind};

/// How far back a delivered row may be and still be this echo's. A deposit
/// lands at the end of a transcript; anything older is another message that
/// happened to say the same thing.
const TAIL: usize = 4;

/// Whether `text` has appeared as a delivered message in the transcript's
/// tail — the moment the echo stops being an echo and becomes a row.
pub fn taken(transcript: &[Entry], text: &str) -> bool {
    let from = transcript.len().saturating_sub(TAIL);
    transcript.get(from..).is_some_and(|tail| {
        tail.iter().any(|entry| match &entry.kind {
            EntryKind::Delivered { body, .. } => body.trim() == text.trim(),
            _ => false,
        })
    })
}

#[cfg(test)]
mod tests;
