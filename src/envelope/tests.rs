//! The envelope, over real minted material — the grade check is the design,
//! so it is proved against certificates rather than against a stated word.

use super::{Envelope, TAG, VERSION, land, read};
use crate::leaf::Grade;
use crate::test_support::{mint_ca, mint_foot, mint_leaf, scratch};
use std::path::Path;

/// The envelope a seat would render, built out of a minted leaf so the stated
/// grade and the certificate's own agree by construction.
fn minted(dir: &Path, grade: &str, name: &str) -> String {
    mint_ca(dir, "ca");
    if grade == "foot" {
        mint_foot(dir, "ca", name);
    } else {
        mint_leaf(dir, "ca", name, false);
    }
    envelope(dir, grade, &format!("notreal-{name}"), name)
}

fn envelope(dir: &Path, grade: &str, client: &str, leaf: &str) -> String {
    let read_file = |f: String| std::fs::read_to_string(dir.join(f)).unwrap();
    serde_json::to_string(&serde_json::json!({
        TAG: VERSION,
        "grade": grade,
        "name": client,
        "address": "engine.example.com:7737",
        "ca": read_file("ca.pem".to_owned()),
        "cert": read_file(format!("{leaf}.pem")),
        "key": read_file(format!("{leaf}.key")),
    }))
    .unwrap()
}

/// The whole path in one test: a pasted envelope reads, lands under the names
/// the reader wants, and the standing derived from what landed is the
/// component the operator minted — no stored choice anywhere in it.
#[test]
fn a_pasted_envelope_lands_material_the_reader_reads_back() {
    let dir = scratch();
    let text = minted(&dir, "operator", "desk");
    let envelope = read(&text).unwrap();
    assert_eq!(envelope.grade, Grade::Operator);
    assert_eq!(envelope.name, "notreal-desk");

    let wire = dir.join("wire");
    land(&wire, &envelope).unwrap();
    for file in crate::material::WANTED {
        assert!(wire.join(file).is_file(), "{file}");
    }
    let crate::bootstrap::Standing::Enrolled(enrolled) = crate::bootstrap::standing(&wire).unwrap()
    else {
        panic!("not enrolled");
    };
    assert_eq!(enrolled.component, crate::bootstrap::Component::Seat);
    assert_eq!(enrolled.client, "notreal-desk");
    assert_eq!(enrolled.material.address, "engine.example.com:7737");
}

/// A foot envelope enrolls a foot, and the component that comes up is read off
/// the certificate exactly as it is for a leaf delivered by cable — one path,
/// whichever channel carried the material (DESIGN §5, §9).
#[test]
fn a_foot_envelope_enrolls_a_foot() {
    let dir = scratch();
    let text = minted(&dir, "foot", "phone");
    let wire = dir.join("wire");
    land(&wire, &read(&text).unwrap()).unwrap();
    let crate::bootstrap::Standing::Enrolled(enrolled) = crate::bootstrap::standing(&wire).unwrap()
    else {
        panic!("not enrolled");
    };
    assert_eq!(enrolled.component, crate::bootstrap::Component::Foot);
}

/// **The certificate is the authority.** An envelope claiming operator grade
/// over a foot-grade leaf is a minting seat whose word and whose CA disagree,
/// and landing it would enroll this device as something its own leaf is not.
#[test]
fn a_stated_grade_that_the_certificate_contradicts_refuses() {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_foot(&dir, "ca", "phone");
    let text = envelope(&dir, "operator", "notreal-phone", "phone");
    let e = read(&text).unwrap_err();
    assert!(e.contains("operator"), "{e}");
    assert!(e.contains("foot"), "{e}");
    assert!(e.contains("REMOTE §4.2"), "{e}");
}

/// The same defect one field over: the engine's registry knows this device by
/// what the CA wrote, so an envelope naming something else is material minted
/// wrong rather than a rename.
#[test]
fn a_stated_name_that_the_certificate_contradicts_refuses() {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "desk", false);
    let text = envelope(&dir, "operator", "somebody-else", "desk");
    let e = read(&text).unwrap_err();
    assert!(e.contains("somebody-else"), "{e}");
    assert!(e.contains("notreal-desk"), "{e}");
}

/// Every way the text can fail to be this envelope, each refusing by name — a
/// camera reads whatever is in front of it, so "that is not one of these" has
/// to be a sentence rather than a silent nothing.
#[test]
fn text_that_is_not_this_envelope_refuses_by_name() {
    for (text, expected) in [
        ("not json at all", "not an enroll envelope"),
        ("[1,2,3]", "expected an object"),
        (r#"{"something":1}"#, "no \"yog-enroll\" field"),
        (r#"{"yog-enroll":"one"}"#, "must be a version number"),
        (r#"{"yog-enroll":2}"#, "version 2"),
    ] {
        let e = read(text).unwrap_err();
        assert!(e.contains(expected), "{text}: {e}");
    }
    // A newer minting seat is named in both directions, so an operator knows
    // which end to move.
    let e = read(r#"{"yog-enroll":2}"#).unwrap_err();
    assert!(e.contains("version 1"), "{e}");
}

/// A field missing, mistyped, empty, or carrying a grade nobody mints. Empty
/// is refused rather than landed: four present-but-blank files are the one
/// half-provisioned shape `material::read_dir` cannot name.
#[test]
fn a_field_missing_mistyped_or_empty_refuses_by_name() {
    let dir = scratch();
    let text = minted(&dir, "operator", "desk");
    let mut obj: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&text).unwrap();

    for key in ["grade", "name", "address", "ca", "cert", "key"] {
        let mut without = obj.clone();
        without.remove(key);
        let e = read(&serde_json::to_string(&without).unwrap()).unwrap_err();
        assert!(e.contains(key), "{key}: {e}");

        let mut blank = obj.clone();
        blank.insert(key.to_owned(), serde_json::Value::String(String::new()));
        let e = read(&serde_json::to_string(&blank).unwrap()).unwrap_err();
        assert!(e.contains(key), "{key} blank: {e}");

        let mut typed = obj.clone();
        typed.insert(key.to_owned(), serde_json::Value::from(7));
        let e = read(&serde_json::to_string(&typed).unwrap()).unwrap_err();
        assert!(e.contains(key), "{key} mistyped: {e}");
    }

    obj.insert("grade".to_owned(), serde_json::Value::from("admiral"));
    let e = read(&serde_json::to_string(&obj).unwrap()).unwrap_err();
    assert!(e.contains("admiral"), "{e}");
}

/// A cert field holding something that is not a certificate: every field is
/// present and every one is a string, so this is not the missing-field case
/// and the grade check has nothing to read.
#[test]
fn a_cert_field_that_holds_no_certificate_refuses() {
    let dir = scratch();
    let text = minted(&dir, "operator", "desk");
    let mut obj: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&text).unwrap();
    obj.insert(
        "cert".to_owned(),
        serde_json::Value::from("not a certificate"),
    );
    let e = read(&serde_json::to_string(&obj).unwrap()).unwrap_err();
    assert!(e.contains("no certificate"), "{e}");
}

/// A QR payload minted compact may carry no trailing newline, and both the PEM
/// readers and the address reader want one. The landing supplies it rather
/// than leaving the store subtly unreadable.
#[test]
fn landing_supplies_the_trailing_newline_a_compact_payload_drops() {
    let dir = scratch();
    let text = minted(&dir, "operator", "desk");
    let envelope = Envelope {
        address: "engine.example.com:7737".to_owned(),
        ..read(&text).unwrap()
    };
    let wire = dir.join("wire");
    land(&wire, &envelope).unwrap();
    let address = std::fs::read_to_string(wire.join(crate::material::ADDRESS)).unwrap();
    assert_eq!(address, "engine.example.com:7737\n");
    assert!(crate::bootstrap::standing(&wire).is_ok());
}

/// A directory that cannot be written is named, not swallowed: a partial
/// landing is the half-provisioned store, reported against an operator who did
/// nothing wrong.
#[test]
fn a_directory_that_will_not_take_the_material_is_named() {
    let dir = scratch();
    let text = minted(&dir, "operator", "desk");
    let envelope = read(&text).unwrap();
    // A regular file where the directory must go: `create_dir_all` refuses.
    let blocked = dir.join("blocked");
    std::fs::write(&blocked, "in the way").unwrap();
    let e = land(&blocked, &envelope).unwrap_err();
    assert!(e.contains("blocked"), "{e}");
}
