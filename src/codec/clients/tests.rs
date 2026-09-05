//! The roster's two lifetimes, and the one shape an advertised element has
//! wherever it is said.

use serde_json::{Value, json};

fn object(v: &Value) -> serde_json::Map<String, Value> {
    v.as_object().cloned().unwrap_or_default()
}

/// **A machine that is not connected still says what it offers**, and one that
/// is connected may offer nothing. The two facts have different lifetimes and
/// neither is derived from the other.
#[test]
fn presence_and_the_advertised_set_are_two_facts_with_two_lifetimes() {
    let rows = super::rows(&object(&json!({ "rows": [
        { "client": "laptop", "present": false, "tools": [
            { "name": "Bash", "description": "run a command",
              "input_schema": { "type": "object" }, "subject_cwd": true }] },
        { "client": "phone", "present": true, "tools": [] }] })))
    .unwrap();
    let laptop = rows.first().cloned().unwrap_or_else(|| unreachable!());
    assert!(!laptop.present);
    assert_eq!(laptop.tools.len(), 1);
    assert!(laptop.tools.first().is_some_and(|tool| tool.subject_cwd));
    let phone = rows.get(1).cloned().unwrap_or_else(|| unreachable!());
    assert!(phone.present && phone.tools.is_empty());
}

/// **The consent is a fact of the element and absent reads false**, which is
/// `codec::tools`' own rule — one reader, so this roster cannot disagree with
/// what the same machine's advertisement said.
#[test]
fn an_element_that_states_no_consent_reads_as_refusing_one() {
    let rows = super::rows(&object(&json!({ "rows": [
        { "client": "laptop", "present": true, "tools": [
            { "name": "Bash", "description": "d", "input_schema": {} }] }] })))
    .unwrap();
    assert!(
        rows.first()
            .and_then(|row| row.tools.first())
            .is_some_and(|tool| !tool.subject_cwd)
    );
}

#[test]
fn a_row_that_is_not_an_object_or_states_no_client_refuses_by_name() {
    assert_eq!(
        super::rows(&object(&json!({ "rows": ["laptop"] }))).unwrap_err(),
        "clients: row is not an object"
    );
    assert_eq!(
        super::rows(&object(
            &json!({ "rows": [{ "present": true, "tools": [] }] })
        ))
        .unwrap_err(),
        "missing or non-string field \"client\""
    );
}

/// **The roster is paintable only under the workspace it was read for.**
#[test]
fn a_roster_is_paintable_only_under_the_workspace_it_was_read_for() {
    let machines = super::Machines {
        workspace: "home".to_owned(),
        rows: Vec::new(),
    };
    assert!(machines.about("home"));
    assert!(!machines.about("other"));
}
