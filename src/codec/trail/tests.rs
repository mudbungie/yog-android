//! The trail row, read strictly — and the refusal naming the field that was
//! not there, which is what tells a client too old for a shape from one that
//! read past it.

use serde_json::json;

#[test]
fn a_row_reads_its_own_facts_and_derives_nothing() {
    let row = super::row(&json!({ "argv": "bl close x", "cwd": "/p", "exit": 1,
                                  "origin": "balls", "stderr": "gate",
                                  "stdout": "", "ts": "1700" }))
    .unwrap();
    assert_eq!(
        (row.ts.as_str(), row.origin.as_str(), row.exit),
        ("1700", "balls", 1)
    );
    assert_eq!(row.stderr, "gate");
}

/// A sentinel exit crosses as the number the engine wrote. Nothing here reads
/// meaning into it: the words for it are `exit_label` and `standing`, which
/// this build's corpus does not carry (bl-8e3c).
#[test]
fn a_sentinel_exit_is_carried_not_interpreted() {
    let row = super::row(
        &json!({ "argv": "litany prompt c-1", "cwd": "/p", "exit": -2,
                                  "origin": "conversation", "stderr": "",
                                  "stdout": "", "ts": "1705" }),
    )
    .unwrap();
    assert_eq!(row.exit, -2);
}

#[test]
fn a_row_that_is_not_an_object_refuses_naming_the_shape() {
    assert_eq!(
        super::row(&json!("nope")).unwrap_err(),
        "ops row: not an object"
    );
}

#[test]
fn a_missing_field_refuses_naming_it() {
    let err = super::row(&json!({ "argv": "x", "cwd": "/p", "exit": 0,
                                  "origin": "world", "stdout": "", "ts": "1" }))
    .unwrap_err();
    assert_eq!(err, "missing or non-string field \"stderr\"");
}
