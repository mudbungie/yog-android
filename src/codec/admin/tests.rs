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

#[test]
fn a_grade_this_wire_does_not_have_refuses_naming_it() {
    let frame = object(
        &json!({ "op": "enroll", "workspace": "ws", "name": "phone-2",
                                "grade": "admin" }),
    );
    assert_eq!(
        crate::codec::enroll::decode(&frame).unwrap_err(),
        "enroll: unknown grade \"admin\""
    );
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
