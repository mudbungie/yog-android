//! The interface tools' pure half: the advertised elements, the argument
//! reading, and the two-line protocol the platform service speaks. The JNI
//! itself is the device's to answer for; everything up to it is asserted
//! here, which is why the seam sits where it does.

use super::{REFUSED, SHOT_NAME, absent, answer, run, tools};
use crate::tools::BAD_INPUT;
use serde_json::json;

fn args(v: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    v.as_object().cloned().unwrap_or_default()
}

#[test]
fn the_table_names_every_interface_tool_and_says_what_it_needs() {
    let set = tools();
    let names: Vec<&str> = set.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(
        names,
        ["ui_read", "ui_tap", "ui_type", "ui_key", "screenshot"]
    );
    for tool in &set {
        // The refusal a disabled service earns is the operator's fix, so the
        // description must say the service is needed — a model that reads
        // "requires" acts on the refusal instead of retrying blindly.
        assert!(
            tool.description.contains("accessibility service"),
            "{} does not say what it needs",
            tool.name
        );
        assert_eq!(tool.input_schema["type"], "object");
    }
}

#[test]
fn the_two_line_protocol_splits_into_the_captures_three_facts() {
    let ok = answer("ok\nthe payload\nsecond line");
    assert_eq!(ok.stdout, "the payload\nsecond line");
    assert_eq!(ok.stderr, "");
    assert_eq!(ok.exit_code, 0);
    let err = answer("err\nthe service is not enabled");
    assert_eq!(err.stdout, "");
    assert_eq!(err.stderr, "the service is not enabled\n");
    assert_eq!(err.exit_code, REFUSED);
    // An empty payload is an ordinary answer, not a malformed one.
    assert_eq!(answer("ok\n").stdout, "");
}

#[test]
fn an_answer_in_no_protocol_at_all_says_so_rather_than_guessing() {
    for reply in ["", "ok", "what", "\n"] {
        let capture = answer(reply);
        assert_eq!(capture.exit_code, REFUSED, "for {reply:?}");
        assert!(
            capture.stderr.contains("unreadable"),
            "for {reply:?}: {}",
            capture.stderr
        );
    }
}

#[test]
fn a_tap_that_names_nowhere_to_land_is_a_mis_call() {
    let capture = run("ui_tap", &args(json!({})), "/tmp");
    assert_eq!(capture.exit_code, BAD_INPUT);
    assert_eq!(
        capture.stderr,
        "state either \"text\", or both \"x\" and \"y\" in screen pixels\n"
    );
    // Half a coordinate is the same mis-call.
    assert_eq!(
        run("ui_tap", &args(json!({ "x": 10 })), "/tmp").exit_code,
        BAD_INPUT
    );
    // A coordinate no display can hold is not a coordinate.
    assert_eq!(
        run(
            "ui_tap",
            &args(json!({ "x": 10_i64.pow(18), "y": 4 })),
            "/tmp"
        )
        .exit_code,
        BAD_INPUT
    );
}

#[test]
fn the_typing_and_key_tools_read_their_one_argument_strictly() {
    let no_text = run("ui_type", &args(json!({})), "/tmp");
    assert_eq!(no_text.exit_code, BAD_INPUT);
    assert_eq!(no_text.stderr, "missing or non-string argument \"text\"\n");
    let no_key = run("ui_key", &args(json!({ "key": 4 })), "/tmp");
    assert_eq!(no_key.exit_code, BAD_INPUT);
    assert_eq!(no_key.stderr, "missing or non-string argument \"key\"\n");
}

#[test]
fn a_build_with_no_android_says_so_for_every_tool_that_reaches_the_bridge() {
    // The host suite is such a build, so this is what the whole family
    // answers here — and the arm is real: a tool that silently did nothing
    // off-device would be worse than one that says where it lives.
    for (tool, input) in [
        ("ui_read", json!({})),
        ("ui_tap", json!({ "x": 1, "y": 2 })),
        ("ui_tap", json!({ "text": "Send" })),
        ("ui_type", json!({ "text": "hello" })),
        ("ui_key", json!({ "key": "back" })),
        ("screenshot", json!({ "path": "/tmp/x.png" })),
        ("screenshot", json!({})),
    ] {
        let capture = run(tool, &args(input), "/tmp");
        assert_eq!(capture.exit_code, REFUSED, "for {tool}");
        assert_eq!(
            capture.stderr,
            "this build has no Android interface to read: the tool exists only on the device\n",
            "for {tool}"
        );
    }
    assert!(absent().starts_with("err\n"));
}

#[test]
fn a_screenshot_with_no_path_lands_in_the_apps_own_storage() {
    // The path is built before the bridge is reached, so the default is
    // observable here even though the capture is the absent-build refusal.
    assert_eq!(SHOT_NAME, "screenshot.png");
    let capture = run("screenshot", &args(json!({})), "/data/data/app/files");
    assert_eq!(capture.exit_code, REFUSED);
}
