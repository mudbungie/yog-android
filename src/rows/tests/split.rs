//! Keys, the preview/body split, and the one row that states its own size.

use super::{delivered, go, model, result, text, thought};

/// One character past the cap, so the clip is provable rather than assumed.
const OVER_CAP: usize = 161;

#[test]
fn a_key_is_the_entry_name_and_the_block_ordinal() {
    // The turn ends on machinery, so nothing rolls up and both rows survive.
    let rows = go(&[model("004-turn.json", vec![text("said"), thought("hmm")])]);
    assert_eq!(rows[0].key, "tx/004-turn.json#0");
    assert_eq!(rows[1].key, "tx/004-turn.json#1");
}

#[test]
fn a_payload_that_fits_one_line_has_no_body_to_fold() {
    let rows = go(&[delivered("001", "user", "short")]);
    assert_eq!(rows[0].preview, "short");
    assert_eq!(rows[0].body, "");
}

#[test]
fn a_multi_line_payload_previews_its_first_line_and_folds_the_whole() {
    let rows = go(&[delivered("001", "user", "first\nsecond\nthird")]);
    assert_eq!(rows[0].preview, "first");
    assert_eq!(rows[0].body, "first\nsecond\nthird");
}

#[test]
fn a_long_first_line_clips_at_the_cap_in_characters_not_bytes() {
    let line: String = "é".repeat(OVER_CAP);
    let rows = go(&[delivered("001", "user", &line)]);
    assert_eq!(rows[0].preview.chars().count(), OVER_CAP);
    assert!(rows[0].preview.ends_with('…'));
    assert_eq!(rows[0].preview.chars().filter(|c| *c == 'é').count(), 160);
    assert_eq!(rows[0].body, line);
}

#[test]
fn a_line_exactly_at_the_cap_is_not_clipped() {
    let line: String = "é".repeat(OVER_CAP - 1);
    let rows = go(&[delivered("001", "user", &line)]);
    assert_eq!(rows[0].preview, line);
    assert_eq!(rows[0].body, "");
}

#[test]
fn an_empty_payload_previews_as_nothing() {
    let rows = go(&[model("001", vec![text("")])]);
    assert_eq!(rows[0].preview, "");
    assert_eq!(rows[0].body, "");
}

#[test]
fn only_a_foldable_tool_result_states_its_size() {
    let small = go(&[result("001", "t1", "ok", false)]);
    assert_eq!(small[0].prefix, "✔ tool result — ok");

    let big = go(&[result("001", "t1", "a\nb\nc", true)]);
    assert_eq!(big[0].prefix, "✖ tool result — error · 5 chars");
}

#[test]
fn the_size_hint_counts_characters_not_bytes() {
    let rows = go(&[result("001", "t1", "é\né", false)]);
    assert_eq!(rows[0].prefix, "✔ tool result — ok · 3 chars");
}

#[test]
fn no_other_row_class_carries_a_size_hint() {
    let rows = go(&[delivered("001", "user", "a\nb\nc")]);
    assert_eq!(rows[0].prefix, "user:");
}
