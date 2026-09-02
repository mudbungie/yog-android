//! **The follow lane, read one shot at a time** (REMOTE §5.5, bl-4822): the
//! accumulated tail of the answer being written right now.
//!
//! **A read starts holding nothing, so a re-ask replaces.** §5.5 is explicit
//! about both halves of that: *"the engine's reader is minted per held
//! connection and opens the response file at byte zero — so the **first**
//! frame of any read is the whole tail so far"*, and *"Two reads by the same
//! seat are two reads: the second starts holding nothing, so it replaces
//! rather than appending."* This seat holds no connection, so every read it
//! makes is a first frame and the fold it needs is assignment. The append
//! fold is the held connection's problem and this device does not have one
//! (DESIGN §7).
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
