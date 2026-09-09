//! The six machinery shapes, read strictly — and the absences that are facts
//! rather than zeros: a conversation with no strip, a notch nothing can pin,
//! a log with no bytes, and a deposit whose envelope stated nothing.
//!
//! The corpus replay (`tests/conformance`) drives every shape's real frames;
//! what is asserted here is the readings those frames do not reach and the
//! refusals nobody would otherwise see.

use super::{Framing, Orphan};
use serde_json::{Value, json};

fn object(v: &Value) -> serde_json::Map<String, Value> {
    v.as_object().cloned().unwrap_or_default()
}

/// The least an `agent` answer can say and still be one.
fn bare() -> Value {
    json!({ "display": "Pennant", "root": "r-0", "state": "stopped",
            "present": false, "refused": false, "tip": "" })
}

#[test]
fn the_conversation_reads_what_it_states_and_nothing_it_does_not() {
    let head = super::agent_of(&object(&bare())).unwrap();
    assert_eq!(head.state, "stopped");
    assert_eq!(head.flight, "");
    assert!(head.marks.is_empty() && head.seats.is_empty());
    assert_eq!((head.usd.as_str(), head.strip.is_none()), ("", true));
    assert!(head.context.is_none() && head.failure.is_none());
}

#[test]
fn the_engines_own_renderings_cross_as_it_wrote_them() {
    let mut said = object(&bare());
    said.insert("flight".to_owned(), json!("tools"));
    said.insert("marks".to_owned(), json!(["held"]));
    said.insert(
        "strip".to_owned(),
        json!({ "class": "tools", "facts": "Bash · 5s" }),
    );
    said.insert(
        "seats".to_owned(),
        json!([{ "name": "kid", "doing": "idle" }]),
    );
    said.insert("spend".to_owned(), json!({ "usd": "$4.00" }));
    said.insert(
        "context".to_owned(),
        json!({ "model": "claude-x", "percent": 140 }),
    );
    let head = super::agent_of(&said).unwrap();
    assert_eq!(head.flight, "tools");
    assert_eq!(head.marks, vec!["held".to_owned()]);
    assert_eq!(head.strip.as_deref(), Some("Bash · 5s"));
    assert_eq!(
        head.seats.first().map(|seat| seat.doing.clone()),
        Some("idle".to_owned())
    );
    assert_eq!(head.usd, "$4.00");
    // Unclamped on purpose: a context that has outgrown its window reads as
    // what the engine said it is, never as a hundred.
    assert_eq!(head.context.map(|full| full.percent), Some(140));
}

#[test]
fn a_nested_object_that_is_not_one_refuses_naming_the_field() {
    let mut said = object(&bare());
    said.insert("strip".to_owned(), json!("tools"));
    assert_eq!(
        super::agent_of(&said).unwrap_err(),
        "field \"strip\" is not an object"
    );
    let mut said = object(&bare());
    said.insert("marks".to_owned(), json!([7]));
    assert_eq!(
        super::agent_of(&said).unwrap_err(),
        "field \"marks\": a non-string entry"
    );
}

#[test]
fn a_step_row_states_its_total_and_refuses_when_it_states_no_counters() {
    let rows = json!({ "rows": [{ "seq": "001", "framing": "complete", "wound": "none",
                                  "attempts": 1, "tokens": { "total": 9 } }],
                       "orphan": "none" });
    let steps = super::steps_of(&object(&rows)).unwrap();
    assert_eq!(steps.orphan, super::Orphan::None);
    assert_eq!(steps.rows.first().map(|row| row.tokens), Some(9));
    assert_eq!(
        steps.rows.first().map(|row| row.commit.clone()),
        Some(String::new())
    );
    let bare = json!({ "rows": [{ "seq": "001", "framing": "complete", "wound": "none",
                                  "attempts": 1 }], "orphan": "none" });
    assert_eq!(
        super::steps_of(&object(&bare)).unwrap_err(),
        "steps: a row states no tokens"
    );
}

/// **The four framing words, and the line each one labels a row with**
/// (PROTOCOL 18). The word is the engine's own — a second wording here would
/// be a second vocabulary for one that already has an authority — and the line
/// lives beside the decode because it joins six fields and two optional ones.
#[test]
fn every_framing_word_reads_back_and_labels_its_row() {
    use crate::codec::Framing;
    for (word, framing) in [
        ("complete", Framing::Complete),
        ("failed", Framing::Failed),
        ("killed", Framing::Killed),
        ("in_flight", Framing::InFlight),
    ] {
        let rows = json!({ "rows": [{ "seq": "007", "framing": word, "wound": "none",
                                      "attempts": 1, "commit": "abc",
                                      "tokens": { "total": 99 } }],
                           "orphan": "none" });
        let read = super::steps_of(&object(&rows)).unwrap();
        let row = read.rows.first().unwrap();
        assert_eq!(row.framing, framing);
        assert_eq!(row.framing.word(), word);
        assert_eq!(
            row.line(),
            format!("007 · {word} · none\n99 tokens · 1 attempt(s) · abc")
        );
    }
}

/// A wound with words says them, and a step that recorded no commit says
/// nothing about one rather than an empty separator.
#[test]
fn a_line_carries_the_wounds_reason_and_omits_a_commit_it_has_not_got() {
    let rows = json!({ "rows": [{ "seq": "003", "framing": "failed",
                                  "wound": "refused", "wound_reason": "no credential",
                                  "attempts": 2, "tokens": { "total": 4 } }],
                       "orphan": "none" });
    let read = super::steps_of(&object(&rows)).unwrap();
    assert_eq!(
        read.rows.first().unwrap().line(),
        "003 · failed · refused — no credential\n4 tokens · 2 attempt(s)"
    );
}

/// **A framing word this build has not heard of is said as unknown** (REMOTE
/// §3.2). It rode as a bare string while nothing branched on it; a screen
/// branches on it now, so reading an unknown word as the nearest known one
/// would paint a live step as a killed one — the defect PROTOCOL 18 exists to
/// end. The third answer keeps the census: the row reads, the label says the
/// word, and `crate::theme::framing` gives it resting ink.
#[test]
fn a_framing_token_this_build_has_not_heard_of_is_said_as_unknown() {
    let rows = json!({ "rows": [{ "seq": "001", "framing": "dawdling", "wound": "none",
                                  "attempts": 1, "tokens": { "total": 1 } }],
                       "orphan": "none" });
    let steps = super::steps_of(&object(&rows)).unwrap();
    let row = steps.rows.first().unwrap();
    assert_eq!(row.framing, Framing::Unknown("dawdling".to_owned()));
    assert_eq!(row.framing.word(), "unknown framing: dawdling");
    assert_eq!(row.seq, "001");
}

#[test]
fn an_orphan_token_this_build_has_not_heard_of_is_carried() {
    let said = json!({ "rows": [], "orphan": "sideways" });
    let steps = super::steps_of(&object(&said)).unwrap();
    assert_eq!(steps.orphan, Orphan::Unknown("sideways".to_owned()));
}

#[test]
fn a_binary_log_carries_its_class_and_no_bytes() {
    let said = json!({ "seq": "001", "meta": { "kind": "absent" },
                       "request": { "kind": "absent" }, "staging": { "kind": "absent" },
                       "response": [], "tools": [],
                       "stderr": { "kind": "binary", "size": 12 } });
    let step = super::step_of(&object(&said)).unwrap();
    assert_eq!(
        step.stderr.as_ref().map(|log| log.kind.clone()),
        Some("binary".to_owned())
    );
    assert_eq!(step.stderr.map(|log| log.text), Some(String::new()));
    assert!(step.driver.is_none());
}

#[test]
fn a_step_missing_one_of_its_record_files_refuses_naming_it() {
    let said = json!({ "seq": "001", "meta": { "kind": "absent" },
                       "request": { "kind": "absent" }, "response": [], "tools": [] });
    assert_eq!(
        super::step_of(&object(&said)).unwrap_err(),
        "step: no \"staging\""
    );
}

#[test]
fn a_notch_with_no_commit_is_unpinnable_and_says_so_by_saying_nothing() {
    let said = json!({ "rows": [{ "seq": "002", "budget": 120 }], "cards": [] });
    let rail = super::rail_of(&object(&said)).unwrap();
    assert_eq!(
        rail.notches.first().map(|notch| notch.short.clone()),
        Some(String::new())
    );
    assert!(rail.cards.is_empty());
}

#[test]
fn a_governing_config_that_follows_nothing_carries_the_count_that_held_it() {
    let said = json!({ "oid": "cc", "short_oid": "cccc", "follows": Value::Null,
                       "diverged_lineages": 2, "files": [] });
    let governing = super::governing_of(&object(&said)).unwrap();
    assert_eq!((governing.follows.clone(), governing.diverged), (None, 2));
    assert_eq!(governing.short_oid, "cccc");
}

#[test]
fn a_deposit_that_stated_nothing_is_absent_rather_than_empty() {
    let said = json!({ "rows": [{ "name": "raw.md", "deposit": { "body": "" } }] });
    let mail = super::mail(&object(&said)).unwrap();
    let row = mail.first().cloned().unwrap_or_else(|| unreachable!());
    assert_eq!(
        (row.from, row.deposited_at, row.epitaph),
        (None, None, None)
    );
    let bare = json!({ "rows": [{ "name": "raw.md" }] });
    assert_eq!(
        super::mail(&object(&bare)).unwrap_err(),
        "inbox: a row states no deposit"
    );
}

/// **A row is headed by the name where there is one** (PROTOCOL 17): the
/// display name, else the addressing id, else the file itself — the whole
/// fallback chain, asserted here rather than inside a screen no host test can
/// reach.
#[test]
fn a_deposit_is_headed_by_its_senders_name_then_its_id_then_its_file() {
    let said = json!({ "rows": [
        { "name": "001-user.md", "deposit": { "body": "hi", "from": "user",
                                              "from_name": "DulcetMongoose" } },
        { "name": "002-user.md", "deposit": { "body": "hi", "from": "user" } },
        { "name": "raw.md", "deposit": { "body": "" } }] });
    let mail = super::mail(&object(&said)).unwrap();
    let said: Vec<String> = mail.iter().map(super::Mail::speaker).collect();
    assert_eq!(said, ["DulcetMongoose", "user", "raw.md"]);
    assert_eq!(
        mail.first().and_then(|row| row.from_name.clone()),
        Some("DulcetMongoose".to_owned())
    );
}

#[test]
fn records_are_paintable_only_under_the_conversation_they_were_asked_at() {
    let records = super::Records {
        workspace: "ws".to_owned(),
        agent: "c-1".to_owned(),
        head: super::agent_of(&object(&bare())).unwrap(),
        steps: super::steps_of(&object(&json!({ "rows": [], "orphan": "none" }))).unwrap(),
        rail: super::rail_of(&object(&json!({ "rows": [], "cards": [] }))).unwrap(),
        governing: super::governing_of(&object(&json!({ "short_oid": "b", "follows": "default",
                                                        "diverged_lineages": 0, "files": [] })))
        .unwrap(),
        inbox: Vec::new(),
        lineages: Vec::new(),
        drilled: None,
        anchored: None,
    };
    assert!(records.about("ws", "c-1"));
    assert!(!records.about("ws", "c-2"));
    assert!(!records.about("other", "c-1"));
}
