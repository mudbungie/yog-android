//! **The roving pair** (DESIGN §21.1): the two optional envelope fields an
//! server that roves adds, split from the envelope because they are one
//! concern with three doors — read off a payload, written into one, landed
//! as files — and the envelope's six are another.

use super::field;
use serde_json::{Map, Value};

/// **The optional pair** (DESIGN §21.1, yog REMOTE §8.4 as amended by yog
/// bl-9043, edition 20): the server's rendezvous public key and the pairing
/// salt, hex, under the server's own spellings, carried only by a server
/// that roves — a loopback-only server sends neither. Both or neither — one
/// without the other is refused as the half-provisioned store it would land.
const ROVING: [&str; 2] = ["rendezvous_pub", "pairing_salt"];

/// The optional pair, both or neither, carried as the server spelled it.
/// **The hex is judged by [`checked`], not here** (REMOTE §3.2, corpus
/// edition 23): this reader also reads the `enrolled` reply off the wire,
/// where a word this build cannot spell is carried, never refused — and the
/// operator's own envelope, the one act that lands the pair, judges it
/// before anything is written.
pub(crate) fn read(obj: &Map<String, Value>) -> Result<Option<(String, String)>, String> {
    let [rendezvous, salt] = ROVING;
    match (obj.contains_key(rendezvous), obj.contains_key(salt)) {
        (false, false) => Ok(None),
        (true, true) => Ok(Some((field(obj, rendezvous)?, field(obj, salt)?))),
        _ => Err(format!(
            "fields {rendezvous:?} and {salt:?} come together; one alone is half an entry"
        )),
    }
}

/// The pair as envelope fields, for the writer: none, or both.
pub(super) fn fields(pair: Option<&(String, String)>) -> Vec<(&'static str, String)> {
    pair.map(|(rendezvous, salt)| {
        ROVING
            .into_iter()
            .zip([rendezvous.clone(), salt.clone()])
            .collect()
    })
    .unwrap_or_default()
}

/// **The pair as 32 bytes of hex each**, or the refusal naming the field —
/// asked of an envelope that is about to land, because the next act is
/// writing the files and `crate::rendezvous` reads exactly that shape.
pub(super) fn checked(pair: Option<&(String, String)>) -> Result<(), String> {
    let Some((rendezvous, salt)) = pair else {
        return Ok(());
    };
    for (key, hex) in ROVING.into_iter().zip([rendezvous, salt]) {
        if crate::rendezvous::unhex(hex).is_none_or(|bytes| bytes.len() != 32) {
            return Err(format!("field {key:?} is not 32 bytes of hex"));
        }
    }
    Ok(())
}

/// The pair as files beside the leaf, under the names `crate::rendezvous`
/// reads: none, or both.
pub(super) fn files(pair: Option<&(String, String)>) -> Vec<(&'static str, String)> {
    pair.map(|(rendezvous, salt)| {
        vec![
            (crate::rendezvous::KEY, rendezvous.clone()),
            (crate::rendezvous::SALT, salt.clone()),
        ]
    })
    .unwrap_or_default()
}

#[cfg(test)]
mod tests;
