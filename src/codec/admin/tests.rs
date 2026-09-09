//! The admin family's own readings, and the refusals nobody would otherwise
//! see.
//!
//! The corpus replay (`tests/conformance`) drives every real frame of all five
//! ops in both directions, so what is asserted here is what those frames do
//! not reach: the destination refusals, the malformed target, and the pairing
//! law over a mark.

use serde_json::{Value, json};

use super::{Destination, Marks};

fn object(v: &Value) -> serde_json::Map<String, Value> {
    v.as_object().cloned().unwrap_or_default()
}

#[test]
fn a_destination_this_seat_has_no_picker_for_refuses_naming_it() {
    let workflow = json!({ "file": "litany-workflow", "name": "review" });
    assert_eq!(
        super::destination(Some(&workflow)).unwrap_err(),
        "config: unimplemented destination \"litany-workflow\""
    );
    let branch = json!({ "file": "branch", "lineage": "default", "origin": "advance",
                         "path": "providers.yaml", "workspace": "ws" });
    assert_eq!(
        super::destination(Some(&branch)).unwrap_err(),
        "config: unimplemented destination \"branch\""
    );
}

#[test]
fn a_target_that_is_absent_or_not_an_object_refuses_naming_the_field() {
    assert_eq!(
        super::destination(None).unwrap_err(),
        "config: missing field \"target\""
    );
    assert_eq!(
        super::destination(Some(&json!("brazen"))).unwrap_err(),
        "config: \"target\" is not an object"
    );
}

#[test]
fn every_destination_says_its_own_file_word() {
    let named = [
        (
            Destination::Brazen {
                workspace: "ws".to_owned(),
            },
            "brazen",
        ),
        (Destination::LitanyModels, "litany-models"),
        (Destination::Cadence, "cadence"),
    ];
    for (at, file) in named {
        assert_eq!(at.file(), file);
    }
}

#[test]
fn a_mark_is_about_the_workspace_it_was_read_at_and_no_other() {
    let marks = Marks {
        workspace: "home".to_owned(),
        branch: "balls/tasks".to_owned(),
    };
    assert!(marks.about("home"));
    assert!(!marks.about("other"));
}

#[test]
fn an_answer_that_states_no_text_or_no_branch_refuses_naming_the_field() {
    assert_eq!(
        super::config(&object(&json!({ "kind": "config" }))).unwrap_err(),
        "missing or non-string field \"text\""
    );
    assert_eq!(
        super::marks(&object(&json!({ "kind": "marks" }))).unwrap_err(),
        "missing or non-string field \"branch\""
    );
}

/// **A third grade word is carried, not refused** (REMOTE §3.2). The
/// certificate is §4.2's authority for what was minted, and
/// `crate::envelope::agrees` is where the stated word is held against it — so
/// this reader's job is to say what the frame said, and it round-trips the
/// word rather than flattening it into one of the two it knows.
#[test]
fn a_grade_this_build_has_not_heard_of_rides_as_itself() {
    let frame = object(
        &json!({ "op": "enroll", "workspace": "ws", "name": "phone-2",
                                "grade": "admin" }),
    );
    let act = crate::codec::enroll::decode(&frame).unwrap();
    assert!(
        matches!(&act, crate::codec::Act::Enroll { grade, .. }
            if *grade == crate::leaf::Grade::Unknown("admin".to_owned())),
        "{act:?}"
    );
    // And it round-trips as the word it arrived as: the catch-all says
    // *unknown* on a GLASS, never on a wire.
    let gesture = crate::codec::Gesture::Act(act);
    assert_eq!(
        crate::codec::encode(&gesture),
        json!({ "op": "enroll", "workspace": "ws", "name": "phone-2",
                "grade": "admin" })
    );
}

/// **A stated route refuses by name** (REMOTE §8.4, PROTOCOL 14). The address
/// the enrolled device will dial is a fact about a box this seat cannot see,
/// so a frame carrying one is refused rather than read as the mint without it
/// — the silent misread REMOTE §3's third rule forbids, and the recorded
/// decision `tests/conformance/requests.rs` counts.
#[test]
fn an_enrolment_stating_a_route_refuses_naming_the_op() {
    let frame = object(
        &json!({ "op": "enroll", "workspace": "ws", "name": "phone-2",
                 "grade": "operator", "address": "engine.invalid:7737" }),
    );
    let refusal = crate::codec::enroll::decode(&frame).unwrap_err();
    assert!(refusal.starts_with("enroll: "), "{refusal}");
    assert!(refusal.contains("engine.invalid"), "{refusal}");
}

#[test]
fn an_op_outside_this_family_refuses_by_name() {
    let frame = object(&json!({ "op": "pin", "workspace": "ws" }));
    assert_eq!(
        super::act::decode("pin", &frame).unwrap_err(),
        "admin: unknown op \"pin\""
    );
}

#[test]
fn every_act_that_takes_a_word_says_which_word_while_it_is_dark() {
    use crate::codec::{AdminAct, Destination};
    let named = [
        (
            AdminAct::Config {
                at: Destination::Cadence,
                text: String::new(),
            },
            "edit the file first",
        ),
        (
            AdminAct::Marks {
                workspace: "ws".to_owned(),
                branch: String::new(),
            },
            "type the branch first",
        ),
        (
            AdminAct::DeleteWorkspace {
                workspace: "ws".to_owned(),
                typed: String::new(),
            },
            "type this workspace's name",
        ),
    ];
    for (act, asks) in named {
        assert_eq!(act.wants(), Some(asks));
    }
}

#[test]
fn the_two_that_take_no_word_are_never_dark_for_want_of_one() {
    use crate::codec::AdminAct;
    let scan = AdminAct::Scan {
        workspace: "ws".to_owned(),
    };
    let delete = AdminAct::DeleteAgent {
        workspace: "ws".to_owned(),
        agent: "c-1".to_owned(),
        typed: String::new(),
    };
    assert_eq!((scan.wants(), delete.wants()), (None, None));
    assert_eq!((scan.op(), delete.op()), ("scan", "delete-agent"));
}
