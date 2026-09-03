//! The scheduled fetch's decision, end to end: a real mTLS ask against a
//! one-shot server on loopback (the transport's own recipe), then every way a
//! run answers silence, then the rise rule itself.

use super::{Counts, MEMORY, Notice, SEEN, WIRE, counts, read_seen, risen, sweep, write_seen};
use crate::codec::{WsKind, WsRow};
use crate::test_support::{mint_ca, mint_leaf, scratch, serve_once};

/// A files directory with this device's wire material inside it, addressed at
/// `address` — the shape the platform hands [`sweep`].
fn seated(address: &str) -> std::path::PathBuf {
    let files = scratch();
    let wire = files.join(WIRE);
    std::fs::create_dir_all(&wire).unwrap();
    std::fs::write(wire.join("address"), address).unwrap();
    files
}

/// The same, with a PKI minted into the wire directory under the names
/// `material::WANTED` requires.
fn provisioned(pki: &std::path::Path, address: &str) -> std::path::PathBuf {
    let files = seated(address);
    let wire = files.join(WIRE);
    for (from, to) in [
        ("ca.pem", "ca.pem"),
        ("client.pem", "client.pem"),
        ("client.key", "client.key"),
    ] {
        std::fs::copy(pki.join(from), wire.join(to)).unwrap();
    }
    files
}

fn pki() -> std::path::PathBuf {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "server", true);
    mint_leaf(&dir, "ca", "client", false);
    dir
}

fn roster(rows: &str) -> Vec<u8> {
    format!("{{ \"ok\": true, \"kind\": \"workspaces\", \"rows\": [{rows}] }}").into_bytes()
}

fn ws(workspace: &str, attention: usize) -> String {
    format!(
        "{{ \"workspace\": \"{workspace}\", \"kind\": \"named\", \"attention\": {attention}, \
         \"agents\": 1, \"running\": false }}"
    )
}

fn row(workspace: &str, attention: usize) -> WsRow {
    WsRow {
        workspace: workspace.to_owned(),
        kind: WsKind::Named,
        attention,
        agents: 1,
        running: false,
        pinned: None,
        config_tip: None,
    }
}

fn seen(pairs: &[(&str, usize)]) -> Counts {
    pairs
        .iter()
        .map(|(workspace, count)| ((*workspace).to_owned(), *count))
        .collect()
}

// --- the run, over a real wire -------------------------------------------

#[test]
fn a_first_rise_wakes_the_operator_and_is_remembered() {
    let keys = pki();
    let (address, _served) = serve_once(&keys, "ca", "server", vec![roster(&ws("main", 2))]);
    let files = provisioned(&keys, &address);
    assert_eq!(
        sweep(&files),
        Some(Notice {
            title: "main wants attention".to_owned(),
            text: "main 2".to_owned(),
        })
    );
    // What it announced is what it wrote, so the next run has a floor.
    assert_eq!(read_seen(&files), seen(&[("main", 2)]));
}

#[test]
fn the_same_count_a_second_time_stays_silent() {
    let keys = pki();
    let (address, _served) = serve_once(&keys, "ca", "server", vec![roster(&ws("main", 2))]);
    let files = provisioned(&keys, &address);
    write_seen(&files, &seen(&[("main", 2)]));
    assert_eq!(sweep(&files), None);
}

#[test]
fn a_count_that_fell_is_remembered_so_the_next_rise_speaks() {
    let keys = pki();
    let (address, _served) = serve_once(&keys, "ca", "server", vec![roster(&ws("main", 1))]);
    let files = provisioned(&keys, &address);
    write_seen(&files, &seen(&[("main", 3)]));
    // Falling is the operator having dealt with it: no wake...
    assert_eq!(sweep(&files), None);
    // ...and the floor came down with it, so 2 is a rise again.
    assert_eq!(read_seen(&files), seen(&[("main", 1)]));
}

#[test]
fn a_roster_with_nothing_wanting_writes_an_empty_memory() {
    let keys = pki();
    let (address, _served) = serve_once(&keys, "ca", "server", vec![roster(&ws("main", 0))]);
    let files = provisioned(&keys, &address);
    write_seen(&files, &seen(&[("main", 4)]));
    assert_eq!(sweep(&files), None);
    assert_eq!(read_seen(&files), Counts::new());
}

// --- every silence -------------------------------------------------------

#[test]
fn a_device_with_no_material_says_nothing_and_writes_nothing() {
    let files = scratch();
    assert_eq!(sweep(&files), None);
    assert!(!files.join(MEMORY).join(SEEN).exists());
}

#[test]
fn half_provisioned_is_silence_too() {
    // An address and nothing else: `material::read_dir` refuses, and a
    // scheduled fetch has nobody to hand a refusal to.
    let files = seated("127.0.0.1:1");
    assert_eq!(sweep(&files), None);
}

#[test]
fn material_that_will_not_build_a_seat_is_silence() {
    let keys = pki();
    let files = provisioned(&keys, "127.0.0.1:1");
    std::fs::write(files.join(WIRE).join("ca.pem"), "not a certificate").unwrap();
    assert_eq!(sweep(&files), None);
}

#[test]
fn an_engine_that_does_not_answer_is_silence() {
    let keys = pki();
    // Port 1 on loopback: nothing listens, so the dial fails at the socket.
    let files = provisioned(&keys, "127.0.0.1:1");
    assert_eq!(sweep(&files), None);
    assert!(!files.join(MEMORY).join(SEEN).exists());
}

#[test]
fn an_answer_to_another_question_is_silence() {
    let keys = pki();
    let reply = br#"{ "ok": true, "kind": "conversations", "rows": [] }"#.to_vec();
    let (address, _served) = serve_once(&keys, "ca", "server", vec![reply]);
    let files = provisioned(&keys, &address);
    assert_eq!(sweep(&files), None);
}

// --- the rise rule, and the memory ---------------------------------------

#[test]
fn many_workspaces_are_counted_not_listed_in_the_title() {
    let notice = risen(&seen(&[("main", 1), ("side", 2)]), &Counts::new()).unwrap();
    assert_eq!(notice.title, "2 workspaces want attention");
    assert_eq!(notice.text, "main 1, side 2");
}

#[test]
fn only_the_risen_workspace_is_named() {
    let notice = risen(&seen(&[("main", 1), ("side", 2)]), &seen(&[("side", 2)])).unwrap();
    assert_eq!(notice.title, "main wants attention");
    assert_eq!(notice.text, "main 1");
}

#[test]
fn a_zero_is_stored_as_absence() {
    assert_eq!(
        counts(&[row("main", 0), row("side", 3)]),
        seen(&[("side", 3)])
    );
}

#[test]
fn every_unreadable_memory_reads_as_nothing_announced() {
    let files = scratch();
    let at = files.join(MEMORY);
    std::fs::create_dir_all(&at).unwrap();
    // No file at all.
    assert_eq!(read_seen(&files), Counts::new());
    for body in [
        "not json",
        "[]",
        r#"{ "tag": "yog-attention", "version": 99, "seen": {} }"#,
        r#"{ "tag": "someone-else", "version": 1, "seen": {} }"#,
        r#"{ "tag": "yog-attention", "version": 1 }"#,
    ] {
        std::fs::write(at.join(SEEN), body).unwrap();
        assert_eq!(read_seen(&files), Counts::new(), "{body}");
    }
}

#[test]
fn a_count_that_is_not_a_count_drops_its_row() {
    let files = scratch();
    let at = files.join(MEMORY);
    std::fs::create_dir_all(&at).unwrap();
    std::fs::write(
        at.join(SEEN),
        r#"{ "tag": "yog-attention", "version": 1, "seen": { "main": "two", "side": 3 } }"#,
    )
    .unwrap();
    assert_eq!(read_seen(&files), seen(&[("side", 3)]));
}

#[test]
fn a_memory_that_cannot_be_written_costs_nothing_but_a_repeat() {
    // The memory directory's name taken by a file: `create_dir_all` fails,
    // the write fails, and the run still answers what it decided.
    let files = scratch();
    std::fs::write(files.join(MEMORY), "in the way").unwrap();
    write_seen(&files, &seen(&[("main", 1)]));
    assert_eq!(read_seen(&files), Counts::new());
}
