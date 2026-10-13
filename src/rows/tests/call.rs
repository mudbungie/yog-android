//! **A call and its result, met on one row** (DESIGN §22 item 4, lernie
//! bl-eb4e): the head a call wears, the result as its payload, and the result
//! that stands alone when its call is not on the read.

use super::{call, go, model, result};
use crate::rows::{Kind, Tone};

/// **A call its result has met is one row** (§22 item 4, lernie bl-eb4e):
/// the call's head, the result as its payload, and no row of the result's own.
#[test]
fn a_retired_tool_call_drops_the_word_and_the_pulse() {
    let entries = [
        model("001", vec![call("t1", "Read", "{\"path\":\"a.rs\"}")]),
        result("002", "t1", "line one\nline two", false),
    ];
    let rows = go(&entries);
    assert_eq!(rows.len(), 1, "the result stands on its call's row");
    assert_eq!(rows[0].prefix, "Read(\"a.rs\")");
    assert_eq!(rows[0].preview, "line one");
    assert_eq!(rows[0].body, "line one\nline two");
    assert_eq!(rows[0].tone, Tone::Plain);
    assert_eq!(rows[0].kind, Kind::Answered);
    assert!(!rows[0].expanded, "folded to its head");

    let failed = [
        model("001", vec![call("t1", "Read", "")]),
        result("002", "t1", "boom", true),
    ];
    let rows = go(&failed);
    assert_eq!(rows[0].prefix, "Read");
    assert_eq!(rows[0].tone, Tone::Bad);
}

/// **A call's head is the desktop's**: a JSON object's first value, a
/// non-object's own text, its first line elided at sixty characters.
#[test]
fn a_call_head_reads_its_first_argument() {
    let head = |input: &str| {
        go(&[model("001", vec![call("t1", "bash", input)])])[0]
            .prefix
            .clone()
    };
    assert_eq!(
        head("{\"command\":\"ls -la\"}"),
        "bash(\"ls -la\") — running"
    );
    assert_eq!(head("[1,2]"), "bash([1,2]) — running");
    assert_eq!(head("plain\nmore"), "bash(plain…) — running");
    assert_eq!(head("{}"), "bash() — running");
    let long = "x".repeat(70);
    assert_eq!(head(&long), format!("bash({}…) — running", "x".repeat(60)));
}

#[test]
fn only_a_matching_id_retires_a_call() {
    let entries = [
        model("001", vec![call("t1", "Read", "{}")]),
        result("002", "t2", "ok", false),
    ];
    assert_eq!(go(&entries)[0].prefix, "Read() — running");
    assert_eq!(
        go(&entries)[1].kind,
        Kind::Result,
        "an orphan result stands alone"
    );
}
