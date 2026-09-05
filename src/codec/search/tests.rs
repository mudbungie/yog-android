//! The search answer, both directions of the strictness contract: the
//! engine's own three address shapes read back, the empty answer is a value
//! rather than an absence, and every malformed shape refuses **naming the
//! shape** — which is what the conformance replay asserts of a refusal.

use serde_json::{Value, json};

use super::{Address, HitField, found_of};

/// The engine's own envelope, as `corpus/reply/search.json` carries it.
fn envelope(rows: Value, unreadable: Value) -> serde_json::Map<String, Value> {
    json!({ "ok": true, "kind": "search", "needle": "gate",
            "rows": rows, "unreadable": unreadable })
    .as_object()
    .unwrap()
    .clone()
}

fn one(row: Value) -> super::Hit {
    found_of(&envelope(json!([row]), json!([])))
        .unwrap()
        .hits
        .remove(0)
}

#[test]
fn the_three_addresses_read_back() {
    let ball = one(json!({ "at": "ball", "project": "p", "id": "bl-1",
                           "field": "name", "offset": 0, "excerpt": "bl-1" }));
    assert_eq!(
        ball.at,
        Address::Ball {
            project: "p".into(),
            id: "bl-1".into()
        }
    );
    assert_eq!(ball.field, HitField::Name);
    let workspace = one(json!({ "at": "workspace", "workspace": "ws",
                                "field": "summary", "offset": 3, "excerpt": "ws" }));
    assert_eq!(workspace.at, Address::Workspace { name: "ws".into() });
    assert_eq!(workspace.field, HitField::Summary);
    let conversation = one(
        json!({ "at": "conversation", "workspace": "ws", "agent": "c-1",
                                   "field": "text", "offset": 12, "excerpt": "the gate" }),
    );
    assert_eq!(
        conversation.at,
        Address::Conversation {
            workspace: "ws".into(),
            agent: "c-1".into()
        }
    );
    assert_eq!(conversation.field, HitField::Text);
    assert_eq!(
        (conversation.offset, conversation.excerpt),
        (12, "the gate".to_owned())
    );
}

/// **An answer that found nothing is still an answer** (upstream bl-648a):
/// the needle rides back, so "nothing matched" is not read as "no search".
#[test]
fn the_empty_answer_carries_its_own_question() {
    let found = found_of(&envelope(json!([]), json!([]))).unwrap();
    assert_eq!(found.needle, "gate");
    assert!(found.hits.is_empty());
    assert!(found.unreadable.is_empty());
}

/// The unreadable half is carried, never swallowed: a corner of the world
/// that could not be read shrinks the corpus, it does not fail the search.
#[test]
fn what_could_not_be_read_rides_back_with_the_hits() {
    let found = found_of(&envelope(json!([]), json!(["p: balls unlistable"]))).unwrap();
    assert_eq!(found.unreadable, ["p: balls unlistable"]);
}

/// The three tier words, read back off the variants that own them.
#[test]
fn every_tier_answers_the_word_it_is_read_by() {
    assert_eq!(
        HitField::ALL.map(HitField::word),
        ["name", "summary", "text"]
    );
}

#[test]
fn refusals_name_the_shape() {
    let sentence = |rows: Value| found_of(&envelope(rows, json!([]))).unwrap_err();
    assert_eq!(sentence(json!([3])), "search: hit is not an object");
    assert_eq!(
        sentence(json!([{ "at": "elsewhere" }])),
        "search: hit at unknown address \"elsewhere\""
    );
    assert_eq!(
        sentence(json!([{ "at": "workspace", "workspace": "ws",
                          "field": "shouted", "offset": 0, "excerpt": "x" }])),
        "search: hit in unknown field \"shouted\""
    );
    assert_eq!(
        sentence(json!([{ "at": "workspace", "workspace": "ws",
                          "field": "name", "offset": -1, "excerpt": "x" }])),
        "search: missing or non-integer field \"offset\""
    );
    assert_eq!(
        sentence(json!([{ "at": "ball", "project": "p", "id": "bl-1",
                          "field": "name", "offset": 0 }])),
        "search: missing or non-string field \"excerpt\""
    );
    assert_eq!(
        sentence(json!([{ "at": "conversation", "workspace": "ws",
                          "field": "name", "offset": 0, "excerpt": "x" }])),
        "search: missing or non-string field \"agent\""
    );
    assert_eq!(
        sentence(json!([{ "at": "ball", "id": "bl-1",
                          "field": "name", "offset": 0, "excerpt": "x" }])),
        "search: missing or non-string field \"project\""
    );
    assert_eq!(
        found_of(&envelope(json!([]), json!([7]))).unwrap_err(),
        "search: non-string in \"unreadable\""
    );
    let no_needle = json!({ "ok": true, "kind": "search", "rows": [], "unreadable": [] });
    assert_eq!(
        found_of(no_needle.as_object().unwrap()).unwrap_err(),
        "search: missing or non-string field \"needle\""
    );
    let no_rows = json!({ "ok": true, "kind": "search", "needle": "g", "unreadable": [] });
    assert_eq!(
        found_of(no_rows.as_object().unwrap()).unwrap_err(),
        "search: missing or non-array field \"rows\""
    );
    let no_unreadable = json!({ "ok": true, "kind": "search", "needle": "g", "rows": [] });
    assert_eq!(
        found_of(no_unreadable.as_object().unwrap()).unwrap_err(),
        "search: missing or non-array field \"unreadable\""
    );
}
