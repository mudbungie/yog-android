//! The diff row read strictly, and the refusals nobody would otherwise see.
//!
//! The corpus replay (`tests/conformance`) drives every real frame of both
//! answers that carry one — `work-diff` and `science`, which is the point of
//! there being one reader — so what is asserted here is the pairing law, the
//! state token's own strictness, and the shapes a malformed row would
//! otherwise be guessed at in.

use serde_json::{Value, json};

use super::{Churn, Work};

fn object(v: &Value) -> serde_json::Map<String, Value> {
    v.as_object().cloned().unwrap_or_default()
}

#[test]
fn a_listing_is_about_the_workspace_it_was_read_at_and_no_other() {
    let work = Work {
        workspace: "ws".to_owned(),
        rows: Vec::new(),
        patch: None,
        opened: None,
    };
    assert!(work.about("ws"));
    assert!(!work.about("other"));
}

#[test]
fn an_answer_with_no_patch_says_so_rather_than_carrying_an_empty_one() {
    let bare = super::churned(&object(&json!({ "rows": [] }))).unwrap();
    assert_eq!((bare.rows.len(), bare.patch), (0, None));
}

#[test]
fn a_state_this_codec_cannot_read_refuses_naming_it() {
    let stray = json!({ "ball_id": "bl-1", "project": "p", "state": "renamed" });
    assert_eq!(
        super::diff(&stray).unwrap_err(),
        "work-diff: unknown state \"renamed\""
    );
    assert_eq!(
        super::diff(&json!("bl-1")).unwrap_err(),
        "work-diff: a row is not an object"
    );
}

#[test]
fn an_absent_ref_names_what_is_missing_and_refuses_a_missing_list_of_anything_else() {
    let absent = super::diff(
        &json!({ "ball_id": "bl-2", "project": "p", "state": "absent",
                                      "target": "main", "source": "work/bl-2",
                                      "missing": ["work/bl-2"] }),
    )
    .unwrap();
    assert_eq!(absent.missing, vec!["work/bl-2".to_owned()]);
    assert_eq!((absent.target_oid.as_str(), absent.files.len()), ("", 0));
    let numbered = json!({ "ball_id": "bl-2", "project": "p", "state": "absent",
                           "target": "main", "source": "work/bl-2", "missing": [7] });
    assert_eq!(
        super::diff(&numbered).unwrap_err(),
        "work-diff: non-string element in field \"missing\""
    );
}

#[test]
fn churn_is_binary_by_its_shape_and_counted_by_its_absence() {
    let diffed = super::diff(&json!({ "ball_id": "bl-3", "project": "p", "state": "diff",
                                      "target": "main", "source": "work/bl-3",
                                      "target_oid": "aaa", "source_oid": "bbb",
                                      "truncated": false,
                                      "files": [{ "path": "a", "binary": true },
                                                { "path": "b", "added": 3, "removed": 1 }] }))
    .unwrap();
    assert_eq!(
        diffed.files.first(),
        Some(&Churn {
            path: "a".to_owned(),
            added: 0,
            removed: 0,
            binary: true
        })
    );
    assert!(diffed.files.get(1).is_some_and(|file| !file.binary));
}

#[test]
fn a_malformed_churn_refuses_naming_the_shape_it_was_in() {
    let listed = json!({ "ball_id": "bl-3", "project": "p", "state": "diff",
                         "target": "main", "source": "work/bl-3",
                         "target_oid": "aaa", "source_oid": "bbb", "truncated": false,
                         "files": ["src/a.rs"] });
    assert_eq!(
        super::diff(&listed).unwrap_err(),
        "work-diff: a file is not an object"
    );
}

#[test]
fn the_asked_file_refuses_a_shape_that_is_not_an_object() {
    assert_eq!(
        super::file::decode(&json!("src/a.rs")).unwrap_err(),
        "work-diff: \"file\" is not an object"
    );
}
