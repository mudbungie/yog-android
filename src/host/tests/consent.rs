//! **REMOTE §5.4's worktree lane, arriving at a machine that never consented**
//! (bl-0ac8): an invocation carrying a `cwd`, and what this device does with
//! it. Its own file rather than another block in the parent, because the
//! parent is the loop's own story — advertise, wait, run, complete, and every
//! way a channel stops it — and this is the one thing the loop refuses.

use super::{advertised, host_against, routed, settle, work};
use serde_json::{Value, json};

/// The refusal, end to end. Two properties, and the second is the one worth
/// the test: the tool **did not run** — the dispatch echoes its arguments, so
/// a capture that carried `echo:` would be it having run — and the loop went
/// on, completing the invocation like any other. A carried cwd is a refusal,
/// never a stop: the far end asked a question this machine has an answer to.
#[test]
fn an_invocation_carrying_a_working_directory_is_refused_naming_the_key() {
    let (mut host, served) = host_against(vec![
        vec![advertised()],
        vec![work(json!([{ "invocation": "i1", "tool": "echo",
                           "cwd": "/w/home/agents/c-1",
                           "input": { "say": "hi" } }]))],
        vec![routed("i1")],
        vec![],
    ]);
    let standing = settle(&mut host, &|s| s.served == 1);
    assert_eq!(standing.last.as_deref(), Some("echo → 3"));
    let requests = served.join().unwrap();
    let completion: Value = serde_json::from_slice(&requests[2]).unwrap();
    let capture = &completion["capture"];
    assert_eq!(capture["exit_code"], json!(crate::tools::UNCONSENTED));
    assert_eq!(capture["stdout"], json!(""));
    let said = capture["stderr"].as_str().unwrap();
    // The key, the tool, the directory that was not entered, and the fact
    // nothing ran — a model reads this and a routing decision changes.
    assert!(said.contains("\"subject_cwd\""), "{said}");
    assert!(said.contains("echo"), "{said}");
    assert!(
        said.contains("/w/home/agents/c-1 was not entered"),
        "{said}"
    );
    assert!(said.contains("nothing ran"), "{said}");
}

/// A `cwd` stated as null is no cwd at all (the codec's own reading), so the
/// tool runs — the boundary between the two arms is the codec's `Option`, not
/// a second rule here.
#[test]
fn a_null_working_directory_is_no_working_directory_and_the_tool_runs() {
    let (mut host, served) = host_against(vec![
        vec![advertised()],
        vec![work(
            json!([{ "invocation": "i1", "tool": "echo", "cwd": null,
                           "input": { "say": "hi" } }]),
        )],
        vec![routed("i1")],
        vec![],
    ]);
    let standing = settle(&mut host, &|s| s.served == 1);
    assert_eq!(standing.last.as_deref(), Some("echo → 0"));
    let requests = served.join().unwrap();
    let completion: Value = serde_json::from_slice(&requests[2]).unwrap();
    assert_eq!(completion["capture"]["exit_code"], json!(0));
    assert_eq!(
        completion["capture"]["stdout"],
        json!("echo:{\"say\":\"hi\"}")
    );
}
