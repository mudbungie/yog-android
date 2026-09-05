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
/// **13 since the re-vendor of bl-8e3c.** Twelve moves stand behind it, and
/// the last five are the ones this build consumes. 2 was the tool-host pair
/// (`subject_cwd` on an advertised element, `cwd` on an invocation — REMOTE
/// §5.1, §5.3); **3** put `failure` on the conversation row, the agent answer
/// and the queue row (§9.10); **4** put `flag` on the queue row (§9.11); **5**
/// rewrote `reply/governing` (§9.12); **6** minted the §9.4 tuning pair and
/// widened `reply/providers`; **7** put `surface` on every `reply/help` row —
/// the interface-parity classification `crate::parity` judges this seat by;
/// **8** put `wrote` on `reply/advertised`, the foot's own receipt
/// (`codec::reply`, DESIGN §6). `last_active_unix` rode in at 2 with §9.9.
///
/// **9** (§9.16) put the `wounded` entry on `reply/transcript` — the settled-
/// failure notice, read by `codec::transcript` and painted by `rows::wounded`
/// — and took `auth_failed` off `reply/steps`, a shape this seat refuses.
/// **10** (§14.1) moved **no field at all**: `attention` became follow-class,
/// the same ask and the same reply shape answered as a *sequence* by an intake
/// that can hold — precisely "what a spelling already in use is taken to
/// say", and the one class of move the corpus ledger cannot see, since frame
/// count is not a field signature. **11** (§9.17) put `failed`, `exit_label`
/// and `standing` on `reply/ops`, so the trail reads the engine's words rather
/// than classifying `exit` (`codec::trail`). **12** put `says` on every queue
/// row — the firing rules in words, one home on the engine (`codec::queue`).
/// **13** (§9.18) put a typed `settings` array on `reply/config`, a shape this
/// seat refuses; it cost the integer and nothing else.
///
/// **10 is the bump that was a design decision here, not a re-vendor.** The
/// wire intake this seat dials HOLDS a follow-class read: the first frame at
/// connect, a frame per change, a terminator when the hold ends — thirty
/// seconds, the follow lane's own bound. A one-shot read of a held lane
/// blocks for the whole hold, and the standing pass used to make exactly that
/// read of `attention` on every cycle and of `follow` at a 500 ms rest. So
/// since bl-8e3c this seat holds both lanes beside the pass (DESIGN §14.1):
/// `seat::lane` parks a reader on the connection and hands each frame to the
/// worker, which folds it — the §5.5 append fold for `follow`, replacement
/// for `attention` — and the pass never waits on either.
///
/// **A move in a shape this codec does not spell still moves this number,
/// and that is the point.** `governing`, `steps` and `config` are recorded
/// refusals here (each the answer to a gesture this seat does not send), so
/// 5, 9's loss and 13 cost this client nothing but the integer — and the
/// integer is the whole of what the handshake gates on. A seat that stayed at
/// 8 because "nothing it reads changed" would simply stop speaking to the
/// engine, which is what this ball was filed against.
///
/// **The version is the only thing that breaks an old seat, and it breaks it
/// on purpose.** An unknown FIELD is tolerated — this codec reads the fields
/// it spells and ignores the rest, which `codec::conv`'s own test pins. What
/// ends an old build is this preface: fail-closed, both ways, by §3's design.
pub const PROTOCOL: u32 = 13;

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
