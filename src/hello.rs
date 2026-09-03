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
//! value is the server's `src/wire/hello.rs` constant and nothing else, so
//! [`PROTOCOL`] below mirrors that constant and this paragraph states no
//! integer for a second reader to trust. One fact, one home.
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
///
/// **8 since the re-vendor of bl-cc54.** Seven moves stand behind it and the
/// last six are why an old build must not be left running: 2 was the
/// tool-host pair (`subject_cwd` on an advertised element, `cwd` on an
/// invocation — REMOTE §5.1, §5.3); **3** put `failure` on the conversation
/// row, the agent answer and the queue row (§9.10); **4** put `flag` on the
/// queue row (§9.11); **5** rewrote `reply/governing` — `branch` out,
/// `follows` and `diverged_lineages` in, and `oid` now naming the followed
/// lineage's head rather than the fork commit an agent's branch left (§9.12,
/// litany's follow-the-tip ruling reaching the wire); and **6** minted the
/// §9.4 tuning pair — `effort` and `priority`, two request shapes that are
/// new at 6 — and widened `reply/providers`, whose rows now state per
/// provider whether either can be asked for at all; and **7** put `surface`
/// on every `reply/help` row — the interface-parity classification (yog
/// `docs/PARITY.md` §2), two values, `control` for an op every seat owes a
/// discoverable interactable and `machine` for one spoken only by programs;
/// and **8** put `wrote` on `reply/advertised` — a required boolean saying
/// whether the engine changed the stored set or found it identical and
/// compared (§5.1, yog bl-66d4). `last_active_unix` rode in at 2 with §9.9.
///
/// **7 is the bump this client consumes without decoding a byte of it.** The
/// `help` reply is a recorded refusal here, so the field reaches this repo as
/// a fixture rather than as a codec change — and it is load-bearing anyway:
/// `crate::parity` reads the classification straight out of the vendored
/// corpus, which is why the roster of ops this seat owes a control is the
/// engine's own list rather than one kept here.
///
/// **8 is the one this device reads as a device.** The other six moved shapes
/// a seat paints; this one moved the foot's own receipt, and this box is a
/// foot — so `wrote` is decoded (`codec::reply`), carried out of
/// [`crate::foot::Foot::advertise`], and judged by the host loop, where a
/// `true` on a re-assertion is this machine learning its advertised set was
/// replaced while it was off running a tool (DESIGN §6). The seat half reads
/// nothing off it: a seat never advertises, so it never earns the receipt —
/// the desktop seat's own re-vendor reached the same conclusion.
///
/// **A move in a shape this codec does not spell still moves this number,
/// and that is the point.** `governing` is a recorded refusal here (it is
/// the answer to a gesture this seat does not send), so 5 cost this client
/// nothing but the integer — and the integer is the whole of what the
/// handshake gates on. A seat that stayed at 4 because "nothing it reads
/// changed" would simply stop speaking to the engine.
///
/// **The version is the only thing that breaks an old seat, and it breaks it
/// on purpose.** An unknown FIELD is tolerated — this codec reads the fields
/// it spells and ignores the rest, which `codec::conv`'s own test pins — so
/// none of those three shapes would have hurt a protocol-2 build. What ends
/// it is this preface: fail-closed, both ways, by §3's design. A seat that
/// does not follow the engine's number stops speaking to it, whatever it can
/// or cannot read.
///
/// One meaning moved with no signature to see it, and it is still true:
/// REMOTE §5.5's follow frame is an **append**, so a client that consumes
/// that lane must read the section and not only re-vendor the fixtures
/// (DESIGN §7 — this seat reads the lane one shot at a time, where the fold
/// is assignment).
pub const PROTOCOL: u32 = 8;

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
