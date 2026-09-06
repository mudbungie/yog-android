//! The sign-in family read strictly, and the fold a held lane spends.

use super::{LoginLine, LoginView, view};
use serde_json::{Value, json};

fn body(v: &Value) -> Result<LoginView, String> {
    view(v.as_object().unwrap())
}

fn said(err: bool, text: &str) -> LoginLine {
    LoginLine {
        err,
        text: text.to_owned(),
    }
}

/// The three shapes the corpus carries: a run that has said nothing, one
/// mid-flow, and one that settled badly with a command to run by hand.
#[test]
fn a_run_reads_its_lines_its_exit_and_its_fallback() {
    assert_eq!(
        body(&json!({ "kind": "login", "ok": true, "lines": [] })).unwrap(),
        LoginView::default()
    );
    let flowing = body(&json!({ "kind": "login", "ok": true,
        "lines": [{ "err": true, "text": "open https://provider.invalid/auth" },
                  { "err": false, "text": "{\"ready\":true}" }] }))
    .unwrap();
    assert_eq!(
        flowing.lines,
        [
            said(true, "open https://provider.invalid/auth"),
            said(false, "{\"ready\":true}")
        ]
    );
    assert!(!flowing.settled());
    let ended = body(&json!({ "kind": "login", "ok": true, "outcome": 78,
        "fallback": "yog exec --ws /ws bz --login --provider acme --browser",
        "lines": [{ "err": true, "text": "this provider has no device endpoint" }] }))
    .unwrap();
    assert_eq!(ended.outcome, Some(78));
    assert_eq!(
        ended.fallback.as_deref(),
        Some("yog exec --ws /ws bz --login --provider acme --browser")
    );
    assert!(ended.settled());
}

/// **Strict, like every other body here**: the array must be there, a line
/// must be an object, and both of its fields must be present and typed.
#[test]
fn a_malformed_line_refuses_naming_what_it_is() {
    assert_eq!(
        body(&json!({ "kind": "login", "ok": true })).unwrap_err(),
        "missing or non-array field \"lines\""
    );
    assert_eq!(
        body(&json!({ "kind": "login", "ok": true, "lines": ["said"] })).unwrap_err(),
        "login line: not a JSON object"
    );
    assert_eq!(
        body(&json!({ "kind": "login", "ok": true, "lines": [{ "text": "x" }] })).unwrap_err(),
        "missing or non-boolean field \"err\""
    );
    assert_eq!(
        body(&json!({ "kind": "login", "ok": true, "lines": [{ "err": false }] })).unwrap_err(),
        "missing or non-string field \"text\""
    );
    assert_eq!(
        body(&json!({ "kind": "login", "ok": true, "lines": [], "outcome": "0" })).unwrap_err(),
        "missing or non-integer field \"outcome\""
    );
}

/// **A frame is an append, and the settlement is the last word.** Lines
/// accrete in order; an exit that has arrived is never lost to a later frame
/// that states none, and one that has not is taken the moment it does.
#[test]
fn frames_fold_in_order_and_the_exit_is_kept() {
    let mut fold = LoginView {
        lines: vec![said(true, "one")],
        ..LoginView::default()
    };
    fold.absorb(LoginView {
        lines: vec![said(false, "two")],
        ..LoginView::default()
    });
    assert_eq!(fold.lines, [said(true, "one"), said(false, "two")]);
    assert!(!fold.settled());
    fold.absorb(LoginView {
        lines: Vec::new(),
        outcome: Some(0),
        fallback: None,
    });
    assert!(fold.settled());
    // A later frame that says nothing about the exit leaves the one that
    // arrived standing — the run settled, and nothing un-settles it.
    fold.absorb(LoginView::default());
    assert_eq!(fold.outcome, Some(0));
    // The fallback obeys the same rule from both directions: kept when the
    // later frame states none, replaced when it does.
    let mut held = LoginView {
        fallback: Some("by hand".to_owned()),
        ..LoginView::default()
    };
    held.absorb(LoginView::default());
    assert_eq!(held.fallback.as_deref(), Some("by hand"));
    held.absorb(LoginView {
        fallback: Some("this way instead".to_owned()),
        ..LoginView::default()
    });
    assert_eq!(held.fallback.as_deref(), Some("this way instead"));
}
