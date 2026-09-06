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
//! **The grade is a closed vocabulary and an unknown one refuses.** REMOTE
//! §4.2 has two grades and the certificate is the authority for which was
//! minted; a third word on the wire is a build this one cannot talk to, and
//! reading it as either would be the silent misread §3's third rule forbids.

use serde_json::{Map, Value, json};

use super::Act;
use super::fields::str_of;
use crate::envelope::Envelope;
use crate::leaf::Grade;

/// Encode the mint.
pub(crate) fn encode(workspace: &str, name: &str, grade: Grade) -> Value {
    json!({ "op": "enroll", "workspace": workspace, "name": name,
            "grade": word(grade) })
}

/// Read one back.
pub(crate) fn decode(o: &Map<String, Value>) -> Result<Act, String> {
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

/// The §4.2 grade, in the engine's own two words.
fn grade(o: &Map<String, Value>) -> Result<Grade, String> {
    match str_of(o, "grade")?.as_str() {
        "foot" => Ok(Grade::Foot),
        "operator" => Ok(Grade::Operator),
        other => Err(format!("enroll: unknown grade {other:?}")),
    }
}

/// The same two words on the way out. One table, both directions.
pub(crate) fn word(grade: Grade) -> &'static str {
    match grade {
        Grade::Foot => "foot",
        Grade::Operator => "operator",
    }
}
