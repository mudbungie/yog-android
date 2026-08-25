//! The tool vocabulary's pins: the exact bytes the server writes and reads,
//! because a drifted spelling is a refused gesture rather than a red test
//! anywhere else.

use super::{Capture, Invocation, Tool, capture_of, capture_value, encode_tools, invocation_of};
use serde_json::json;

fn tool() -> Tool {
    Tool {
        name: "shell".into(),
        description: "run a command".into(),
        input_schema: json!({ "type": "object" }),
    }
}

#[test]
fn an_advertised_element_is_three_facts_and_no_more() {
    assert_eq!(
        encode_tools(&[tool()]),
        json!([{ "name": "shell", "description": "run a command",
                 "input_schema": { "type": "object" } }])
    );
    assert_eq!(encode_tools(&[]), json!([]));
}

#[test]
fn a_capture_is_the_three_facts_both_ways() {
    let capture = Capture {
        stdout: "out".into(),
        stderr: "err".into(),
        exit_code: 3,
    };
    let v = capture_value(&capture);
    assert_eq!(
        v,
        json!({ "stdout": "out", "stderr": "err", "exit_code": 3 })
    );
    assert_eq!(capture_of(&v).unwrap(), capture);
}

#[test]
fn a_malformed_capture_refuses_by_name() {
    assert_eq!(
        capture_of(&json!("x")).unwrap_err(),
        "capture: not a JSON object"
    );
    assert_eq!(
        capture_of(&json!({ "stderr": "", "exit_code": 0 })).unwrap_err(),
        "missing or non-string field \"stdout\""
    );
    assert_eq!(
        capture_of(&json!({ "stdout": "", "stderr": "", "exit_code": 5_000_000_000_i64 }))
            .unwrap_err(),
        "capture: exit_code 5000000000 out of range"
    );
}

#[test]
fn an_invocation_reads_back_with_its_arguments_verbatim() {
    let v = json!({ "invocation": "i1", "tool": "shell",
                    "input": { "command": "id" } });
    assert_eq!(
        invocation_of(&v).unwrap(),
        Invocation {
            id: "i1".into(),
            tool: "shell".into(),
            input: json!({ "command": "id" }),
        }
    );
}

#[test]
fn a_malformed_invocation_refuses_by_name() {
    assert_eq!(
        invocation_of(&json!([])).unwrap_err(),
        "invocation: not an object"
    );
    assert_eq!(
        invocation_of(&json!({ "tool": "shell", "input": {} })).unwrap_err(),
        "missing or non-string field \"invocation\""
    );
    // A call with no input is not a call with `{}`.
    assert_eq!(
        invocation_of(&json!({ "invocation": "i1", "tool": "shell" })).unwrap_err(),
        "invocation: missing field \"input\""
    );
}

#[test]
fn equality_is_reflexive_over_a_schema_and_an_argument_object() {
    assert_eq!(tool(), tool());
    let one = Invocation {
        id: "i".into(),
        tool: "t".into(),
        input: json!({ "n": 1.5 }),
    };
    assert_eq!(one.clone(), one);
}
