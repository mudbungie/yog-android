//! The idiom's grouping (DESIGN §22 item 4): which rows stand at the gutter,
//! which on a stem under a fold head, what parts them, and the glyph each
//! stands behind.

use std::collections::BTreeSet;

use super::{call, delivered, go, go_open, model, result, text, thought, wounded};
use crate::rows::{Before, Kind, Line, Part, glyph, lines, mark};

/// The lines' shape: what each is, what parts it from the one above, its glyph.
fn shape(lines: &[Line]) -> Vec<(Part, Before, String)> {
    lines
        .iter()
        .map(|line| (line.part, line.before, glyph(&line.row)))
        .collect()
}

/// One expected line.
fn t(part: Part, before: Before, glyph: &str) -> (Part, Before, String) {
    (part, before, glyph.to_owned())
}

fn none() -> BTreeSet<String> {
    BTreeSet::new()
}

#[test]
fn a_finished_turn_is_the_prompt_a_shut_trail_and_the_response() {
    let turn = [
        delivered("001", "user", "go"),
        model("002", vec![thought("hmm"), call("t1", "Read", "{}")]),
        result("003", "t1", "contents", false),
        model("004", vec![text("done")]),
    ];
    let out = lines(&go(&turn), &none());
    assert_eq!(
        shape(&out),
        [
            t(Part::Said, Before::Nothing, "›"),
            t(Part::Head, Before::Gap, "+"),
            t(Part::Said, Before::Gap, "¦"),
        ]
    );
    assert_eq!(out[1].row.prefix, "worked · 1 call · 1 note");
}

#[test]
fn an_open_rollup_stands_its_steps_and_remarks_on_one_stem() {
    let turn = [
        delivered("001", "user", "go"),
        model("002", vec![thought("hmm"), call("t1", "Read", "{}")]),
        result("003", "t1", "contents", false),
        model("004", vec![text("checking")]),
        model("005", vec![call("t2", "Bash", "{}")]),
        result("006", "t2", "boom", true),
        model("007", vec![text("done")]),
    ];
    let out = lines(&go_open(&turn), &none());
    let step = |last| (Part::Step { last }, Before::Nothing);
    let parts: Vec<_> = out.iter().map(|line| (line.part, line.before)).collect();
    assert_eq!(
        parts,
        [
            (Part::Said, Before::Nothing),
            (Part::Head, Before::Gap),
            step(false),
            step(false),
            step(false),
            step(true),
            (Part::Said, Before::Response),
        ]
    );
    assert_eq!(out[1].row.prefix, "worked · 2 calls · 2 notes");
    assert_eq!(glyph(&out[1].row), "−");
    assert_eq!(
        out[4].row.kind,
        Kind::Remark,
        "a remark is a step of the turn"
    );
    let glyphs: Vec<_> = out.iter().map(|line| glyph(&line.row)).collect();
    assert_eq!(glyphs, ["›", "−", "·", "•", "¦", "•", "¦"]);
    let marks: Vec<_> = out.iter().map(|line| mark(&line.row)).collect();
    let marks: Vec<Option<&str>> = marks.iter().map(Option::as_deref).collect();
    assert_eq!(marks, [None, None, None, Some("✔"), None, Some("✖"), None]);
}

#[test]
fn a_running_run_gets_its_own_head_open_until_shut() {
    let turn = [
        delivered("001", "user", "go"),
        model("002", vec![thought("hmm"), call("t1", "Read", "{}")]),
    ];
    let open = lines(&go(&turn), &none());
    assert_eq!(open.len(), 4);
    assert_eq!(open[1].row.key, "tx/002#trail");
    assert_eq!(open[1].row.prefix, "working · 1 call · 1 note");
    assert_eq!(open[1].row.kind, Kind::Trail);
    assert!(open[1].row.expanded);
    assert_eq!(open[3].part, Part::Step { last: true });

    let folds: BTreeSet<String> = ["tx/002#trail".to_string()].into_iter().collect();
    let shut = lines(&go(&turn), &folds);
    assert_eq!(shut.len(), 2, "a shut head leaves its steps out");
    assert_eq!(glyph(&shut[1].row), "+");
}

#[test]
fn a_hairline_parts_turns_and_a_blank_only_a_change_of_kind() {
    let talk = [
        delivered("001", "user", "one"),
        model("002", vec![text("a")]),
        delivered("003", "peer-x", "mail"),
        delivered("0035", "peer-y", "more mail"),
        delivered("004", "user", "two"),
        wounded("005", "refused", None, None),
    ];
    let out = lines(&go(&talk), &none());
    assert_eq!(
        shape(&out),
        [
            t(Part::Said, Before::Nothing, "›"),
            t(Part::Said, Before::Gap, "¦"),
            t(Part::Said, Before::Gap, "·"),
            t(Part::Said, Before::Nothing, "·"),
            t(Part::Said, Before::Parting, "›"),
            t(Part::Said, Before::Gap, "✖"),
        ]
    );
}

#[test]
fn an_orphan_result_stands_behind_how_it_ended() {
    let out = lines(
        &go(&[
            result("001", "t1", "fine", false),
            result("002", "t2", "boom", true),
        ]),
        &none(),
    );
    let glyphs: Vec<_> = out.iter().map(|line| glyph(&line.row)).collect();
    assert_eq!(glyphs, ["−", "✔", "✖"]);
    assert_eq!(out[0].row.prefix, "working");
    assert_eq!(mark(&out[1].row), None);
}

#[test]
fn an_empty_transcript_is_no_lines() {
    assert!(lines(&[], &none()).is_empty());
}

#[test]
fn the_first_prompt_after_a_trail_is_a_blank_not_a_hairline() {
    let out = lines(
        &go(&[
            model("001", vec![thought("a")]),
            delivered("002", "user", "hi"),
        ]),
        &none(),
    );
    assert_eq!(
        shape(&out),
        [
            t(Part::Head, Before::Nothing, "−"),
            t(Part::Step { last: true }, Before::Nothing, "·"),
            t(Part::Said, Before::Gap, "›"),
        ]
    );
}

#[test]
fn the_response_elbow_stands_only_under_an_open_working() {
    let turn = [
        delivered("001", "user", "go"),
        model("002", vec![thought("hmm"), call("t1", "Read", "{}")]),
        result("003", "t1", "contents", false),
        model("004", vec![text("done")]),
    ];
    let open = lines(&go_open(&turn), &none());
    assert_eq!(open.last().map(|line| line.before), Some(Before::Response));
    let shut = lines(&go(&turn), &none());
    assert_eq!(shut.last().map(|line| line.before), Some(Before::Gap));
}
