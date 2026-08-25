//! What a rolled-up turn says it held — the aggregate sentence, counted from
//! committed bytes and worded for what those bytes do not know.

use serde_json::json;

use super::{call, go, metered, model, result, text, thought};

#[test]
fn a_full_census_names_every_term_and_sums_the_counters() {
    let entries = [
        metered(
            "001",
            vec![thought("a"), call("t1", "Read", "x")],
            json!({"input_tokens": 100, "output_tokens": 2000}),
        ),
        result("002", "t1", "ok", false),
        metered(
            "003",
            vec![call("t2", "Write", "y")],
            json!({"input_tokens": 40, "output_tokens": 1150}),
        ),
        result("004", "t2", "ok", false),
        model("005", vec![text("done")]),
    ];
    assert_eq!(
        go(&entries)[0].prefix,
        "⚙ 2 inference calls · 2 tool calls · 1 thinking block · 140 input tokens · 3150 output tokens"
    );
}

#[test]
fn a_turn_only_partly_metered_says_at_least() {
    let entries = [
        metered(
            "001",
            vec![thought("a"), call("t1", "Read", "x")],
            json!({"input_tokens": 100, "output_tokens": 2000}),
        ),
        result("002", "t1", "ok", false),
        model("003", vec![call("t2", "Write", "y")]),
        result("004", "t2", "ok", false),
        model("005", vec![text("done")]),
    ];
    assert_eq!(
        go(&entries)[0].prefix,
        "⚙ 2 inference calls · 2 tool calls · 1 thinking block · 100+ input tokens · 2000+ output tokens"
    );
}

#[test]
fn a_zero_term_is_left_unsaid_and_the_singular_has_no_s() {
    let entries = [
        model("001", vec![thought("a")]),
        model("002", vec![text("hi")]),
    ];
    assert_eq!(
        go(&entries)[0].prefix,
        "⚙ 1 inference call · 1 thinking block"
    );
}

#[test]
fn a_counter_at_zero_is_not_a_term() {
    let entries = [
        metered(
            "001",
            vec![thought("a")],
            json!({"cache_read": 0, "input_tokens": 5}),
        ),
        model("002", vec![text("hi")]),
    ];
    assert_eq!(
        go(&entries)[0].prefix,
        "⚙ 1 inference call · 1 thinking block · 5 input tokens"
    );
}

#[test]
fn a_counter_that_is_not_a_whole_number_is_not_read() {
    let entries = [
        metered(
            "001",
            vec![thought("a")],
            json!({"note": "n/a", "ratio": 1.5}),
        ),
        model("002", vec![text("hi")]),
    ];
    assert_eq!(
        go(&entries)[0].prefix,
        "⚙ 1 inference call · 1 thinking block"
    );
}
