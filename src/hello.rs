//! The wire's **version preface** — the client half of the server's
//! `src/wire/hello.rs` (yog REMOTE §3, upstream bl-a670).
//!
//! REMOTE §3, verbatim: *"Every connection opens with a version preface: each
//! end writes one frame, `{"protocol": <integer>}`, before it reads the
//! peer's. Both write before either reads, so neither waits on the other and
//! there is no ordering rule to remember. `PROTOCOL` is `1`."*
//!
//! Three properties this end must keep, each the refusal of something easier:
//!
//! - **Write before reading.** This seat writes its preface and its request in
//!   the same breath and confirms the engine's on the way to the answer, so
//!   the check costs no round trip and the only connection it stops is one
//!   that was going to be refused anyway.
//! - **No negotiation.** No version list, no capability probe, no compat shim:
//!   a mismatch is fail-closed and the sentence — which names *both* versions
//!   and the remedy — is the upgrade prompt. It arrives at the caller as the
//!   one `Err(String)` every other transport failure already arrives as.
//! - **The request frame is untouched.** The preface rides *beside* the
//!   gesture envelope, never inside it, so the frame this crate writes is byte
//!   for byte the frame the codec built and the codec gains no field.
//!
//! **A peer that states no version is refused exactly as a peer of the wrong
//! one** (REMOTE §3). An unversioned engine — a reply envelope where a preface
//! belongs — a frame that is not an object, an object without the key, and a
//! peer that hung up mid-preface are one case: none of them can be served, and
//! three sentences for one outcome is three sentences.

use std::io::{self, Read, Write};

use serde_json::{Value, json};

use crate::frame;

/// The protocol this build speaks. Mirrors the server's `wire::hello::PROTOCOL`
/// — one integer, and **a new verb is not a bump**: an unknown `op` or reply
/// `kind` already refuses in band naming it, which is the boundary correcting
/// itself rather than two protocols meeting. It moves when an *existing* shape
/// changes meaning: the framing, the envelope, or what a spelling already in
/// use is taken to say.
pub const PROTOCOL: u32 = 1;

/// The preface's one key, and the whole of its shape.
const KEY: &str = "protocol";

/// What a peer that stated no version is called in the sentence.
const UNSTATED: &str = "no version";

/// Write this build's preface. Called before this end reads anything, which is
/// what makes the exchange deadlock-free without an ordering rule to remember.
pub(crate) fn state(w: &mut dyn Write) -> io::Result<()> {
    frame::write_frame(w, json!({ KEY: PROTOCOL }).to_string().as_bytes())
}

/// The version the peer stated, or `None` when it stated none — a frame that
/// never arrived, the terminator, bytes that are not JSON, a value that is not
/// an object and an object without the key collapsing to the one answer a
/// reader can act on.
fn stated(r: &mut dyn Read) -> Option<u64> {
    let body = frame::read_frame(r).ok().flatten()?;
    let value: Value = serde_json::from_slice(&body).ok()?;
    value.get(KEY)?.as_u64()
}

/// Read the engine's preface and refuse a mismatch to the caller.
pub(crate) fn confirm(r: &mut dyn Read) -> Result<(), String> {
    let peer = stated(r);
    if peer == Some(u64::from(PROTOCOL)) {
        return Ok(());
    }
    Err(mismatch(peer))
}

/// The refusal, said the same way at both ends: both versions, and what to do
/// about it. It is the upgrade prompt, so it names a number an operator can act
/// on rather than a code — and it is the server's sentence word for word,
/// because one rule said two ways is two rules.
fn mismatch(peer: Option<u64>) -> String {
    let peer = peer.map_or_else(|| UNSTATED.to_owned(), |v| v.to_string());
    format!(
        "wire protocol mismatch: this end speaks version {PROTOCOL}, \
         the peer speaks {peer}. There is no negotiation — \
         upgrade the older component until both speak one version."
    )
}

#[cfg(test)]
mod tests;
