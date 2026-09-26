//! The roving pair through the envelope's three doors.

use crate::envelope::tests::minted;
use crate::envelope::{land, read, write};
use crate::test_support::scratch;

/// **The roving pair** (DESIGN §21.1): an engine that roves adds two hex
/// fields; the reader lands them under `rendezvous`'s names, the writer
/// carries them, and the material reads back as a pairing. One without the
/// other, or one that is not 32 bytes of hex, is refused before anything
/// lands — the half-provisioned store, caught at the envelope.
#[test]
fn the_roving_pair_rides_beside_the_six_and_lands_as_a_pairing() {
    let dir = scratch();
    let text = minted(&dir, "operator", "desk");
    let mut value: serde_json::Value = serde_json::from_str(&text).unwrap();
    let hex = |fill: u8| crate::rendezvous::hex(&[fill; 32]);
    value["rendezvous_pub"] = serde_json::Value::from(hex(1));
    value["pairing_salt"] = serde_json::Value::from(hex(2));
    let envelope = read(&value.to_string()).unwrap();
    assert_eq!(envelope.roving, Some((hex(1), hex(2))));
    let wire = dir.join("wire");
    land(&wire, &envelope).unwrap();
    let material = crate::material::read_dir(&wire).unwrap().unwrap();
    assert_eq!(
        material.pairing,
        Some(crate::rendezvous::Pairing {
            engine: [1u8; 32],
            salt: [2u8; 32],
        })
    );
    // The writer carries both, and the round trip is exact.
    assert_eq!(read(&write(&envelope)).unwrap(), envelope);

    value.as_object_mut().unwrap().remove("pairing_salt");
    let e = read(&value.to_string()).unwrap_err();
    assert!(e.contains("come together"), "{e}");
    value["pairing_salt"] = serde_json::Value::from("zz");
    let e = read(&value.to_string()).unwrap_err();
    assert_eq!(e, "field \"pairing_salt\" is not 32 bytes of hex");
    value["pairing_salt"] = serde_json::Value::from("");
    let e = read(&value.to_string()).unwrap_err();
    assert_eq!(e, "field \"pairing_salt\" is empty");
}
