//! The paper tools' pure half: the advertised elements, what each one
//! promises the model about its own price, and the argument reading. The
//! platform calls are the device's to answer for — everything up to them is
//! asserted here, which is why the seam sits where it does.

use super::{absent, run, tools};
use crate::tools::BAD_INPUT;
use crate::tools::bridged::REFUSED;
use serde_json::json;

fn args(v: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    v.as_object().cloned().unwrap_or_default()
}

#[test]
fn the_table_names_every_paper_tool() {
    let set = tools();
    let names: Vec<&str> = set.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["device", "clipboard_set", "notify", "open"]);
    for tool in &set {
        assert_eq!(tool.input_schema["type"], "object");
        assert!(!tool.description.is_empty(), "{} says nothing", tool.name);
    }
}

/// **The containment rule, asserted** (DESIGN §6, §16.1): the description is
/// the one text a model reads before it spends a call, so each tool's own
/// price — the permission, the foreground fact, the platform's own refusal —
/// is written there and not only in the refusal that comes back. A tool whose
/// description promised more than the platform allows is the decoy shape
/// bl-5710 named.
#[test]
fn every_tool_states_its_own_price_where_the_model_will_read_it() {
    let said = |name: &str| {
        tools()
            .into_iter()
            .find(|t| t.name == name)
            .map(|t| t.description)
            .unwrap_or_default()
    };
    assert!(said("device").contains("no runtime permission"));
    // The refused shape is stated where the model would otherwise ask for it.
    assert!(said("clipboard_set").contains("no tool that READS"));
    assert!(said("clipboard_set").contains("clears the clipboard"));
    assert!(said("notify").contains("notification permission"));
    assert!(said("notify").contains("not the seat's own attention machinery"));
    assert!(said("open").contains("not in front"));
    assert!(said("open").contains("no run-any-intent tool"));
}

#[test]
fn the_writing_tools_read_their_arguments_strictly() {
    let no_text = run("clipboard_set", &args(json!({})));
    assert_eq!(no_text.exit_code, BAD_INPUT);
    assert_eq!(no_text.stderr, "missing or non-string argument \"text\"\n");
    let no_title = run("notify", &args(json!({ "text": "body only" })));
    assert_eq!(no_title.exit_code, BAD_INPUT);
    assert_eq!(
        no_title.stderr,
        "missing or non-string argument \"title\"\n"
    );
    // A title that is not a string is the same mis-call as no title at all.
    assert_eq!(
        run("notify", &args(json!({ "title": 7 }))).exit_code,
        BAD_INPUT
    );
}

#[test]
fn an_open_that_names_nothing_to_open_is_a_mis_call() {
    let capture = run("open", &args(json!({})));
    assert_eq!(capture.exit_code, BAD_INPUT);
    assert_eq!(
        capture.stderr,
        "state either \"url\", the thing to open, or \"text\", the text to share\n"
    );
    // A url that is not a string names nothing either.
    assert_eq!(run("open", &args(json!({ "url": 3 }))).exit_code, BAD_INPUT);
}

#[test]
fn a_build_with_no_android_says_so_for_every_paper_tool() {
    // The host suite is such a build, so this is what the whole family
    // answers here — and every dispatch arm is reached on the way, including
    // the optional body, the url and the text halves of `open`.
    for (tool, input) in [
        ("device", json!({})),
        ("clipboard_set", json!({ "text": "copy me" })),
        (
            "notify",
            json!({ "title": "green", "text": "the build passed" }),
        ),
        ("notify", json!({ "title": "green" })),
        ("open", json!({ "url": "https://example.org" })),
        ("open", json!({ "text": "share me" })),
        // Both stated: the url is what opens, as the description says.
        (
            "open",
            json!({ "url": "https://example.org", "text": "ignored" }),
        ),
    ] {
        let capture = run(tool, &args(input));
        assert_eq!(capture.exit_code, REFUSED, "for {tool}");
        assert_eq!(
            capture.stderr,
            "this build has no Android to ask: the tool exists only on the device\n",
            "for {tool}"
        );
    }
    assert!(absent().starts_with("err\n"));
}
