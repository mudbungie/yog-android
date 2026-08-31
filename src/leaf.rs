//! **The grade and the name this device's own leaf carries** (yog REMOTE §4.2,
//! upstream bl-1dd3) — the client-side half of the server's
//! `registry/leaf.rs`, over the one certificate this app holds.
//!
//! §4.2: *"The grade is on the leaf, not in a registration and not in a
//! config. It is issued out of channel with the certificate, by the
//! operator's own CA, on the same act §1.4 already requires."* And the
//! spelling: *"`CN=<client>, OU=foot` is a foot, and a subject with no
//! `OU=foot` is operator grade."*
//!
//! **This reads the grade; it does not enforce it.** Enforcement is one raise
//! at the engine's chokepoint, in band, naming the grade — nothing here can
//! grant this device anything. What reading it buys is that the app knows
//! **which component it is** before it starts one: a phone on a foot-grade
//! leaf that ran the seat's standing-question loop would earn a refusal per
//! question, forever, and the operator would read a wall of sentences where a
//! component boundary belongs.
//!
//! **Default-operator, not default-foot**, which is why [`grade`] answers a
//! [`Grade`] rather than an `Option`: §4.2 is explicit that *"a certificate
//! minted before this existed, or by a recipe that has not learned the flag,
//! must keep working exactly as it did — a silently demoted seat would be an
//! outage with no sentence attached, while a silently promoted foot cannot
//! happen, because promotion requires the operator's CA to have written the
//! word."*
//!
//! **This crate links no certificate library** (AGENTS.md rule 6), so it is a
//! DER walk. Structural rather than a byte search, and the structure is the
//! point: the **issuer** carries a common name too and comes FIRST, so a scan
//! for the common-name object identifier would answer the CA's name for every
//! client on the box.
//!
//! What it reads, per RFC 5280:
//!
//! ```text
//! Certificate     ::= SEQUENCE { tbsCertificate, signatureAlgorithm, signature }
//! TBSCertificate  ::= SEQUENCE { [0] version OPTIONAL, serialNumber INTEGER,
//!                                signature, issuer, validity, subject, … }
//! Name            ::= SEQUENCE OF SET OF SEQUENCE { type OID, value ANY }
//! ```
//!
//! The optional `[0] version` is why `subject` is located **relative to the
//! serial number** rather than at a fixed index: the serial is the first field
//! certainly present, and `subject` is four constructed values past it, so a
//! version-1 certificate and a version-3 one take one path rather than two.

/// The two grades REMOTE §4.2 mints, and there are exactly two.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    /// The whole boundary, within the registrations §4 scopes.
    Operator,
    /// *"The tool-host gestures and nothing else: `advertise`, `invocations`
    /// and `complete`. No other `Query`, no other `Action`."*
    Foot,
}

/// The organizational unit that says foot. §4.2: it is *"presence-shaped …
/// precisely so there is no word to mistype into a demotion."*
const FOOT: &str = "foot";

/// DER tags this walk names.
const INTEGER: u8 = 0x02;
const OID: u8 = 0x06;

/// `id-at-commonName` — ASN.1 `{joint-iso-itu-t(2) ds(5) attributeType(4)
/// commonName(3)}`, in its DER encoding. Spelled as bytes rather than as the
/// dotted arc string because the dotted form of four small arcs is
/// indistinguishable from an IPv4 address, to a reader and to `make leak-scan`.
const COMMON_NAME: [u8; 3] = [0x55, 0x04, 0x03];

/// `id-at-organizationalUnitName` — the same arc one attribute over, and the
/// home §4.2 gives the grade. Spelled as bytes for [`COMMON_NAME`]'s reason.
const ORG_UNIT: [u8; 3] = [0x55, 0x04, 0x0b];

/// How many constructed fields separate `serialNumber` from `subject`:
/// signature, issuer, validity, subject.
const SERIAL_TO_SUBJECT: usize = 4;

/// This leaf's client identity — the subject common name, which REMOTE §2
/// rules **is** the client: *"One certificate = one client identity (its leaf
/// name)."*
///
/// The **last** common name wins: a distinguished name is written
/// most-general first in DER and most-specific last, so the final `CN` is the
/// leaf's own. A certificate minted by the one recipe has exactly one and the
/// question does not arise.
pub fn common_name(der: &[u8]) -> Option<String> {
    attributes(subject(der)?, COMMON_NAME).pop()
}

/// The grade the same subject carries. Bytes that are not a certificate, a
/// subject with other organizational units and a subject with none are one
/// answer — operator grade — because default-operator is made total here
/// rather than defaulted at each caller.
pub fn grade(der: &[u8]) -> Grade {
    let foot =
        subject(der).is_some_and(|name| attributes(name, ORG_UNIT).iter().any(|unit| unit == FOOT));
    if foot { Grade::Foot } else { Grade::Operator }
}

/// The `Name` bytes of the certificate's **subject** — located relative to the
/// serial number, for the reason the module doc gives.
fn subject(der: &[u8]) -> Option<&[u8]> {
    let (_, certificate, _) = tlv(der)?;
    let (_, tbs, _) = tlv(certificate)?;
    let fields = elements(tbs);
    let serial = fields.iter().position(|(tag, _)| *tag == INTEGER)?;
    let &(_, subject) = fields.get(serial + SERIAL_TO_SUBJECT)?;
    Some(subject)
}

/// Every value of attribute `oid` in a `Name`, in DER order, decoded as UTF-8.
/// Every string type these attributes are minted in — `UTF8String`,
/// `PrintableString`, `IA5String` — is UTF-8 or a subset of it, and one that is
/// not (`BMPString` is UTF-16) fails the decode and is skipped rather than
/// mis-read.
fn attributes(name: &[u8], oid: [u8; 3]) -> Vec<String> {
    let mut found = Vec::new();
    for (_, rdn) in elements(name) {
        for (_, attribute) in elements(rdn) {
            let parts = elements(attribute);
            let (Some(&(tag, kind)), Some(&(_, value))) = (parts.first(), parts.get(1)) else {
                continue;
            };
            if tag != OID || kind != oid {
                continue;
            }
            if let Ok(text) = std::str::from_utf8(value) {
                found.push(text.to_owned());
            }
        }
    }
    found
}

/// One DER type-length-value off the front of `bytes`: its tag, its contents,
/// and what follows it. `None` for a truncated header, a truncated value, or a
/// length DER does not permit — the indefinite form (`0x80`), which BER allows
/// and DER forbids, and a length wider than this walk will serve.
fn tlv(bytes: &[u8]) -> Option<(u8, &[u8], &[u8])> {
    let tag = *bytes.first()?;
    let first = *bytes.get(1)?;
    let (len, header) = if first < 0x80 {
        (usize::from(first), 2)
    } else {
        let width = usize::from(first & 0x7f);
        if width == 0 || width > 4 {
            return None;
        }
        let mut len: usize = 0;
        for i in 0..width {
            len = (len << 8) | usize::from(*bytes.get(2 + i)?);
        }
        (len, 2 + width)
    };
    // Saturating rather than checked: an unreachable overflow arm is an
    // untestable branch, and a saturated end simply fails the read below.
    let end = header.saturating_add(len);
    let value = bytes.get(header..end)?;
    Some((tag, value, bytes.get(end..).unwrap_or_default()))
}

/// Every element of a constructed value, in order. A trailing byte run that is
/// not a whole TLV ends the walk — a malformed tail yields the elements read
/// before it, which is what makes every read above total.
fn elements(mut body: &[u8]) -> Vec<(u8, &[u8])> {
    let mut out = Vec::new();
    while let Some((tag, value, rest)) = tlv(body) {
        out.push((tag, value));
        body = rest;
    }
    out
}

#[cfg(test)]
mod tests;
