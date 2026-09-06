//! The held lane's own half: what a frame does to the rise rule, against a
//! real held mTLS server on loopback — the transport's own recipe, the same
//! one rung 1's tests dial.

use super::super::{Counts, Notice, read_seen, write_seen};
use super::{queued, wake};
use crate::codec::QueueRow;
use crate::test_support::{Turn, mint_ca, mint_leaf, scratch, serve_lanes};

/// A `wire/` under this app's files directory, minted and addressed the way
/// the platform hands one over.
fn provisioned(keys: &std::path::Path, address: &str) -> std::path::PathBuf {
    let files = scratch();
    let wire = files.join(super::super::WIRE);
    std::fs::create_dir_all(&wire).unwrap();
    for (from, to) in [
        ("ca.pem", "ca.pem"),
        ("client.pem", "client.pem"),
        ("client.key", "client.key"),
    ] {
        std::fs::copy(keys.join(from), wire.join(to)).unwrap();
    }
    std::fs::write(wire.join("address"), address).unwrap();
    files
}

/// **The lane is served ASIDE, and that is the harness's own rule** (DESIGN
/// §14.1): a connection whose request is `attention` is answered from the lane
/// script rather than positionally, because the seat's own attention lane
/// stands for its whole life and scripting it by position would move every
/// other index. This rung's read IS that op, so its turns go there.
fn lane(
    keys: &std::path::Path,
    turns: Vec<Turn>,
) -> (String, std::thread::JoinHandle<Vec<Vec<u8>>>) {
    serve_lanes(keys, "ca", "server", Vec::new(), turns)
}

fn pki() -> std::path::PathBuf {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "server", true);
    mint_leaf(&dir, "ca", "client", false);
    dir
}

/// One attention frame: one row per named conversation.
fn frame(rows: &[(&str, &str)]) -> Vec<u8> {
    let rows: Vec<serde_json::Value> = rows
        .iter()
        .map(|(workspace, agent)| {
            serde_json::json!({ "workspace": workspace, "agent": agent, "display": agent,
                "state": "quiescent", "uncertain": false, "signals": ["notify"], "says": "",
                "preview": "", "age_secs": 1, "pending": 0, "held": null, "failure": null,
                "flag": null })
        })
        .collect();
    serde_json::json!({ "ok": true, "kind": "attention", "rows": rows })
        .to_string()
        .into_bytes()
}

fn seen(pairs: &[(&str, usize)]) -> Counts {
    pairs
        .iter()
        .map(|(workspace, count)| ((*workspace).to_owned(), *count))
        .collect()
}

/// **The queue folds into the number rung 1 keeps.** One row per waiting
/// conversation, counted per workspace, and a workspace with no row absent
/// rather than zero.
#[test]
fn a_frame_counts_one_per_waiting_conversation() {
    let rows = vec![
        QueueRow {
            workspace: "main".to_owned(),
            ..blank()
        },
        QueueRow {
            workspace: "main".to_owned(),
            ..blank()
        },
        QueueRow {
            workspace: "side".to_owned(),
            ..blank()
        },
    ];
    assert_eq!(queued(&rows), seen(&[("main", 2), ("side", 1)]));
    assert_eq!(queued(&[]), Counts::new());
}

fn blank() -> QueueRow {
    QueueRow {
        workspace: String::new(),
        agent: "a".to_owned(),
        display: String::new(),
        state: crate::codec::AgentState::Quiescent,
        uncertain: false,
        signals: Vec::new(),
        says: String::new(),
        preview: String::new(),
        age_secs: 0,
        pending: 0,
        held: None,
        failure: None,
        flag: None,
    }
}

/// **A frame is the wake**, and what it announced is what it remembered — the
/// same rule and the same file rung 1 writes, so the two cannot double-wake.
#[test]
fn the_first_frame_that_rises_is_the_wake() {
    let keys = pki();
    let (address, _served) = lane(
        &keys,
        vec![Turn::Answer(vec![frame(&[
            ("main", "c-1"),
            ("main", "c-2"),
        ])])],
    );
    let files = provisioned(&keys, &address);
    assert_eq!(
        wake(&files),
        Some(Notice {
            title: "main wants attention".to_owned(),
            text: "main 2".to_owned(),
        })
    );
    assert_eq!(read_seen(&files), seen(&[("main", 2)]));
}

/// **A frame that says what was already announced is not a wake**, and the
/// lane goes on listening: the second frame is the rise.
#[test]
fn the_lane_keeps_reading_until_something_rises() {
    let keys = pki();
    let (address, _served) = lane(
        &keys,
        vec![Turn::Answer(vec![
            frame(&[("main", "c-1")]),
            frame(&[("main", "c-1"), ("main", "c-2")]),
        ])],
    );
    let files = provisioned(&keys, &address);
    write_seen(&files, &seen(&[("main", 1)]));
    assert_eq!(
        wake(&files),
        Some(Notice {
            title: "main wants attention".to_owned(),
            text: "main 2".to_owned(),
        })
    );
}

/// **A queue that emptied is a FALL, remembered and silent** — the operator
/// dealt with it, and the floor comes down so the next one speaks.
#[test]
fn an_emptied_queue_is_remembered_and_says_nothing() {
    let keys = pki();
    let (address, _served) = lane(&keys, vec![Turn::Answer(vec![frame(&[])])]);
    let files = provisioned(&keys, &address);
    write_seen(&files, &seen(&[("main", 3)]));
    assert_eq!(wake(&files), None);
    assert_eq!(read_seen(&files), Counts::new());
}

/// **Every failure is silence**, which is rung 1's rule at this rung: no
/// material, an engine that will not answer, and a frame of another kind each
/// end the lane with nothing said and nothing written.
#[test]
fn every_failure_is_silence() {
    // No material at all.
    let bare = scratch();
    assert_eq!(wake(&bare), None);
    // Material that names an address nothing answers on.
    let keys = pki();
    let files = provisioned(&keys, "127.0.0.1:9");
    assert_eq!(wake(&files), None);
    // An engine that answers another shape: the lane ends rather than
    // guessing at it, and nothing is remembered.
    let (address, _served) = lane(
        &keys,
        vec![Turn::Answer(vec![
            serde_json::json!({ "ok": true, "kind": "nudged" })
                .to_string()
                .into_bytes(),
        ])],
    );
    let wrong = provisioned(&keys, &address);
    assert_eq!(wake(&wrong), None);
    assert_eq!(read_seen(&wrong), Counts::new());
}
