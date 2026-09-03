//! The judgement's own tests. Every input is a string, so the whole gate is
//! host-testable without an emulator; what the device adds is the inventory,
//! and `tests/parity.rs` is where that arrives.

use std::fmt::Write as _;

use super::{exempt, judge, roster, tag, tags};

/// A help reply carrying exactly these `(verb, surface)` rows.
fn help(rows: &[(&str, &str)]) -> String {
    let rows: Vec<String> = rows
        .iter()
        .map(|(verb, surface)| {
            format!(r#"{{"verb":"{verb}","surface":"{surface}","usage":"/{verb}"}}"#)
        })
        .collect();
    format!(
        r#"{{"frames":[{{"kind":"help","rows":[{}]}}]}}"#,
        rows.join(",")
    )
}

/// A dump carrying one tagged node per op, in the shape uiautomator writes.
fn dump(ops: &[&str]) -> String {
    let mut out = String::new();
    for op in ops {
        let _ = writeln!(out, r#"<node content-desc="send {}" />"#, tag(op));
    }
    out
}

#[test]
fn the_tag_is_the_op_under_the_reserved_prefix() {
    assert_eq!(tag("message"), "act:message");
}

#[test]
fn the_inventory_file_is_one_tag_a_line() {
    let ops = ["message".to_owned(), "stop".to_owned()]
        .into_iter()
        .collect();
    assert_eq!(super::inventory(&ops), "act:message\nact:stop\n");
}

#[test]
fn the_roster_splits_control_from_machine() {
    let read = roster::read(&help(&[("message", "control"), ("invoke", "machine")]))
        .expect("the roster reads");
    assert_eq!(read.every.len(), 2);
    assert!(read.control.contains("message"));
    assert!(!read.control.contains("invoke"));
}

#[test]
fn a_roster_that_is_not_json_is_refused() {
    let why = roster::read("{").expect_err("not JSON");
    assert!(why.contains("not JSON"), "{why}");
}

#[test]
fn a_roster_with_no_rows_is_refused() {
    let why = roster::read(r#"{"frames":[]}"#).expect_err("no rows");
    assert!(why.contains("no frames[0].rows"), "{why}");
}

#[test]
fn a_row_with_no_verb_is_refused() {
    let why =
        roster::read(r#"{"frames":[{"rows":[{"surface":"control"}]}]}"#).expect_err("no verb");
    assert!(why.contains("states no verb"), "{why}");
}

#[test]
fn a_row_with_no_surface_names_the_re_vendor() {
    let why = roster::read(r#"{"frames":[{"rows":[{"verb":"stop"}]}]}"#).expect_err("no surface");
    assert!(why.contains("re-vendor"), "{why}");
}

#[test]
fn a_third_classification_is_refused_rather_than_ignored() {
    let why = roster::read(&help(&[("stop", "advisory")])).expect_err("unknown class");
    assert!(why.contains("neither"), "{why}");
}

#[test]
fn the_exemption_file_reads_lines_and_skips_comments() {
    let rows = exempt::read("# a note\n\nstop = \"unbuilt, bl-1234\"\n").expect("reads");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].op, "stop");
    assert_eq!(rows[0].reason, "unbuilt, bl-1234");
}

#[test]
fn a_section_sign_is_a_citation_too() {
    let rows = exempt::read("stop = \"ruled out by DESIGN §8\"\n").expect("reads");
    assert_eq!(rows.len(), 1);
}

#[test]
fn a_line_that_is_not_a_pair_is_refused() {
    let why = exempt::read("stop\n").expect_err("no =");
    assert!(why.contains("is not `op"), "{why}");
}

#[test]
fn an_unquoted_reason_is_refused() {
    let why = exempt::read("stop = bl-1234\n").expect_err("unquoted");
    assert!(why.contains("double-quoted"), "{why}");
}

#[test]
fn an_op_that_is_not_a_token_is_refused() {
    let why = exempt::read("Stop = \"bl-1234\"\n").expect_err("not a token");
    assert!(why.contains("not an op token"), "{why}");
    let why = exempt::read(" = \"bl-1234\"\n").expect_err("empty");
    assert!(why.contains("not an op token"), "{why}");
}

#[test]
fn a_reason_that_cites_nothing_is_refused() {
    let why = exempt::read("stop = \"not yet\"\n").expect_err("no citation");
    assert!(why.contains("cites nothing"), "{why}");
    // `bl-` with too little after it, and with a non-hex quad, are both
    // uncitations: the shape is the check.
    let why = exempt::read("stop = \"bl-12\"\n").expect_err("short id");
    assert!(why.contains("cites nothing"), "{why}");
    let why = exempt::read("stop = \"bl-zzzz\"\n").expect_err("not hex");
    assert!(why.contains("cites nothing"), "{why}");
}

#[test]
fn one_op_gets_one_line() {
    let why = exempt::read("stop = \"bl-1234\"\nstop = \"bl-5678\"\n").expect_err("twice");
    assert!(why.contains("twice"), "{why}");
}

#[test]
fn tags_are_found_anywhere_in_the_bytes() {
    let found = tags::found(&dump(&["message", "clear-trail"]));
    assert!(found.contains("message"), "{found:?}");
    assert!(found.contains("clear-trail"), "{found:?}");
    assert_eq!(found.len(), 2);
}

#[test]
fn a_bare_prefix_yields_the_empty_token() {
    assert!(tags::found("act:").contains(""));
    assert!(tags::found("nothing here").is_empty());
}

#[test]
fn a_walk_that_reaches_every_control_passes() {
    let judged = judge(
        &help(&[("message", "control"), ("invoke", "machine")]),
        "",
        &dump(&["message"]),
    );
    assert!(judged.failures.is_empty(), "{:?}", judged.failures);
    assert!(
        judged
            .report
            .contains("2 ops in the corpus, 1 classed `control`")
    );
}

#[test]
fn an_unreached_control_fails_unless_it_is_cited() {
    let roster = help(&[("message", "control"), ("nudge", "control")]);
    let judged = judge(&roster, "", &dump(&["message"]));
    assert_eq!(judged.failures.len(), 1);
    assert!(judged.failures[0].starts_with("nudge: classed `control`"));
    // The same tree with the absence recorded is green, and the report says
    // so on every run.
    let judged = judge(
        &roster,
        "nudge = \"unbuilt, bl-1234\"\n",
        &dump(&["message"]),
    );
    assert!(judged.failures.is_empty(), "{:?}", judged.failures);
    assert!(judged.report.contains("nudge"), "{}", judged.report);
    assert!(judged.report.contains("1 exempt"), "{}", judged.report);
}

#[test]
fn a_tag_naming_no_op_fails() {
    let judged = judge(
        &help(&[("message", "control")]),
        "",
        &dump(&["message", "mesage"]),
    );
    assert_eq!(judged.failures.len(), 1);
    assert!(
        judged.failures[0].contains("act:mesage"),
        "{:?}",
        judged.failures
    );
}

#[test]
fn a_rotted_exemption_fails() {
    let judged = judge(
        &help(&[("message", "control"), ("invoke", "machine")]),
        "invoke = \"machine, bl-1234\"\n",
        &dump(&["message"]),
    );
    assert_eq!(judged.failures.len(), 1);
    assert!(
        judged.failures[0].contains("does not class it"),
        "{:?}",
        judged.failures
    );
}

#[test]
fn a_stale_exemption_fails() {
    let judged = judge(
        &help(&[("message", "control")]),
        "message = \"unbuilt, bl-1234\"\n",
        &dump(&["message"]),
    );
    assert_eq!(judged.failures.len(), 1);
    assert!(
        judged.failures[0].contains("drop the line"),
        "{:?}",
        judged.failures
    );
}

#[test]
fn a_file_that_will_not_parse_refuses_the_whole_judgement() {
    let judged = judge("{", "", "");
    assert_eq!(judged.failures.len(), 1);
    assert!(judged.report.is_empty());
    let judged = judge(&help(&[("stop", "control")]), "stop\n", "");
    assert_eq!(judged.failures.len(), 1);
    assert!(judged.report.is_empty());
}

/// **This tree's own two files, judged for everything that does not need a
/// device.** The inventory arrives only after a walk, so coverage and
/// staleness are `tests/parity.rs`'s; that every exemption parses, cites, and
/// still names an op the engine classes `control` is answerable here — and so
/// it is answered on every `make check`, where a re-vendor that retires an op
/// reddens immediately instead of waiting for an emulator.
#[test]
fn the_shipped_exemptions_are_readable_and_unrotted() {
    let roster = roster::read(include_str!("../../corpus/reply/help.json"))
        .expect("the vendored roster reads");
    let rows = exempt::read(include_str!("../../parity.toml")).expect("parity.toml reads");
    for row in &rows {
        assert!(
            roster.control.contains(&row.op),
            "parity.toml exempts `{}`, which the roster does not class control",
            row.op
        );
    }
}
