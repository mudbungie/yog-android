//! The `http` tool's pure half: the advertised element, the argument reading,
//! the containment rule on a saved path, and the elision. The request itself
//! is the device's to answer for — every call below lands on the host build's
//! absent-bridge arm, which is the honest thing a box with no Android says.

use serde_json::{Value, json};

use super::{NAME, elide, run, tool};
use crate::tools::{BAD_INPUT, bridged::REFUSED};

/// The app's own storage, as this suite spells it. Nothing here writes a
/// file: the save path is *decided* on this side and *used* on the far one.
const STORAGE: &str = "/data/user/0/dev.yog/files";

fn call(input: &Value) -> crate::codec::Capture {
    let o = input.as_object().expect("an object");
    run(o, STORAGE)
}

#[test]
fn the_advertised_element_states_the_platform_stack_the_cut_and_the_saved_path() {
    let element = tool();
    assert_eq!(element.name, NAME);
    assert!(!element.subject_cwd);
    let said = element.description;
    assert!(said.contains("device's own networking stack"), "{said}");
    assert!(said.contains("CA store"), "{said}");
    assert!(said.contains("`open` tool"), "{said}");
    assert!(said.contains("no proxy setting"), "{said}");
    let schema = element.input_schema;
    assert_eq!(schema["required"], json!(["url"]));
    for field in ["url", "method", "headers", "body", "save_to", "limit"] {
        assert!(
            schema["properties"][field].is_object(),
            "no {field} in the schema"
        );
    }
}

#[test]
fn a_call_with_no_url_is_a_mis_call_and_names_the_field() {
    let capture = call(&json!({}));
    assert_eq!(capture.exit_code, BAD_INPUT);
    assert_eq!(capture.stderr, "missing or non-string argument \"url\"\n");
}

#[test]
fn a_method_is_letters_and_is_upper_cased() {
    // The refusals: not a string, empty, and one carrying anything but
    // letters — which is how a second line would be written into a request.
    for (method, said) in [
        (json!(7), "\"method\" is not a string"),
        (json!(""), "is not an HTTP method"),
        (json!("GET /x HTTP/1.1"), "is not an HTTP method"),
    ] {
        let capture = call(&json!({ "url": "https://example.com", "method": method }));
        assert_eq!(capture.exit_code, BAD_INPUT, "for {method}");
        assert!(
            capture.stderr.contains(said),
            "for {method}: {}",
            capture.stderr
        );
    }
    // And the ordinary arm: a lower-case method is read, and reaches the
    // bridge — which on this build is absent, which is the answer.
    let capture = call(&json!({ "url": "https://example.com", "method": "post" }));
    assert_eq!(capture.exit_code, REFUSED);
    assert!(
        capture.stderr.contains("no Android network stack"),
        "{}",
        capture.stderr
    );
}

#[test]
fn headers_are_names_and_values_and_a_line_break_in_either_is_refused() {
    for (headers, said) in [
        (json!("Accept: text/html"), "is not an object"),
        (json!({ "Accept": 7 }), "has a non-string value"),
        (json!({ "Ac\ncept": "x" }), "carries a line break"),
        (
            json!({ "Accept": "text/html\r\nX: y" }),
            "carries a line break",
        ),
    ] {
        let capture = call(&json!({ "url": "https://example.com", "headers": headers }));
        assert_eq!(capture.exit_code, BAD_INPUT, "for {headers}");
        assert!(
            capture.stderr.contains(said),
            "for {headers}: {}",
            capture.stderr
        );
    }
    let capture = call(&json!({ "url": "https://example.com",
                                "headers": { "Accept": "text/html" },
                                "body": "hello" }));
    assert_eq!(capture.exit_code, REFUSED);
}

#[test]
fn a_saved_path_is_contained_in_the_apps_own_storage() {
    for (save_to, said) in [
        (json!(7), "\"save_to\" is not a string"),
        (json!(""), "state a filename"),
        (
            json!("../../etc/passwd"),
            "walks out of the app's own storage",
        ),
        (
            json!("/sdcard/thing.apk"),
            "is not under this app's own storage",
        ),
    ] {
        let capture = call(&json!({ "url": "https://example.com", "save_to": save_to }));
        assert_eq!(capture.exit_code, BAD_INPUT, "for {save_to}");
        assert!(
            capture.stderr.contains(said),
            "for {save_to}: {}",
            capture.stderr
        );
    }
    // A bare name lands in the app's storage, and an absolute path already
    // under it stands: both reach the bridge rather than a refusal.
    for save_to in [
        json!("yog.apk"),
        json!("/data/user/0/dev.yog/files/yog.apk"),
    ] {
        let capture = call(&json!({ "url": "https://example.com", "save_to": save_to }));
        assert_eq!(capture.exit_code, REFUSED, "for {save_to}");
    }
}

#[test]
fn an_answer_longer_than_the_cap_keeps_its_head_and_its_tail_and_names_what_went() {
    let text: String = ('a'..='z').cycle().take(1000).collect();
    assert_eq!(elide(&text, 1000), text);
    assert_eq!(elide(&text, 1001), text);
    let cut = elide(&text, 100);
    let head: String = text.chars().take(75).collect();
    let tail: String = text.chars().skip(975).collect();
    assert!(cut.starts_with(&head), "{cut}");
    assert!(cut.ends_with(&tail), "{cut}");
    assert!(cut.contains("… 900 characters elided …"), "{cut}");
    // The cut counts CHARACTERS, not bytes: a multi-byte answer must not be
    // cut through the middle of one.
    let wide: String = "é".repeat(50);
    assert_eq!(elide(&wide, 8).chars().next(), Some('é'));
    // And a cap of one is a bound, not a panic.
    assert!(elide(&wide, 1).starts_with('é'));
}
