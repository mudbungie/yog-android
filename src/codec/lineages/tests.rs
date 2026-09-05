//! One lineage row, and the refusals a malformed one earns.

use serde_json::{Value, json};

fn object(v: &Value) -> serde_json::Map<String, Value> {
    v.as_object().cloned().unwrap_or_default()
}

#[test]
fn a_lineage_carries_its_name_its_clipped_tip_and_what_it_holds() {
    let rows = super::rows(&object(&json!({ "rows": [
        { "name": "main", "oid": "abcdef1234", "short_oid": "abcdef1",
          "committed": 1_700_000_000_i64, "files": ["workflow.yaml"] }] })))
    .unwrap();
    let row = rows.first().cloned().unwrap_or_else(|| unreachable!());
    assert_eq!(
        (row.name.as_str(), row.short_oid.as_str()),
        ("main", "abcdef1")
    );
    assert_eq!(row.committed, 1_700_000_000);
    assert_eq!(row.files, ["workflow.yaml"]);
}

#[test]
fn a_row_that_is_not_an_object_or_states_no_name_refuses_by_name() {
    assert_eq!(
        super::rows(&object(&json!({ "rows": ["main"] }))).unwrap_err(),
        "lineages: row is not an object"
    );
    assert_eq!(
        super::rows(&object(
            &json!({ "rows": [{ "short_oid": "a", "committed": 0 }] })
        ))
        .unwrap_err(),
        "missing or non-string field \"name\""
    );
}
