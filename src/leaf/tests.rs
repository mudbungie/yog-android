//! The DER walk, over certificates the same `openssl` recipe mints that an
//! operator's own provisioning uses — a hand-built byte fixture would prove
//! this walk agrees with whoever wrote the fixture.

use super::{Grade, common_name, grade};
use crate::test_support::{mint_ca, mint_foot, mint_leaf, scratch};
use rustls::pki_types::CertificateDer;
use rustls::pki_types::pem::PemObject;
use std::path::Path;

/// One minted leaf's DER.
fn der(dir: &Path, name: &str) -> Vec<u8> {
    CertificateDer::pem_file_iter(dir.join(format!("{name}.pem")))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .as_ref()
        .to_vec()
}

/// The subject's name, and **not the issuer's** — the trap the structural walk
/// exists for. The CA's own common name comes first in the bytes, so a scan
/// for the object identifier would answer `notreal-ca` for every client.
#[test]
fn the_name_is_the_subjects_and_never_the_issuers() {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "client", false);
    assert_eq!(
        common_name(&der(&dir, "client")),
        Some("notreal-client".to_owned())
    );
    // The CA signs itself, so its own subject is its own name — the same walk
    // over a self-issued certificate answers one name, not the leaf's.
    assert_eq!(common_name(&der(&dir, "ca")), Some("notreal-ca".to_owned()));
}

/// REMOTE §4.2: *"`CN=<client>, OU=foot` is a foot, and a subject with no
/// `OU=foot` is operator grade."*
#[test]
fn the_organizational_unit_is_where_the_grade_lives() {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "seat", false);
    mint_foot(&dir, "ca", "phone");
    assert_eq!(grade(&der(&dir, "seat")), Grade::Operator);
    assert_eq!(grade(&der(&dir, "phone")), Grade::Foot);
    // The grade rides beside the name; it does not replace it.
    assert_eq!(
        common_name(&der(&dir, "phone")),
        Some("notreal-phone".to_owned())
    );
}

/// **Default-operator, made total.** §4.2: *"a silently demoted seat would be
/// an outage with no sentence attached, while a silently promoted foot cannot
/// happen, because promotion requires the operator's CA to have written the
/// word."* So every way of failing to read a subject answers operator grade,
/// and only the written word answers foot.
#[test]
fn everything_unreadable_is_operator_grade_and_carries_no_name() {
    let nonsense: [&[u8]; 6] = [
        // Nothing at all, and a tag with no length byte.
        &[],
        &[0x30],
        // A length promising more bytes than are present.
        &[0x30, 0x7f, 0x00],
        // DER's forbidden indefinite length form, which BER allows.
        &[0x30, 0x80, 0x00, 0x00],
        // A long-form length wider than this walk will serve.
        &[0x30, 0x85, 0x01, 0x01, 0x01, 0x01, 0x01],
        // A well-formed SEQUENCE that is not a certificate: no inner TBS.
        &[0x30, 0x03, 0x02, 0x01, 0x07],
    ];
    for bytes in nonsense {
        assert_eq!(grade(bytes), Grade::Operator, "{bytes:?}");
        assert_eq!(common_name(bytes), None, "{bytes:?}");
    }
}

/// A subject that carries organizational units, none of which says foot. This
/// is the arm between "no units at all" and "the word is there" — and the one
/// that would silently promote if the walk matched loosely.
#[test]
fn an_organizational_unit_that_is_not_foot_is_still_a_seat() {
    let dir = scratch();
    mint_ca(&dir, "ca");
    crate::test_support::mint_unit(&dir, "ca", "desk", "footwear");
    assert_eq!(grade(&der(&dir, "desk")), Grade::Operator);
}
