//! **The mint** (REMOTE §8.4, DESIGN §13.18): the act an operator-grade seat
//! fires to enroll the NEXT device, and the six fields it is answered with.
//!
//! **This device is on both sides of §8.4 now, and the two are not symmetric.**
//! `crate::envelope` reads a payload a camera saw; this reads an answer the
//! engine gave over mTLS. The scanning half checks the stated grade and name
//! against the leaf the same payload carries, because a photograph has no
//! provenance; this half does not, because the channel already settled it.
//! What the two share is the VALUE — one `Envelope`, one home — so what the
//! mint displays and what the next device reads are the same six fields said
//! once.
//!
//! **A third grade word is carried, not guessed at and not refused** (REMOTE
//! §3.2). §4.2 has two grades and the CERTIFICATE is the authority for which
//! was minted, so reading an unknown word as either would be the silent
//! misread §3's third rule forbids — but refusing it is not this reader's
//! call either. It rides as `Grade::Unknown`, and the check that matters fires
//! where it always did: `crate::envelope::agrees` holds the stated grade
//! against the leaf, and no unknown word can equal a leaf's own.

use serde_json::{Map, Value, json};

use super::Act;
use super::fields::str_of;
use crate::envelope::Envelope;
use crate::leaf::Grade;

/// Encode the mint.
pub(crate) fn encode(workspace: &str, name: &str, grade: Grade) -> Value {
    json!({ "op": "enroll", "workspace": workspace, "name": name,
            "grade": word(&grade) })
}

/// Read one back.
pub(crate) fn decode(o: &Map<String, Value>) -> Result<Act, String> {
    unrouted(o.get("address"))?;
    Ok(Act::Enroll {
        workspace: str_of(o, "workspace")?,
        name: str_of(o, "name")?,
        grade: grade(o)?,
    })
}

/// Read the `enrolled` answer: §8.4's six fields, as the value the scanning
/// half already has a type for.
pub(super) fn enrolled(o: &Map<String, Value>) -> Result<Envelope, String> {
    Ok(Envelope {
        grade: grade(o)?,
        name: str_of(o, "name")?,
        address: str_of(o, "address")?,
        ca: str_of(o, "ca")?,
        cert: str_of(o, "cert")?,
        key: str_of(o, "key")?,
    })
}

/// **The route the enrolled device will dial is a fact about THAT device**
/// (REMOTE §8.4, PROTOCOL 14), and this seat has no field to state one in. The
/// engine takes an optional `address` for a device that does not share its own
/// view of itself — an emulator reaching its host by the emulator's alias, a
/// phone on the LAN — and absent is the engine's own `wire/address`, which is
/// the only spelling this seat can honestly send: a phone minting for the next
/// box knows nothing about the route that box will take. So a frame stating
/// one is refused by name rather than read as the mint without it, which is
/// the silent misread REMOTE §3's third rule forbids.
fn unrouted(address: Option<&Value>) -> Result<(), String> {
    match address {
        None => Ok(()),
        Some(stated) => Err(format!("enroll: unimplemented address {stated}")),
    }
}

/// The §4.2 grade, in the engine's own two words, or the catch-all.
fn grade(o: &Map<String, Value>) -> Result<Grade, String> {
    Ok(match str_of(o, "grade")?.as_str() {
        "foot" => Grade::Foot,
        "operator" => Grade::Operator,
        other => Grade::Unknown(other.to_owned()),
    })
}

/// The same words on the way out. One table, both directions — an unknown one
/// round-trips as itself, which is what keeps a frame this build carried
/// byte-identical to the frame it read.
pub(crate) fn word(grade: &Grade) -> String {
    match grade {
        Grade::Foot => "foot".to_owned(),
        Grade::Operator => "operator".to_owned(),
        Grade::Unknown(word) => word.clone(),
    }
}
