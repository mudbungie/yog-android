//! The learning loop's shapes, read strictly: the listing, the reading beside
//! it, the two verdicts, and every refusal named.

use serde_json::{Value, json};

use super::{Staged, Verdict, staged, verdict};

fn body(v: &Value) -> serde_json::Map<String, Value> {
    v.as_object().unwrap().clone()
}

fn row(id: &str, fresh: bool) -> Value {
    json!({ "id": id, "lineages": ["default"], "parent": "9f2c1ab4",
            "fresh": fresh, "diffstat": "1 file changed, 6 insertions(+)",
            "subject": "notes: record what the span taught" })
}

/// The bare listing reads whole, and **`whole` absent is `None`** — *nobody
/// asked for a reading*, which is a different fact from an empty one.
#[test]
fn the_bare_listing_carries_rows_and_no_reading() {
    let (rows, whole) = staged(&body(
        &json!({ "rows": [row("r1", true), row("r2", false)] }),
    ))
    .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].id, "r1");
    assert_eq!(rows[0].lineages, ["default"]);
    assert_eq!(rows[0].parent, "9f2c1ab4");
    assert!(rows[0].fresh);
    assert_eq!(rows[0].diffstat, "1 file changed, 6 insertions(+)");
    assert_eq!(rows[0].subject, "notes: record what the span taught");
    // **`fresh` is read and never inferred** (REMOTE §9.22): a stale row still
    // states its lineages, and only the engine can be sure the two were read
    // in one pass.
    assert!(!rows[1].fresh);
    assert_eq!(whole, None);
}

/// Naming an id answers that proposal whole beside the listing — one op at
/// its second depth.
#[test]
fn a_named_proposal_rides_beside_the_listing() {
    let frame = json!({ "rows": [row("r1", true)], "whole": "commit 71011c3d\n\n+ a line\n" });
    let (rows, whole) = staged(&body(&frame)).unwrap();
    assert_eq!(rows.len(), 1);
    assert!(whole.unwrap().contains("+ a line"));
}

/// A row of the wrong shape refuses naming the shape it refused.
#[test]
fn a_row_that_is_not_an_object_refuses_by_name() {
    assert_eq!(
        staged(&body(&json!({ "rows": ["r1"] }))).unwrap_err(),
        "proposals: row is not an object"
    );
}

/// A missing field refuses too, rather than reading as an empty one: a
/// listing that dropped `diffstat` would say a patch changes nothing.
#[test]
fn a_row_missing_a_field_refuses() {
    let mut bare = body(&row("r1", true));
    bare.remove("diffstat");
    assert!(
        staged(&body(&json!({ "rows": [Value::Object(bare)] })))
            .unwrap_err()
            .contains("diffstat")
    );
}

/// Both verdicts round-trip through their own word, and the words are the
/// engine's.
#[test]
fn every_verdict_round_trips_through_its_own_word() {
    assert_eq!(Verdict::ALL, [Verdict::Accept, Verdict::Reject]);
    for word in Verdict::ALL {
        let read = verdict(&body(&json!({ "verdict": word.word() }))).unwrap();
        assert_eq!(read, word);
    }
    assert_eq!(Verdict::Accept.word(), "accept");
    assert_eq!(Verdict::Reject.word(), "reject");
}

/// **A verdict this build has not heard of refuses by name.** Reading an
/// unknown word as the nearest one known would, on this op, throw somebody's
/// work away or merge it.
#[test]
fn an_unknown_verdict_refuses_by_name() {
    assert_eq!(
        verdict(&body(&json!({ "verdict": "maybe" }))).unwrap_err(),
        "proposal: unknown verdict \"maybe\""
    );
}

/// **A listing is unpaintable under another workspace's name** — the reply
/// echoes no workspace, so the ask names it and the pairing is what makes the
/// answer honest.
#[test]
fn a_listing_is_about_the_workspace_it_was_read_for() {
    let held = Staged {
        workspace: "home".to_owned(),
        rows: Vec::new(),
        whole: None,
    };
    assert!(held.about("home"));
    assert!(!held.about("away"));
}
