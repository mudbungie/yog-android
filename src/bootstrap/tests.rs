//! The standing, over real minted material — the derivation is the design, so
//! it is proved against certificates rather than against a stored flag.

use super::{Component, Offer, Standing, channels, offers, standing};
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

/// The first-run surface's content. Three branded choices in the ruling's
/// order, two of them the default path and the server not — the ruling's own
/// words — and each naming the grade of leaf that takes it, because the grade
/// is the whole difference between the two enrollments (REMOTE §4.2).
#[test]
fn the_offers_are_three_branded_bootstraps_with_enrollment_emphasised() {
    let offers = offers();
    let components: Vec<Component> = offers.iter().map(|o| o.component).collect();
    assert_eq!(
        components,
        vec![Component::Seat, Component::Foot, Component::Server]
    );
    // The brand is the name on the control, and it is what the operator picks
    // by — a screen listing "seat / tool host / server" is a taxonomy, not a
    // choice.
    let brands: Vec<&str> = offers.iter().map(|o| o.brand.as_str()).collect();
    assert_eq!(brands, vec!["Lernie", "Thrall", "Yog"]);
    // Every choice says what taking it makes this device, under its own name.
    for offer in &offers {
        assert!(!offer.tagline.is_empty(), "{}", offer.brand);
    }
    let default: Vec<Component> = offers
        .iter()
        .filter(|o| o.default)
        .map(|o| o.component)
        .collect();
    assert_eq!(default, vec![Component::Seat, Component::Foot]);
    // Each enrollment names the grade of leaf that takes it, and says that
    // this app never mints one — the §1.4 line, on the screen an operator
    // reads rather than only in a design document.
    for offer in offers.iter().filter(|o| o.default) {
        assert!(offer.how.contains("OU=foot"), "{}", offer.how);
        assert!(offer.how.contains("never mints"), "{}", offer.how);
    }
    // No path in this prose. Where material lands is the shell's own fact,
    // painted from the boot standing beside `material::WANTED`; an earlier
    // shape folded it in here too and the screen said it twice.
    for offer in &offers {
        assert!(!offer.how.contains('/'), "{}", offer.how);
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

/// DESIGN §5's three delivery channels, and the invariant under all three:
/// the material is carried here through existing trust, never fetched by a
/// device asserting something about itself.
#[test]
fn the_enrollment_screen_names_the_three_delivery_channels() {
    let channels = channels();
    assert_eq!(channels.len(), 3);
    assert!(channels[0].contains("cable"), "{}", channels[0]);
    assert!(channels[1].contains("already-trusted"), "{}", channels[1]);
    assert!(channels[2].contains("screen"), "{}", channels[2]);
}

#[test]
fn each_component_wears_a_word() {
    assert_eq!(Component::Seat.word(), "seat");
    assert_eq!(Component::Foot.word(), "tool host");
    assert_eq!(Component::Server.word(), "server");
}
