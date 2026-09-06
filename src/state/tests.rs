//! The process's one slot, over real hosts against real scripted servers.
//!
//! **One test, and that is deliberate.** The subject is a process-global, so a
//! second test function would be a second writer racing this one under
//! `cargo test`'s thread pool — the ordering IS the assertion here (nothing
//! held, then one, then a refusal, then a replacement), and a lock to
//! serialize two halves of one story would only re-create the ordering this
//! test already has.

use super::{hold, holding, standing};
use crate::codec::{Capture, Tool};
use crate::foot::Foot;
use crate::host::{Health, Host};
use crate::test_support::{material, mint_ca, mint_leaf, scratch, serve_many};
use serde_json::{Value, json};
use std::time::Duration;

/// One host over the real transport against a server reading `scripts`, with a
/// nap that does not sleep — the ladder is `host::tests`' subject, not this
/// file's.
fn host(scripts: Vec<Vec<Vec<u8>>>) -> Host {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "server", true);
    mint_leaf(&dir, "ca", "client", false);
    let (address, _served) = serve_many(&dir, "ca", "server", scripts);
    let foot = Foot::open(&material(&dir, "ca", "client", &address)).unwrap();
    let tools = vec![Tool {
        name: "echo".into(),
        description: "say it back".into(),
        input_schema: json!({ "type": "object" }),
        subject_cwd: false,
    }];
    Host::start(
        foot,
        tools,
        Box::new(|tool: &str, input: &Value| Capture {
            stdout: format!("{tool}:{input}"),
            stderr: String::new(),
            exit_code: 0,
        }),
        Box::new(|_| {}),
    )
}

/// A host the engine refuses outright: it stops for good, which is the state
/// that makes the slot free again.
fn refused() -> Host {
    host(vec![vec![
        json!({ "ok": false, "error": "not registered here" })
            .to_string()
            .into_bytes(),
    ]])
}

/// A host that presents and then parks on its `invocations` read — the
/// ordinary living foot, and the one a second `hold` must not displace. The
/// server script ends after the receipt, so the read simply never answers.
fn parked() -> Host {
    host(vec![vec![
        json!({ "ok": true, "kind": "advertised", "wrote": false })
            .to_string()
            .into_bytes(),
    ]])
}

/// **At most one LIVE host per process**, in the order the app meets it: a
/// cold process, the first host, a relaunch that must not build a second, and
/// a stopped lane that the operator's own remedy — opening the app — can start
/// again.
#[test]
fn the_process_holds_at_most_one_live_host() {
    // A process that has taken up no host has no standing to paint. This is
    // the state every launch begins in and the one `pocket::line` answers
    // `None` from.
    assert_eq!(standing(), None);
    // …and nothing to ask the same question of one step earlier: `holding` is
    // what a caller asks BEFORE it builds one (§18.8).
    assert!(!holding());

    // The first host is taken up.
    assert!(hold(refused()));
    assert!(standing().is_some());

    // It is refused by the engine and stops for good. Waited for rather than
    // assumed: the worker publishes on its own thread.
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let health = standing().map(|s| s.health);
        if matches!(health, Some(Health::Stopped(_))) {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the refused host never stopped; last: {health:?}"
        );
        std::thread::sleep(Duration::from_millis(2));
    }

    // **A stopped host does not own the slot forever.** Without this the app
    // could never start a foot again inside the process that stopped one, and
    // the operator's remedy would silently do nothing. `holding` reads the
    // same publication `hold` does, so a stopped host is not one being held
    // either — which is what lets a service's door build a replacement.
    assert!(!holding());
    assert!(hold(parked()));

    // **And a live one does own it.** This is the relaunch case: an activity
    // destroyed and created again would otherwise build a second host on this
    // device's certificate while the first is still parked on its read, which
    // REMOTE §5.1's one-reader guard refuses naming this very device.
    assert!(!hold(parked()));
    assert!(holding());
    let standing = standing().expect("the live host is still the process's");
    assert!(!matches!(standing.health, Health::Stopped(_)));
}
