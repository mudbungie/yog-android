//! **The enroll envelope, written** — what is left of it (bl-dd7b, yog
//! bl-f4e3) until the minting seat's `enroll` gives way to `offer` (yog
//! REMOTE §15.4).
//!
//! yog's `enroll` act answers `{grade, name, address, ca, cert, key}`, and an
//! operator-grade seat renders that as a QR for another device to read:
//!
//! ```text
//! {"yog-enroll":1,"grade":…,"name":…,"address":…,"ca":…,"cert":…,"key":…}
//! ```
//!
//! **This device no longer READS one.** The envelope put a private key in a
//! symbol and on a clipboard; a copy of it was the device. A device now
//! registers itself by an offer, with a key it makes (`crate::register`,
//! DESIGN §5), and the reader, the landing and the paste screen's hint are
//! deleted. [`write`] stands only because the admin surface still shows what
//! `enroll` answers (DESIGN §13.18), and goes with it.

use serde_json::{Map, Value};

use crate::leaf::Grade;

/// The tag that both names this envelope and states its version — one field,
/// because a payload that carried a version but no name would be read out of
/// any JSON a camera happened to see.
pub const TAG: &str = "yog-enroll";

/// The envelope version this build speaks.
pub const VERSION: u64 = 1;

/// One minted enrollment, read and not yet landed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    /// The grade the minting seat says it issued. Checked against the leaf,
    /// never trusted over it.
    pub grade: Grade,
    /// The client common name, which REMOTE §2 rules **is** the client.
    pub name: String,
    /// The `host:port` this device will dial.
    pub address: String,
    /// The operator CA, PEM.
    pub ca: String,
    /// This device's leaf, PEM.
    pub cert: String,
    /// This device's private key, PEM.
    pub key: String,
}

/// **Write one out** — the compact JSON REMOTE §8.4 rules, tag first in
/// meaning if not in byte order: this is a JSON object and the reader is a
/// JSON reader, so which key lands where is not part of the contract.
///
/// It is the minting seat's half (DESIGN §13.18). It does not check the
/// certificate: the answer arrived over mTLS from the server that minted it.
#[must_use]
pub fn write(envelope: &Envelope) -> String {
    let mut map = Map::new();
    map.insert(TAG.to_owned(), Value::from(VERSION));
    map.insert("grade".to_owned(), Value::from(word(&envelope.grade)));
    for (key, said) in [
        ("name", &envelope.name),
        ("address", &envelope.address),
        ("ca", &envelope.ca),
        ("cert", &envelope.cert),
        ("key", &envelope.key),
    ] {
        map.insert(key.to_owned(), Value::from(said.clone()));
    }
    Value::Object(map).to_string()
}

/// The word a grade wears in an envelope and in a refusal.
fn word(grade: &Grade) -> String {
    match grade {
        Grade::Foot => "foot".to_owned(),
        Grade::Operator => "operator".to_owned(),
        // A word this build cannot spell is said back as the server said it.
        Grade::Unknown(word) => word.clone(),
    }
}

#[cfg(test)]
mod tests;
