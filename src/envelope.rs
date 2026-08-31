//! **The enroll envelope**: the material a trusted seat mints, carried here
//! by eye (bl-dd7b, yog bl-f4e3).
//!
//! yog's `enroll` act answers a reply carrying `{grade, name, address, ca,
//! cert, key}` and shreds the leaf key server-side; the operator's seat renders
//! that as a QR, and this device reads it. The envelope is that payload with a
//! version tag in front:
//!
//! ```text
//! {"yog-enroll":1,"grade":…,"name":…,"address":…,"ca":…,"cert":…,"key":…}
//! ```
//!
//! **REMOTE §1.4 is untouched, and it is worth saying exactly why.** The new
//! device performs no channel act: an already-trusted operator-grade seat
//! performs the mint over *its* authenticated channel, and the material travels
//! out of channel — a screen, and an operator's own eyes. That is DESIGN §5's
//! third delivery channel, not a pairing protocol. This module never dials.
//!
//! **This is the QR's degraded path and it is the same sink.** A camera that
//! will not focus, a denied permission, an operator reading a laptop screen
//! into a text field: pasting the envelope must work regardless, so it is
//! built first and a decoder, when one is adjudicated (bl-d815), is only a
//! producer of the same string.
//!
//! **The grade is not taken on the envelope's word.** REMOTE §4.2 puts the
//! grade on the certificate and DESIGN §9 derives the component from it; an
//! envelope field that disagreed would be a second authority for one fact, and
//! landing it would enroll a device as something its own leaf is not. So the
//! stated grade must AGREE with the leaf, and a disagreement refuses naming
//! both.

use std::path::Path;

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

/// Read an envelope out of the text a scan or a paste produced.
///
/// **The version is checked first and by name.** A stranger's QR, a truncated
/// scan and a payload from a newer minting seat are three different sentences,
/// and each names what it saw — the same fail-closed shape `crate::hello`
/// gives the wire preface, one channel over.
pub fn read(text: &str) -> Result<Envelope, String> {
    let value: Value = serde_json::from_str(text.trim())
        .map_err(|e| format!("not an enroll envelope: {e}; expected JSON with a {TAG:?} field"))?;
    let obj = value.as_object().ok_or_else(|| {
        format!("not an enroll envelope: expected an object with a {TAG:?} field")
    })?;
    let stated = obj
        .get(TAG)
        .ok_or_else(|| format!("not an enroll envelope: no {TAG:?} field"))?
        .as_u64()
        .ok_or_else(|| format!("{TAG:?} must be a version number"))?;
    if stated != VERSION {
        return Err(format!(
            "enroll envelope version {stated}; this build reads version {VERSION}"
        ));
    }
    let grade = match field(obj, "grade")?.as_str() {
        "foot" => Grade::Foot,
        "operator" => Grade::Operator,
        other => {
            return Err(format!(
                "unknown grade {other:?}; expected foot or operator"
            ));
        }
    };
    let envelope = Envelope {
        grade,
        name: field(obj, "name")?,
        address: field(obj, "address")?,
        ca: field(obj, "ca")?,
        cert: field(obj, "cert")?,
        key: field(obj, "key")?,
    };
    agrees(&envelope)?;
    Ok(envelope)
}

/// **The envelope's grade against the leaf's own.** The certificate is the
/// authority (REMOTE §4.2) and this is the only thing the stated grade is good
/// for: catching a minting seat whose word and whose CA disagree, at the one
/// moment the material can still be refused.
fn agrees(envelope: &Envelope) -> Result<(), String> {
    let der = certificate(&envelope.cert)?;
    let carried = crate::leaf::grade(&der);
    if carried != envelope.grade {
        return Err(format!(
            "envelope says {} but the certificate is {}; the certificate is the authority \
             (REMOTE §4.2), so this material was minted wrong",
            word(envelope.grade),
            word(carried)
        ));
    }
    // The name is the leaf's too, and a mismatch is the same defect one field
    // over — the engine's registry knows this device by what the CA wrote.
    let carried = crate::leaf::common_name(&der).unwrap_or_default();
    if carried != envelope.name {
        return Err(format!(
            "envelope names {:?} but the certificate names {carried:?}",
            envelope.name
        ));
    }
    Ok(())
}

/// The leaf's DER, out of the envelope's PEM. **The first certificate is the
/// leaf**, for `crate::bootstrap`'s reason: a chain is written end-entity
/// first, so any other one would answer the issuing CA's grade.
fn certificate(pem: &str) -> Result<Vec<u8>, String> {
    use rustls::pki_types::CertificateDer;
    use rustls::pki_types::pem::PemObject;
    let leaf = CertificateDer::pem_slice_iter(pem.as_bytes())
        .next()
        .ok_or_else(|| "the envelope's cert holds no certificate".to_owned())?
        .map_err(|e| format!("the envelope's cert will not read: {e}"))?;
    Ok(leaf.as_ref().to_vec())
}

/// The word a grade wears in an envelope and in a refusal.
fn word(grade: Grade) -> String {
    match grade {
        Grade::Foot => "foot",
        Grade::Operator => "operator",
    }
    .to_owned()
}

/// A required, non-empty string field. Empty is refused rather than landed: a
/// blank address or a blank certificate is a half-provisioned store with every
/// file present, which is the one shape `crate::material` cannot name.
fn field(obj: &Map<String, Value>, key: &str) -> Result<String, String> {
    let text = obj
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing or non-string field {key:?}"))?;
    if text.is_empty() {
        return Err(format!("field {key:?} is empty"));
    }
    Ok(text.to_owned())
}

/// Write the envelope into this device's material directory, under the names
/// [`crate::material`] reads.
///
/// The file list is the reader's own ([`crate::material::WANTED`]) rather than
/// a second copy: a fifth required file must not be readable-but-unwritten.
/// Every write is checked, because a partial landing is exactly the
/// half-provisioned store `read_dir` exists to name — and it would be named
/// against an operator who did nothing wrong.
pub fn land(dir: &Path, envelope: &Envelope) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let written = [
        (crate::material::ANCHORS, envelope.ca.clone()),
        (crate::material::CHAIN, envelope.cert.clone()),
        (crate::material::KEY, envelope.key.clone()),
        (crate::material::ADDRESS, envelope.address.clone()),
    ];
    for (name, body) in &written {
        let path = dir.join(name);
        std::fs::write(&path, ends_in_newline(body))
            .map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(())
}

/// PEM readers and the address reader both want a trailing newline, and a QR
/// payload minted compact may carry none.
fn ends_in_newline(body: &str) -> String {
    if body.ends_with('\n') {
        body.to_owned()
    } else {
        format!("{body}\n")
    }
}

#[cfg(test)]
mod tests;
