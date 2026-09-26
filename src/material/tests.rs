//! The three answers, and only the three: absence is the wire off, a partial
//! store names every gap at once, and presence hands back the four facts.

use super::{ADDRESS, ANCHORS, CHAIN, KEY, read_dir};
use crate::test_support::scratch;

fn touch(dir: &std::path::Path, name: &str) {
    std::fs::write(dir.join(name), "x").unwrap();
}

#[test]
fn nothing_provisioned_is_none() {
    let dir = scratch();
    assert_eq!(read_dir(&dir).unwrap(), None);
}

#[test]
fn half_provisioned_names_every_gap_at_once() {
    let dir = scratch();
    touch(&dir, ANCHORS);
    let e = read_dir(&dir).unwrap_err();
    assert!(e.contains("half-provisioned"), "{e}");
    assert!(
        e.contains(CHAIN) && e.contains(KEY) && e.contains(ADDRESS),
        "{e}"
    );
}

#[test]
fn provisioned_reads_back() {
    let dir = scratch();
    for f in [ANCHORS, CHAIN, KEY] {
        touch(&dir, f);
    }
    std::fs::write(dir.join(ADDRESS), "192.0.2.7:7737\n").unwrap();
    let m = read_dir(&dir).unwrap().unwrap();
    assert_eq!(m.address, "192.0.2.7:7737");
    assert_eq!(m.anchors, dir.join(ANCHORS));
    assert_eq!(m.chain, dir.join(CHAIN));
    assert_eq!(m.key, dir.join(KEY));
}

#[test]
fn an_empty_address_refuses() {
    let dir = scratch();
    for f in [ANCHORS, CHAIN, KEY] {
        touch(&dir, f);
    }
    std::fs::write(dir.join(ADDRESS), "  \n").unwrap();
    let e = read_dir(&dir).unwrap_err();
    assert!(e.contains("names no address"), "{e}");
}

/// **The optional pair rides beside the four** (DESIGN §21.1): absent, the
/// entry does not rove; present, both files read; one alone is the same
/// half-provisioned refusal the four earn, one rung over.
#[test]
fn the_roving_pair_is_read_beside_the_four_and_half_of_it_refuses() {
    let dir = scratch();
    for f in [ANCHORS, CHAIN, KEY] {
        touch(&dir, f);
    }
    std::fs::write(dir.join(ADDRESS), "192.0.2.7:7737\n").unwrap();
    assert_eq!(read_dir(&dir).unwrap().unwrap().pairing, None);
    let hex = |fill: u8| format!("{}\n", crate::rendezvous::hex(&[fill; 32]));
    std::fs::write(dir.join(crate::rendezvous::KEY), hex(1)).unwrap();
    let e = read_dir(&dir).unwrap_err();
    assert!(e.contains(crate::rendezvous::SALT), "{e}");
    std::fs::write(dir.join(crate::rendezvous::SALT), hex(2)).unwrap();
    assert_eq!(
        read_dir(&dir).unwrap().unwrap().pairing,
        Some(crate::rendezvous::Pairing {
            engine: [1u8; 32],
            salt: [2u8; 32],
        })
    );
}
