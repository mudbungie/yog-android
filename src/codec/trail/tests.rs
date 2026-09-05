//! The trail row, read strictly — and the refusal naming the field that was
//! not there, which is what tells a client too old for a shape from one that
//! read past it.

use super::Standing;
use serde_json::{Value, json};

fn line(exit: i64, failed: bool, label: &str, standing: &str) -> Value {
    json!({ "argv": "bl close x", "cwd": "/p", "exit": exit, "origin": "balls",
            "stderr": "gate", "stdout": "", "ts": "1700", "failed": failed,
            "exit_label": label, "standing": standing })
}

#[test]
fn a_row_reads_its_own_words_and_derives_nothing() {
    let row = super::row(&line(1, true, "exit 1", "live")).unwrap();
    assert_eq!(
        (row.ts.as_str(), row.origin.as_str(), row.exit),
        ("1700", "balls", 1)
    );
    assert_eq!(row.stderr, "gate");
    assert!(row.failed);
    assert_eq!(row.exit_label, "exit 1");
    assert_eq!(row.standing, Standing::Live);
}

/// A sentinel exit crosses as the number the engine wrote AND as the engine's
/// reading of it (REMOTE §9.17): nothing here reads meaning into the number.
#[test]
fn a_sentinel_exit_is_carried_with_the_engines_reading_of_it() {
    let row = super::row(&line(
        -2,
        false,
        "detached — handed off, no exit to observe",
        "detached",
    ))
    .unwrap();
    assert_eq!(row.exit, -2);
    assert!(!row.failed);
    assert_eq!(row.standing, Standing::Detached);
}

/// The five words, each its own standing, and the word reads back as the
/// label the trail paints — one table in both directions.
#[test]
fn every_standing_is_read_and_says_its_own_word() {
    for word in ["clean", "detached", "live", "retired", "acked"] {
        let row = super::row(&line(0, false, "exit 0", word)).unwrap();
        assert_eq!(row.standing.word(), word);
    }
    assert_eq!(
        super::row(&line(0, false, "exit 0", "sleeping")).unwrap_err(),
        "field \"standing\": unknown token \"sleeping\""
    );
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
                                  "origin": "world", "stdout": "", "ts": "1",
                                  "failed": false, "exit_label": "exit 0",
                                  "standing": "clean" }))
    .unwrap_err();
    assert_eq!(err, "missing or non-string field \"stderr\"");
}
