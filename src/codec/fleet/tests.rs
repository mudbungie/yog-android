//! The four spellings, and the one thing this family needs a test for above
//! all: **the receipt says a state and never a setting**, so the sentence a
//! seat paints is composed from the ANSWER and the OP together.

use super::FleetAct;
use serde_json::json;

#[test]
fn each_arming_spells_its_own_envelope_with_the_workspace_stated_once() {
    assert_eq!(
        super::encode(
            "home",
            &FleetAct::Fleet {
                project: "p".to_owned(),
                cap: 4,
            }
        ),
        json!({ "op": "fleet", "workspace": "home", "project": "p", "cap": 4 })
    );
    assert_eq!(
        super::encode(
            "home",
            &FleetAct::Arm {
                model: "haiku".to_owned(),
            }
        ),
        json!({ "op": "arm", "workspace": "home", "model": "haiku" })
    );
    assert_eq!(
        super::encode("home", &FleetAct::Disband),
        json!({ "op": "disband", "workspace": "home" })
    );
    assert_eq!(
        super::encode("home", &FleetAct::Disarm),
        json!({ "op": "disarm", "workspace": "home" })
    );
}

/// **The naming trap, asserted**: one boolean, two settings, and the op is the
/// only thing that says which. A reader that classified off the reply would be
/// guessing between the loop and the monitor.
#[test]
fn the_sentence_is_the_answer_read_against_the_op_that_earned_it() {
    let fleet = FleetAct::Fleet {
        project: "p".to_owned(),
        cap: 2,
    };
    assert_eq!(fleet.said(true), "fleet: the loop is armed");
    assert_eq!(
        FleetAct::Disband.said(false),
        "disband: the loop is not armed"
    );
    let arm = FleetAct::Arm {
        model: "haiku".to_owned(),
    };
    assert_eq!(arm.said(true), "arm: the monitor is armed");
    assert_eq!(
        FleetAct::Disarm.said(false),
        "disarm: the monitor is not armed"
    );
}

/// **Only the two that START something take a word**, and each says which word
/// it takes — which is what disambiguates one field between two live controls.
#[test]
fn the_word_each_act_takes_is_stated_on_the_act_itself() {
    let fleet = FleetAct::Fleet {
        project: String::new(),
        cap: 1,
    }
    .with("p".to_owned(), 3);
    assert_eq!(
        fleet,
        FleetAct::Fleet {
            project: "p".to_owned(),
            cap: 3,
        }
    );
    assert_eq!(fleet.wants(), Some("name the project to run"));
    let arm = FleetAct::Arm {
        model: String::new(),
    }
    .with("haiku".to_owned(), 3);
    assert_eq!(
        arm,
        FleetAct::Arm {
            model: "haiku".to_owned(),
        }
    );
    assert_eq!(arm.wants(), Some("name the monitor's model"));
    assert_eq!(FleetAct::Disband.with("p".to_owned(), 3), FleetAct::Disband);
    assert_eq!(FleetAct::Disarm.with("p".to_owned(), 3), FleetAct::Disarm);
    assert_eq!(FleetAct::Disband.wants(), None);
    assert_eq!(FleetAct::Disarm.wants(), None);
}
