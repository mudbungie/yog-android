//! The wire's **version preface** — the client half of the server's
//! `src/wire/hello.rs` (yog REMOTE §3, upstream bl-a670).
//!
//! REMOTE §3, verbatim: *"Every connection opens with a version preface: each
//! end writes one frame, `{"protocol": <integer>}`, before it reads the
//! peer's. Both write before either reads, so neither waits on the other and
//! there is no ordering rule to remember."*
//!
//! **The number itself is quoted from nowhere, deliberately.** REMOTE §3's
//! prose named a version and went stale the day the wire moved; the standing
//! value is the server's repo-root `PROTOCOL` file and nothing else, so
//! [`PROTOCOL`] below is this repository's own such file compiled in
//! (`build.rs`, bl-6fec) and this paragraph states no integer for a second
//! reader to trust. One fact, one home — and a file-shaped one, because the
//! release gates that read it are other repositories fetching one path out of
//! a tree they do not build.
//!
//! **The preface carries a second number since REMOTE §3.2** (bl-e598): the
//! `edition`, which is the ADDITIVE line inside a major. It is not matched and
//! it can never refuse a connection — a newer engine of this major is one this
//! build must go on talking to, and what the number buys is a control that can
//! grey itself rather than a reader that falls over (`crate::ledger`).
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

/// The number's ledger, and the generated constant it re-exports.
mod version;
pub use version::PROTOCOL;

/// The preface's two keys. `protocol` is the major and is matched exactly;
/// `edition` is the additive line inside it (REMOTE §3.2, `crate::ledger`) and
/// is never matched at all — it is read, kept, and spent by a control deciding
/// whether the engine could have said a field.
const KEY: &str = "protocol";
const EDITION_KEY: &str = "edition";

/// What a peer that stated no version is called in the sentence.
const UNSTATED: &str = "no version";

/// Write this build's preface. Called before this end reads anything, which is
/// what makes the exchange deadlock-free without an ordering rule to remember.
pub(crate) fn state(w: &mut dyn Write) -> io::Result<()> {
    let said = json!({ KEY: PROTOCOL, EDITION_KEY: crate::ledger::EDITION });
    frame::write_frame(w, said.to_string().as_bytes())
}

/// The version the peer stated, or `None` when it stated none — a frame that
/// never arrived, the terminator, bytes that are not JSON, a value that is not
/// an object and an object without the key collapsing to the one answer a
/// reader can act on.
fn stated(r: &mut dyn Read) -> Option<(u64, u64)> {
    let body = frame::read_frame(r).ok().flatten()?;
    let value: Value = serde_json::from_slice(&body).ok()?;
    let protocol = value.get(KEY)?.as_u64()?;
    // **An absent edition is the FLOOR, not a refusal** (REMOTE §3.2). Every
    // engine of this major that predates the field is exactly an engine at the
    // edition the major was cut at, so the absence says precisely that — and
    // an engine that never grows a field never has to state one.
    let edition = value
        .get(EDITION_KEY)
        .and_then(Value::as_u64)
        .unwrap_or_else(|| u64::from(crate::ledger::FLOOR));
    Some((protocol, edition))
}

/// Read the engine's preface, refuse a MAJOR mismatch to the caller, and hand
/// back the edition it stated — a capability fact, never a refusal, which the
/// held read keeps for whatever asks `crate::ledger::spells`.
pub(crate) fn confirm(r: &mut dyn Read) -> Result<u32, String> {
    let Some((protocol, edition)) = stated(r) else {
        return Err(mismatch(None));
    };
    if protocol != u64::from(PROTOCOL) {
        return Err(mismatch(Some(protocol)));
    }
    Ok(u32::try_from(edition).unwrap_or(u32::MAX))
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
