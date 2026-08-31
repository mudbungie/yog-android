//! The standing, over real minted material — the derivation is the design, so
//! it is proved against certificates rather than against a stored flag.

use super::{Component, Offer, Standing, offers, standing};
use crate::test_support::{mint_ca, mint_foot, mint_leaf, scratch};
use std::path::Path;

/// Lay a provisioned directory out the way a delivery channel does: the four
/// files, under the names the channels write.
fn provision(dir: &Path, ca: &str, leaf: &str) -> std::path::PathBuf {
    let wire = dir.join("wire");
    std::fs::create_dir_all(&wire).unwrap();
    std::fs::copy(dir.join(format!("{ca}.pem")), wire.join("ca.pem")).unwrap();
    std::fs::copy(dir.join(format!("{leaf}.pem")), wire.join("client.pem")).unwrap();
    std::fs::copy(dir.join(format!("{leaf}.key")), wire.join("client.key")).unwrap();
    std::fs::write(wire.join("address"), "engine.example.com:7737").unwrap();
    wire
}

/// The gate, and it is the absence of a mechanism: an app with nothing
/// provisioned runs no component. Nothing was decided and nothing started.
#[test]
fn a_device_with_no_material_is_cold() {
    let dir = scratch();
    assert_eq!(standing(&dir.join("wire")), Ok(Standing::Cold));
}

/// The whole ruling in one assertion: the component is read off the leaf the
/// operator issued, so enrolling a phone as a tool host is minting it a
/// foot-grade certificate — not tapping a setting on the phone.
#[test]
fn the_leaf_says_which_component_this_device_is() {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "desk", false);
    mint_foot(&dir, "ca", "phone");

    let Standing::Enrolled(seat) = standing(&provision(&dir, "ca", "desk")).unwrap() else {
        panic!("not enrolled");
    };
    assert_eq!(seat.component, Component::Seat);
    assert_eq!(seat.client, "notreal-desk");
    assert_eq!(seat.material.address, "engine.example.com:7737");

    let dir2 = scratch();
    mint_ca(&dir2, "ca");
    mint_foot(&dir2, "ca", "phone");
    let Standing::Enrolled(foot) = standing(&provision(&dir2, "ca", "phone")).unwrap() else {
        panic!("not enrolled");
    };
    assert_eq!(foot.component, Component::Foot);
    assert_eq!(foot.client, "notreal-phone");
}

/// Half a trust store is a misconfiguration, and the sentence names every
/// absent file at once — a remedy that reveals one gap per run is a remedy run
/// four times. The standing does not swallow it into "cold": a device that has
/// something is not a device that has nothing.
#[test]
fn a_half_provisioned_device_refuses_rather_than_reading_as_cold() {
    let dir = scratch();
    let wire = dir.join("wire");
    std::fs::create_dir_all(&wire).unwrap();
    std::fs::write(wire.join("address"), "engine.example.com:7737").unwrap();
    let e = standing(&wire).unwrap_err();
    assert!(e.contains("half-provisioned"), "{e}");
    assert!(e.contains("ca.pem"), "{e}");
}

/// A certificate file that holds no certificate: the material is all there by
/// name, so this is not the half-provisioned case, and reading a grade off
/// nothing must not silently answer the default one.
#[test]
fn a_chain_with_no_certificate_in_it_refuses() {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "desk", false);
    let wire = provision(&dir, "ca", "desk");
    std::fs::write(wire.join("client.pem"), "not a certificate\n").unwrap();
    let e = standing(&wire).unwrap_err();
    assert!(e.contains("client.pem"), "{e}");
}

/// The first-run surface's content. Two enrollments carry the default emphasis
/// and the server does not — the ruling's own words — and every offer names
/// the directory material lands in, because an operator holding a cable needs
/// the path rather than a description of one.
#[test]
fn the_offers_are_the_three_bootstraps_with_enrollment_emphasised() {
    let dir = Path::new("/home/u/files/wire");
    let offers = offers(dir);
    let components: Vec<Component> = offers.iter().map(|o| o.component).collect();
    assert_eq!(
        components,
        vec![Component::Seat, Component::Foot, Component::Server]
    );
    let default: Vec<Component> = offers
        .iter()
        .filter(|o| o.default)
        .map(|o| o.component)
        .collect();
    assert_eq!(default, vec![Component::Seat, Component::Foot]);
    for offer in offers.iter().filter(|o| o.default) {
        assert!(offer.how.contains("/home/u/files/wire"), "{}", offer.how);
    }
    // The server offer states what it needs and starts nothing (bl-d6c6).
    let server: Vec<&Offer> = offers
        .iter()
        .filter(|o| o.component == Component::Server)
        .collect();
    assert_eq!(server.len(), 1);
    assert!(server[0].how.contains("Not yet"), "{}", server[0].how);
    // It names the two rungs that are not walked, because "not yet" without
    // them is a shrug (DESIGN §10).
    assert!(server[0].how.contains("git"), "{}", server[0].how);
    assert!(server[0].how.contains("API 29"), "{}", server[0].how);
}

#[test]
fn each_component_wears_a_word() {
    assert_eq!(Component::Seat.word(), "seat");
    assert_eq!(Component::Foot.word(), "tool host");
    assert_eq!(Component::Server.word(), "server");
}
