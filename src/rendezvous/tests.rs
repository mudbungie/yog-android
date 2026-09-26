//! The two files, their three states, and the derivations both ends must
//! agree on — pinned as bytes, so a drift on either end fails a literal.

use super::*;
use crate::test_support::scratch;

fn pairing() -> Pairing {
    Pairing {
        engine: [1u8; 32],
        salt: [2u8; 32],
    }
}

#[test]
fn nothing_provisioned_is_none_and_both_files_read_back() {
    let dir = scratch();
    assert_eq!(read_dir(&dir).unwrap(), None);
    std::fs::write(dir.join(KEY), format!("{}\n", hex(&[1u8; 32]))).unwrap();
    std::fs::write(dir.join(SALT), hex(&[2u8; 32])).unwrap();
    assert_eq!(read_dir(&dir).unwrap(), Some(pairing()));
    assert_eq!(ROVING, [KEY, SALT]);
}

#[test]
fn half_the_material_is_a_refusal_naming_both_files() {
    let dir = scratch();
    std::fs::write(dir.join(SALT), hex(&[2u8; 32])).unwrap();
    let refusal = read_dir(&dir).unwrap_err();
    assert!(refusal.contains(KEY) && refusal.contains(SALT), "{refusal}");
    assert!(refusal.contains("half-provisioned"), "{refusal}");
}

#[test]
fn a_file_that_is_not_32_bytes_of_hex_is_no_material() {
    let dir = scratch();
    std::fs::write(dir.join(KEY), "zz\n").unwrap();
    std::fs::write(dir.join(SALT), "abc\n").unwrap();
    assert_eq!(read_dir(&dir).unwrap(), None);
    std::fs::write(dir.join(SALT), format!("{}\n", hex(&[7u8; 31]))).unwrap();
    assert_eq!(read_dir(&dir).unwrap(), None);
}

#[test]
fn hex_round_trips_and_refuses_what_is_not_hex() {
    assert_eq!(hex(&[0, 15, 255]), "000fff");
    assert_eq!(unhex("000fff"), Some(vec![0, 15, 255]));
    assert_eq!(unhex("0"), None, "odd length");
    assert_eq!(unhex("0g"), None, "not a digit");
    assert_eq!(unhex("é0"), None, "not even ASCII");
}

/// **The engine's derivations, byte for byte.** These literals were produced
/// by the same HKDF the engine's `material.rs` runs over the same salt and
/// labels; the presence salt is where its presence is filed, the inbox key
/// is the public half of the keypair its poll expects the call signed under.
#[test]
fn every_derivation_is_the_engines() {
    let p = pairing();
    assert_eq!(hex(&p.presence_salt()), PRESENCE_SALT);
    assert_eq!(hex(&p.inbox_salt()), INBOX_SALT);
    assert_eq!(hex(&p.seal_key()), SEAL_KEY);
    assert_eq!(hex(&p.inbox_keypair().unwrap().public()), INBOX_PUBLIC);
    let other = Pairing {
        engine: [1u8; 32],
        salt: [3u8; 32],
    };
    assert_ne!(other.seal_key(), p.seal_key(), "another salt, another key");
    assert_ne!(
        p.inbox_keypair().unwrap().public(),
        p.engine,
        "the inbox key is derived from the salt, never the engine's own"
    );
}

const PRESENCE_SALT: &str = "832100dd41d9219596850cbb12bb86caabe0d7ebcac788f3dc829b7a2093e55e";
const INBOX_SALT: &str = "5fc17259eeb89d3f93aa22916cecff7a171da3f998f836fb2aaa993b83654b38";
const SEAL_KEY: &str = "c3481ff89e54257cc554538285910e3ab328eccd206cad9a2ad0ed7ddf8b748d";
const INBOX_PUBLIC: &str = "9571b09392c75e738b1829605d29401eaa38adfb63071c7e53259217c7564e58";
