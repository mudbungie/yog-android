//! The tool window: what the wire spells, what the fold makes of it, and the
//! refusals a frame from before the window earns.

use super::{Call, calls_of, window};
use serde_json::{Value, json};

fn read(v: &Value) -> Result<Vec<Call>, String> {
    calls_of(v.as_object().unwrap())
}

/// The corpus's own pair: an opening transition carrying the name and the
/// input, and a closing one that restates neither.
#[test]
fn both_transitions_of_a_call_read_back_as_the_wire_wrote_them() {
    let events = read(&json!({ "tools": [
        { "tool_use": "toolu_01", "tool": "box2_Bash",
          "input": "{\"command\":\"hostname && uptime\"}" },
        { "tool_use": "toolu_01", "exit_code": 0 }
    ] }))
    .unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].tool.as_deref(), Some("box2_Bash"));
    assert!(events[0].exit_code.is_none(), "in flight until the capture");
    assert_eq!(events[1].exit_code, Some(0));
    assert!(events[1].tool.is_none(), "the closing one restates nothing");
}

/// **Required, empty included** (REMOTE §5.5): absent would make "this build
/// has no tool window" and "nothing ran since the last frame" one shape.
#[test]
fn an_empty_window_reads_and_an_absent_one_refuses_by_name() {
    assert!(read(&json!({ "tools": [] })).unwrap().is_empty());
    let missing = read(&json!({ "kind": "follow" })).unwrap_err();
    assert!(missing.contains("follow"), "{missing}");
    assert!(missing.contains("\"tools\""), "{missing}");
}

/// A transition of the wrong shape, or one with no identity, refuses naming
/// the shape — `tool_use` is the key the whole fold turns on.
#[test]
fn a_malformed_transition_refuses_naming_the_shape() {
    for bad in [
        json!({ "tools": [7] }),
        json!({ "tools": [{ "tool": "Bash" }] }),
        json!({ "tools": [{ "tool_use": "toolu_01", "exit_code": "nought" }] }),
    ] {
        let refusal = read(&bad).unwrap_err();
        assert!(refusal.starts_with("follow: "), "{refusal}");
    }
}

/// **The fold is a merge by id**, in the order the calls opened: the closing
/// transition lands on the call the opening one made, and a second call keeps
/// its own place.
#[test]
fn the_fold_merges_transitions_by_id_in_opening_order() {
    let events = read(&json!({ "tools": [
        { "tool_use": "b", "tool": "Read", "input": "{}" },
        { "tool_use": "a", "tool": "box2_Bash", "input": "{\"command\":\"uptime\"}" },
        { "tool_use": "a", "exit_code": 3 }
    ] }))
    .unwrap();
    let calls = window(&events);
    assert_eq!(calls.len(), 2, "two calls, three transitions");
    assert_eq!(
        calls[0].tool_use, "b",
        "the order is the window's, not sorted"
    );
    assert_eq!(
        calls[1].tool.as_deref(),
        Some("box2_Bash"),
        "held from the opening"
    );
    assert_eq!(calls[1].input.as_deref(), Some("{\"command\":\"uptime\"}"));
    assert_eq!(calls[1].exit_code, Some(3), "and the capture's own number");
}

/// A closing transition whose opening this fold never saw opens a call
/// carrying what the wire said and no more — the general path with the fields
/// absent, rather than an error arm that cannot be reached on a whole read.
#[test]
fn a_close_with_no_open_is_a_call_with_nothing_but_its_status() {
    let calls = window(&read(&json!({ "tools": [{ "tool_use": "z", "exit_code": 0 }] })).unwrap());
    assert_eq!(calls.len(), 1);
    assert!(calls[0].tool.is_none() && calls[0].input.is_none());
    assert_eq!(calls[0].exit_code, Some(0));
}

/// Nothing to fold folds to nothing.
#[test]
fn an_empty_window_folds_to_no_calls() {
    assert!(window(&[]).is_empty());
}
