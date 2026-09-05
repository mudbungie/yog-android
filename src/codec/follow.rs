//! **The follow lane's frame** (REMOTE §5.5): what landed on the answer
//! being written since the frame before it — and the fold that turns a
//! sequence of them back into the tail.
//!
//! **A frame is an append, and this seat holds the lane** (DESIGN §14.1,
//! bl-8e3c). §5.5's rule is one line — *"Absorb every frame of a read, in
//! order, onto an empty fold. What you hold after the last frame you have
//! received is what you paint"* — and [`Stream::absorb`] is that fold, the
//! engine's own operation copied so that an engine frame and this seat's
//! accumulation agree by contract: `fold(a).absorb(fold(b)) == fold(a ++ b)`
//! on any line boundary. A read starts holding nothing, so the first frame
//! is the whole tail so far and a lane that re-asks is whole again with
//! nothing to reconcile.
//!
//! **Every field is optional, including all of them.** The corpus's own
//! first frame is `{"stream": {}}` — an answer that has begun and said
//! nothing yet — and `delta` names the kind of the last content event rather
//! than the shape of the frame. It rides as the token the engine wrote: this
//! seat paints the text, and typing a token it does not spend would be
//! inventing a vocabulary (the `alignment` narrowing's own reasoning).

use serde_json::{Map, Value};

use super::fields::{opt, str_of};

/// The answer in flight, as much of it as has landed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stream {
    /// The kind of the last content event — the engine's token, untyped here.
    pub delta: Option<String>,
    /// The answer so far.
    pub text: Option<String>,
    /// The reasoning so far, where the model states one.
    pub thinking: Option<String>,
}

impl Stream {
    /// Whether anything has landed. An answer that has begun and said nothing
    /// is not something to paint a row for.
    pub fn is_empty(&self) -> bool {
        self.text.is_none() && self.thinking.is_none()
    }

    /// Absorb the frame that landed **after** this one's bytes (REMOTE §5.5).
    /// Text accretes in stream order and the newer delta kind wins when the
    /// suffix had one at all. Absent stays absent — a stream that has said
    /// nothing has said nothing, and an empty `Some("")` would read as *it
    /// spoke* to the row projection downstream.
    pub fn absorb(&mut self, later: Self) {
        append(&mut self.text, later.text);
        append(&mut self.thinking, later.thinking);
        self.delta = later.delta.or(self.delta.take());
    }
}

fn append(slot: &mut Option<String>, more: Option<String>) {
    if let Some(more) = more {
        slot.get_or_insert_default().push_str(&more);
    }
}

/// The `stream` object of one follow frame.
pub(crate) fn stream_of(o: &Map<String, Value>) -> Result<Stream, String> {
    let held = o
        .get("stream")
        .ok_or("follow: missing field \"stream\"")?
        .as_object()
        .ok_or("follow: non-object field \"stream\"")?;
    Ok(Stream {
        delta: opt(held, "delta", str_of)?,
        text: opt(held, "text", str_of)?,
        thinking: opt(held, "thinking", str_of)?,
    })
}

#[cfg(test)]
mod tests;
