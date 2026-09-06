//! The worktree read, and the refusals nobody would otherwise see.
//!
//! The corpus replay (`tests/conformance`) drives the two real `files` frames
//! — a present worktree with a text preview, and a torn-down one with a binary
//! preview — so what is asserted here is what those frames do not reach: the
//! pairing law, and every way a malformed answer is named rather than guessed
//! at.

use serde_json::{Value, json};

use super::{Files, Listing, Preview};

fn object(v: &Value) -> serde_json::Map<String, Value> {
    v.as_object().cloned().unwrap_or_default()
}

fn held(listing: Listing) -> Files {
    Files {
        workspace: "ws".to_owned(),
        agent: "c-1".to_owned(),
        listing,
        opened: String::new(),
    }
}

#[test]
fn a_torn_down_worktree_states_no_rows_and_is_not_an_empty_listing() {
    let gone = super::listing(&object(&json!({ "worktree": false }))).unwrap();
    assert!(!gone.worktree);
    assert_eq!((gone.rows.len(), gone.truncated), (0, false));
    assert_eq!((gone.working_dir.as_str(), gone.preview), ("", None));
}

#[test]
fn a_present_worktree_reads_its_rows_and_where_the_work_actually_lands() {
    let listed = super::listing(&object(&json!({
        "worktree": true, "truncated": false, "working_dir": "/home/u/proj",
        "rows": [{ "path": "src/a.rs", "size": 12, "dir": false },
                 { "path": "src", "size": 0, "dir": true }],
    })))
    .unwrap();
    assert_eq!(listed.working_dir, "/home/u/proj");
    assert_eq!(listed.rows.len(), 2);
    assert!(listed.rows.get(1).is_some_and(|row| row.dir));
}

#[test]
fn a_listing_is_about_the_conversation_it_was_read_at_and_no_other() {
    let files = held(super::listing(&object(&json!({ "worktree": false }))).unwrap());
    assert!(files.about("ws", "c-1"));
    assert!(!files.about("ws", "c-2"));
    assert!(!files.about("other", "c-1"));
}

#[test]
fn a_malformed_row_refuses_naming_the_shape_it_was_in() {
    let rows = json!({ "worktree": true, "truncated": false, "rows": ["src/a.rs"] });
    assert_eq!(
        super::listing(&object(&rows)).unwrap_err(),
        "files: a row is not an object"
    );
    let sizeless = json!({ "worktree": true, "truncated": false,
                           "rows": [{ "path": "src/a.rs", "dir": false }] });
    assert_eq!(
        super::listing(&object(&sizeless)).unwrap_err(),
        "missing or non-integer field \"size\""
    );
}

#[test]
fn a_preview_this_codec_cannot_class_refuses_rather_than_guessing() {
    assert_eq!(
        super::preview(&json!("body")).unwrap_err(),
        "preview: not an object"
    );
    assert_eq!(
        super::preview(&json!({ "kind": "sparse" })).unwrap_err(),
        "preview: unknown kind \"sparse\""
    );
}

#[test]
fn a_truncated_preview_carries_what_the_whole_would_have_been() {
    let cut =
        super::preview(&json!({ "kind": "truncated", "text": "head", "size": 9000 })).unwrap();
    assert_eq!(
        cut,
        Preview::Truncated {
            text: "head".to_owned(),
            size: 9000
        }
    );
}
