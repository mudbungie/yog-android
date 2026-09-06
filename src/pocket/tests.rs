//! The pocketed foot's whole decision, over real minted material and every
//! state a host can publish. Nothing here needs a device: the platform half is
//! a service that starts when [`line`] answers and stops when it does not, so
//! what is proved here is what that service does.

use super::line;
use crate::host::{Health, Standing};
use crate::test_support::{mint_ca, mint_foot, mint_leaf, scratch};
use std::path::{Path, PathBuf};

/// This app's private files directory as the platform hands it over, with a
/// `wire/` under it the way a delivery channel writes one (§9's four files).
fn provisioned(leaf: &str, foot: bool) -> PathBuf {
    let files = scratch();
    mint_ca(&files, "ca");
    if foot {
        mint_foot(&files, "ca", leaf);
    } else {
        mint_leaf(&files, "ca", leaf, false);
    }
    let wire = files.join("wire");
    std::fs::create_dir_all(&wire).unwrap();
    std::fs::copy(files.join("ca.pem"), wire.join("ca.pem")).unwrap();
    std::fs::copy(files.join(format!("{leaf}.pem")), wire.join("client.pem")).unwrap();
    std::fs::copy(files.join(format!("{leaf}.key")), wire.join("client.key")).unwrap();
    std::fs::write(wire.join("address"), "engine.example.com:7737").unwrap();
    files
}

fn serving() -> Standing {
    Standing {
        tools: vec!["shell".to_owned(), "read_file".to_owned()],
        advertised: true,
        served: 0,
        restored: 0,
        last: None,
        health: Health::Serving,
    }
}

fn said(files: &Path, standing: Standing) -> (String, String) {
    let notice = line(files, Some(standing)).expect("a foot with a host holds the pocket");
    (notice.title, notice.text)
}

/// **The consent gate, and it is the certificate.** A foot-grade leaf is the
/// operator's enrolment of this device as hands (§16.1's gate 1), so it is the
/// one that holds its lane while pocketed. Everything else answers nothing,
/// which is the service never starting.
#[test]
fn only_a_foot_grade_leaf_holds_the_pocket() {
    assert!(line(&provisioned("phone", true), Some(serving())).is_some());
    assert_eq!(line(&provisioned("desk", false), Some(serving())), None);
}

/// A cold device holds nothing, and neither does one whose material will not
/// read: the direction to fail in is the one that spends no battery on a
/// device nobody enrolled.
#[test]
fn an_unenrolled_device_never_holds_the_pocket() {
    assert_eq!(line(&scratch(), Some(serving())), None);
    let half = scratch();
    std::fs::create_dir_all(half.join("wire")).unwrap();
    std::fs::write(half.join("wire").join("ca.pem"), "not a certificate").unwrap();
    assert_eq!(line(&half, Some(serving())), None);
}

/// **Hands with no lane still answer**, and `None` is reserved for the one
/// thing the service stops on. Two causes share this line — the moment before
/// the boot has taken its host up, and material that names a foot but will not
/// build one — and a service that stopped on either would race the first or
/// silence the second.
#[test]
fn a_foot_with_no_host_says_so_rather_than_going_quiet() {
    let notice = line(&provisioned("phone", true), None).expect("hands still answer");
    assert_eq!(notice.title, "this phone is not serving");
    assert!(
        notice.text.contains("Open yog to see why"),
        "{}",
        notice.text
    );
}

/// The ordinary standing, before any work: the count of what is offered, the
/// honest "nothing yet", and the price in the same breath.
#[test]
fn a_serving_lane_says_what_is_offered_and_what_it_costs() {
    let files = provisioned("phone", true);
    let (title, text) = said(&files, serving());
    assert_eq!(title, "this phone is standing by as hands");
    assert!(
        text.starts_with("2 tools offered · nothing called yet."),
        "{text}"
    );
    assert!(text.contains("A connection stays open"), "{text}");
    assert!(text.contains("radio wakes"), "{text}");
}

/// Work that has happened is what an operator wants from the line — the count
/// and the last tool, in the roster's own vocabulary.
#[test]
fn a_lane_that_has_served_says_how_much_and_what_last() {
    let files = provisioned("phone", true);
    let standing = Standing {
        served: 3,
        last: Some("shell → 0".to_owned()),
        ..serving()
    };
    let (_, text) = said(&files, standing);
    assert!(
        text.starts_with("2 tools offered · served 3 · shell → 0."),
        "{text}"
    );
}

/// The channel is up but the presentation has not landed. A separate title,
/// because "standing by" would be a claim this device has not earned yet.
#[test]
fn a_lane_that_has_not_presented_yet_says_so() {
    let files = provisioned("phone", true);
    let standing = Standing {
        advertised: false,
        ..serving()
    };
    let (title, text) = said(&files, standing);
    assert_eq!(title, "this phone is offering its tools");
    assert!(
        text.starts_with("presenting 2 tools to the engine."),
        "{text}"
    );
}

/// **The redial IS the feature on a phone** (§18): a network flap is the
/// ordinary case, so the shade says what broke and that this end is still
/// trying — never that it is serving.
#[test]
fn a_reconnecting_lane_names_what_broke_and_says_it_keeps_trying() {
    let files = provisioned("phone", true);
    let standing = Standing {
        health: Health::Redialling("receive: Software caused connection abort".to_owned()),
        ..serving()
    };
    let (title, text) = said(&files, standing);
    assert_eq!(title, "this phone is reconnecting");
    assert!(
        text.starts_with("receive: Software caused connection abort."),
        "{text}"
    );
    assert!(text.contains("more slowly each time"), "{text}");
}

/// The one state no redial mends. The notification stands rather than
/// vanishing with a stopped service, because it is the only surface a pocketed
/// phone has to say so — and it states that nothing is being spent, which is
/// the other half of an honest price.
#[test]
fn a_stopped_lane_says_what_ended_it_and_that_nothing_is_being_spent() {
    let files = provisioned("phone", true);
    let standing = Standing {
        health: Health::Stopped("not registered here".to_owned()),
        ..serving()
    };
    let (title, text) = said(&files, standing);
    assert_eq!(title, "this phone has stopped serving");
    assert!(text.starts_with("not registered here."), "{text}");
    assert!(text.contains("Nothing is on the network now"), "{text}");
}

/// **A disarming heals itself and still has to reach somebody** (REMOTE §5.1,
/// bl-cc54). The roster paints it; a pocketed phone's roster is not being
/// looked at, so the shade carries the same words and how many times.
#[test]
fn a_healed_disarming_reaches_the_shade_in_the_hosts_own_words() {
    let files = provisioned("phone", true);
    let standing = Standing {
        restored: 2,
        ..serving()
    };
    let (_, text) = said(&files, standing);
    assert!(text.contains(crate::host::RESTORED), "{text}");
    assert!(text.ends_with("(×2)"), "{text}");
}

// --- the attention lane's half (§17.6, bl-b82d) ---------------------------

/// **A seat holds the attention lane; a foot does not, and no device is
/// both.** The two lanes are mutually exclusive by GRADE rather than by
/// arbitration (REMOTE §4.2: a foot cannot ask about the world), which is what
/// lets one service carry one notification without choosing between two lines.
#[test]
fn the_two_lanes_are_told_apart_by_the_leaf_and_never_overlap() {
    let seat = provisioned("phone", false);
    let foot = provisioned("hands", true);
    assert!(super::attending(&seat).is_some());
    assert!(super::line(&seat, None).is_none());
    assert!(super::attending(&foot).is_none());
    assert!(super::line(&foot, None).is_some());
}

/// **Unreadable material is not a seat**, which is the direction this rung
/// must fail in: a device nobody enrolled spends no battery holding anything.
#[test]
fn a_device_with_no_material_holds_no_lane() {
    let bare = scratch();
    assert!(super::attending(&bare).is_none());
    assert!(super::line(&bare, None).is_none());
}

/// **The price is in the line the operator reads**, and so is the act that
/// ends it — the house rule that a standing cost is stated where it is met
/// (§17.3's precedent, §18.4's own).
#[test]
fn the_held_lane_states_its_price_and_the_act_that_ends_it() {
    let seat = provisioned("phone", false);
    let notice = super::attending(&seat).unwrap();
    assert_eq!(notice.title, "yog is listening for your turn");
    assert!(notice.text.contains("radio wakes"), "{}", notice.text);
    assert!(notice.text.contains("unrestricted"), "{}", notice.text);
    assert!(notice.text.contains("Attention"), "{}", notice.text);
}
