use super::{CAP, Mirror, Move, band, read};
use crate::theme::TOUCH;

#[test]
fn an_answer_carries_focus_a_height_and_the_text() {
    let said = read("ok\n1\n96\nhello").expect("a well-formed answer reads");
    assert!(said.focused);
    assert_eq!(said.high, 96);
    assert_eq!(said.text, "hello");
}

#[test]
fn an_unfocused_field_says_so() {
    let said = read("ok\n0\n48\n").expect("an empty field reads");
    assert!(!said.focused);
    assert_eq!(said.text, "");
}

#[test]
fn the_text_keeps_its_own_lines() {
    let said = read("ok\n1\n96\none\ntwo\n").expect("a multi-line draft reads");
    assert_eq!(said.text, "one\ntwo\n");
}

#[test]
fn a_missing_text_line_is_an_empty_draft() {
    let said = read("ok\n1\n48").expect("a three-line answer reads");
    assert_eq!(said.text, "");
}

#[test]
fn a_refusal_is_the_bridge_s_own_sentence() {
    let Err(why) = read("err\nthe composer field failed: no activity") else {
        panic!("an err answer must refuse");
    };
    assert_eq!(why, "the composer field failed: no activity");
}

#[test]
fn a_shape_this_build_cannot_read_never_echoes_the_draft() {
    for answer in [
        "",
        "yes\n1\n48\nsecret",
        "ok\nmaybe\n48\nsecret",
        "ok\n1\nhigh\nsecret",
    ] {
        let Err(why) = read(answer) else {
            panic!("a malformed answer must refuse");
        };
        assert_eq!(
            why,
            "the composer field answered a shape this build cannot read"
        );
        assert!(!why.contains("secret"));
    }
}

#[test]
fn the_band_is_the_field_s_own_measure_between_the_floor_and_the_cap() {
    assert!((band(96, 2.0, true) - 48.0).abs() < f32::EPSILON);
    assert!((band(0, 2.0, true) - TOUCH).abs() < f32::EPSILON);
    assert!((band(10_000, 2.0, true) - CAP).abs() < f32::EPSILON);
}

#[test]
fn no_scale_yet_is_the_resting_height() {
    assert!((band(600, 0.0, true) - TOUCH).abs() < f32::EPSILON);
    assert!((band(600, -1.0, true) - TOUCH).abs() < f32::EPSILON);
}

/// **A field that does not hold the caret rests** (bl-0691): the band it
/// would have grown to is the transcript's height on the screen the
/// transcript is the point of, and the draft is all still there — one tap
/// away, at whatever height its own text earns.
#[test]
fn an_unfocused_field_rests_at_the_touch_floor_however_long_its_draft() {
    assert!((band(600, 2.0, false) - TOUCH).abs() < f32::EPSILON);
    assert!((band(10_000, 2.0, false) - TOUCH).abs() < f32::EPSILON);
    assert!(
        band(600, 2.0, true) > band(600, 2.0, false),
        "and the caret is what grows it"
    );
}

#[test]
fn the_field_moving_is_adopted() {
    let mut mirror = Mirror::default();
    assert_eq!(mirror.step("hi", ""), Move::Adopt("hi".to_owned()));
    assert_eq!(mirror.step("hi", "hi"), Move::Still);
}

#[test]
fn this_side_moving_is_pushed() {
    let mut mirror = Mirror::default();
    assert_eq!(mirror.step("", ""), Move::Still);
    assert_eq!(
        mirror.step("", "given back"),
        Move::Push("given back".to_owned())
    );
    assert_eq!(mirror.step("given back", "given back"), Move::Still);
}

#[test]
fn the_field_wins_a_frame_in_which_both_moved() {
    let mut mirror = Mirror::default();
    assert_eq!(
        mirror.step("typed", "spent"),
        Move::Adopt("typed".to_owned())
    );
}

#[test]
fn a_send_records_what_it_handed_over() {
    let mut mirror = Mirror::default();
    assert_eq!(mirror.step("sent", ""), Move::Adopt("sent".to_owned()));
    mirror.handed("");
    assert_eq!(mirror.step("", ""), Move::Still);
}
