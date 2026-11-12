//! The door's answer read, and the material it becomes.

use super::{Claimed, file, read};
use crate::material::{ADDRESS, ANCHORS, CHAIN, KEY, read_dir};
use crate::test_support::scratch;
use serde_json::{Value, json};

fn answer() -> Value {
    json!({
        "workspace": "home",
        "name": "phone",
        "grade": "operator",
        "address": ["engine.example:7737", "192.0.2.7:7737"],
        "ca": "-----BEGIN CERTIFICATE-----\nnotreal\n-----END CERTIFICATE-----\n",
        "cert": "-----BEGIN CERTIFICATE-----\nnotreal\n-----END CERTIFICATE-----\n",
    })
}

fn claimed() -> Claimed {
    read(answer().to_string().as_bytes()).unwrap()
}

/// REMOTE §15.1's six fields, the address as the list it is.
#[test]
fn the_answer_reads_as_its_six_fields() {
    let held = claimed();
    assert_eq!(held.workspace, "home");
    assert_eq!(held.name, "phone");
    assert_eq!(held.grade, "operator");
    assert_eq!(held.addresses, ["engine.example:7737", "192.0.2.7:7737"]);
    assert!(held.ca.contains("BEGIN CERTIFICATE"));
    assert!(held.cert.contains("BEGIN CERTIFICATE"));
}

/// A field that is absent, empty or of the wrong type is named; a body that
/// is not JSON names the first one it would have carried.
#[test]
fn an_answer_missing_a_field_names_it() {
    for key in ["workspace", "name", "grade", "address", "ca", "cert"] {
        for wrong in [Value::Null, json!(""), json!(7), json!([])] {
            let mut answer = answer();
            answer[key] = wrong;
            let refusal = read(answer.to_string().as_bytes()).unwrap_err();
            assert!(refusal.contains(&format!("no `{key}`")), "{refusal}");
        }
    }
    let refusal = read(b"<html>").unwrap_err();
    assert!(refusal.contains("no `address`"), "{refusal}");
}

/// **The four files, under the reader's own names**, the address one endpoint
/// to the line — and what was there before is replaced, the retired pair
/// left where it lay.
#[test]
fn filing_writes_the_four_files_and_replaces_an_older_identity() {
    let dir = scratch().join("wire");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(KEY), "the old key\n").unwrap();
    std::fs::write(dir.join("pairing.salt"), "left alone\n").unwrap();
    file(&dir, &claimed(), "the new key").unwrap();
    let read = |leaf: &str| std::fs::read_to_string(dir.join(leaf)).unwrap();
    assert_eq!(read(ADDRESS), "engine.example:7737\n192.0.2.7:7737\n");
    assert_eq!(read(KEY), "the new key\n");
    assert!(read(ANCHORS).ends_with("-----END CERTIFICATE-----\n"));
    assert!(read(CHAIN).ends_with("-----END CERTIFICATE-----\n"));
    assert_eq!(read("pairing.salt"), "left alone\n");
    let held = read_dir(&dir).unwrap().unwrap();
    assert_eq!(held.addresses, ["engine.example:7737", "192.0.2.7:7737"]);
    let mode = std::os::unix::fs::PermissionsExt::mode(
        &std::fs::metadata(dir.join(KEY)).unwrap().permissions(),
    );
    assert_eq!(mode & 0o777, 0o600, "the key is this app's alone");
    let left: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
        .filter(|name| !crate::material::WANTED.contains(&name.as_str()))
        .collect();
    assert_eq!(left, ["pairing.salt"], "nothing staged is left behind");
}

/// **A filing that cannot finish leaves the old identity standing**, and each
/// way it can fail names the path that failed: a directory that cannot be
/// made, a file that cannot be written, and one that cannot be put in place.
#[test]
fn a_filing_that_fails_names_the_path_and_keeps_what_was_there() {
    let root = scratch();
    std::fs::write(root.join("file"), "x").unwrap();
    let under = root.join("file").join("wire");
    let refusal = file(&under, &claimed(), "key").unwrap_err();
    assert!(
        refusal.starts_with(&under.display().to_string()),
        "{refusal}"
    );

    let dir = root.join("wire");
    std::fs::create_dir_all(dir.join(format!("{CHAIN}.new"))).unwrap();
    std::fs::write(dir.join(KEY), "the old key\n").unwrap();
    let refusal = file(&dir, &claimed(), "key").unwrap_err();
    assert!(refusal.contains(&format!("{CHAIN}.new")), "{refusal}");
    assert_eq!(
        std::fs::read_to_string(dir.join(KEY)).unwrap(),
        "the old key\n"
    );

    let dir = root.join("other");
    std::fs::create_dir_all(dir.join(ANCHORS).join("occupied")).unwrap();
    let refusal = file(&dir, &claimed(), "key").unwrap_err();
    assert!(
        refusal.starts_with(&dir.join(ANCHORS).display().to_string()),
        "{refusal}"
    );
}
