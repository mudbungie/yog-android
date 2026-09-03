//! **The outbox's one rule** (bl-66fb): has the message this seat just
//! deposited come back in a transcript read?
//!
//! The local echo's PAINT is the shell's — muted ink, a rule under it, where
//! the row will be — but *what state it is in* is a fact about the engine's
//! answers, which is exactly the kind of reading that belongs under the
//! coverage floor rather than in a paint file. So the echo itself lives here
//! (bl-07b1 moved the rest of it down): the shell holds one and paints it, and
//! every decision about it is made in this file and proven.
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
use crate::seat::Snapshot;

/// How far back a delivered row may be and still be this echo's. A deposit
/// lands at the end of a transcript; anything older is another message that
/// happened to say the same thing.
const TAIL: usize = 4;

/// **The three states a sent message can be in**, and the ink each earns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fate {
    /// Written to the wire, nothing back yet.
    Sent,
    /// The engine's receipt came back and said yes.
    Landed,
    /// **The reply was lost** (yog REMOTE §3): the engine may have taken this
    /// message and may not have, and no gesture this end can send will say
    /// which. It stays on the glass, saying so, until a transcript read
    /// settles it — the read IS the recovery, and it is one this seat makes
    /// every cadence without being asked.
    InDoubt,
}

/// What became of an echo when a snapshot was read against it.
pub enum Settled {
    /// Still an echo, in the state it now holds.
    Standing(Echo),
    /// Gone: it became a transcript row, or the operator left the
    /// conversation it belonged to.
    Gone,
    /// Gone, and its text goes back to the composer — the engine said no, so
    /// saying it again is an ordinary first attempt rather than a repeat.
    Draft(String),
}

/// **The message this seat has sent and the engine has not shown back yet.**
/// It paints the instant it is sent, because that is the whole point: the
/// round trip is seconds and a chat app that shows nothing for seconds is a
/// chat app you press twice.
///
/// What it cannot know by itself is what the engine did with it, so it
/// remembers the seat's deposit counters at the moment it was sent and
/// watches for one of them to move.
pub struct Echo {
    /// What was sent — the text, so it can be given back if the deposit is
    /// refused, and matched against the transcript when it comes home.
    pub text: String,
    /// The conversation it was sent to. An echo is a message in ONE
    /// conversation, and painting it under another would be the same wrong
    /// claim every other pairing in this app refuses.
    pub agent: Option<String>,
    /// How it stands.
    pub fate: Fate,
    /// The deposit counters as they stood when this was sent.
    at: (usize, usize, usize),
}

impl Echo {
    /// One message, just handed to the model.
    pub fn sent(text: String, snap: &Snapshot) -> Self {
        Self {
            text,
            agent: snap.focus.agent.clone(),
            fate: Fate::Sent,
            at: (snap.landed, snap.refused, snap.doubted),
        }
    }

    /// **What became of it**, read against one snapshot — run once per frame,
    /// whatever screen is up.
    ///
    /// The order is the order of certainty. A conversation left behind takes
    /// its echo with it. A refusal is the engine's own no, so the text goes
    /// back to the composer and the banner already carries the sentence. A
    /// **lost reply is neither** (yog REMOTE §3, bl-07b1): handing the draft
    /// back there would put the operator one tap from a second copy of a
    /// message the engine may have taken, so the echo stands and says it is in
    /// doubt. And a row that appears in the transcript dissolves the echo
    /// whatever state it was in — which is how a doubted message resolves,
    /// with no gesture and nothing remembered.
    pub fn settle(mut self, snap: &Snapshot) -> Settled {
        if self.agent != snap.focus.agent {
            return Settled::Gone;
        }
        if snap.refused > self.at.1 {
            return Settled::Draft(self.text);
        }
        if snap.landed > self.at.0 {
            self.fate = Fate::Landed;
        } else if snap.doubted > self.at.2 {
            self.fate = Fate::InDoubt;
        }
        if taken(&snap.transcript, &self.text) {
            return Settled::Gone;
        }
        Settled::Standing(self)
    }
}

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
