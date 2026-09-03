//! The table end to end: every tool runs for real — a real `sh`, real files
//! in a scratch directory — because a tool that only a mock ever exercised is
//! a tool nothing verified.

use super::{BAD_INPUT, NO_SUCH_TOOL, advertisement, run_in};

/// The table's dispatch, with a scratch directory standing in for the app's
/// own storage — no test here reaches the arm that uses it.
fn run(tool: &str, input: &serde_json::Value) -> crate::codec::Capture {
    run_in(tool, input, "/nonexistent")
}
use crate::test_support::scratch;
use serde_json::json;
use std::time::Duration;

#[test]
fn the_advertisement_is_the_table_and_every_element_is_three_facts() {
    let set = advertisement();
    let names: Vec<&str> = set.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "shell",
            "read_file",
            "write_file",
            "list_dir",
            "ui_read",
            "ui_tap",
            "ui_type",
            "ui_key",
            "screenshot",
            "device",
            "clipboard_set",
            "notify",
            "open"
        ]
    );
    for tool in &set {
        assert!(
            !tool.description.is_empty(),
            "{} has no description",
            tool.name
        );
        assert_eq!(tool.input_schema["type"], "object");
        assert!(tool.input_schema["properties"].is_object());
        // A name is a single path component: it is the handle a load act
        // addresses, and a separator would make it address a filesystem.
        assert!(
            !tool.name.contains('/'),
            "{} is not one component",
            tool.name
        );
    }
}

/// **The invariant that makes the consent one fact rather than two**
/// (bl-0ac8). `crate::host` refuses every invocation carrying a `cwd`, and it
/// is allowed to do that unconditionally only because nothing in this table
/// can consent — `tool()` has no parameter for it. The day a dispatch here can
/// honour a directory, this is what fails, and it names the check to change.
#[test]
fn no_advertised_tool_consents_to_a_carried_working_directory() {
    for tool in advertisement() {
        assert!(
            !tool.subject_cwd,
            "{} consents to subject_cwd, so crate::host's blanket refusal is now wrong",
            tool.name
        );
    }
}

#[test]
fn an_unknown_tool_and_unreadable_arguments_are_two_sentences() {
    let missing = run("nope", &json!({}));
    assert_eq!(missing.exit_code, NO_SUCH_TOOL);
    assert_eq!(
        missing.stderr,
        "this machine carries no tool called \"nope\"\n"
    );
    let unreadable = run("shell", &json!("not an object"));
    assert_eq!(unreadable.exit_code, BAD_INPUT);
    assert_eq!(unreadable.stderr, "the arguments are not a JSON object\n");
    let no_field = run("shell", &json!({}));
    assert_eq!(no_field.exit_code, BAD_INPUT);
    assert_eq!(
        no_field.stderr,
        "missing or non-string argument \"command\"\n"
    );
}

#[test]
fn the_shell_tool_runs_a_command_and_carries_all_three_facts() {
    let ok = run("shell", &json!({ "command": "echo out; echo err >&2" }));
    assert_eq!(ok.stdout, "out\n");
    assert_eq!(ok.stderr, "err\n");
    assert_eq!(ok.exit_code, 0);
    let bad = run("shell", &json!({ "command": "exit 7" }));
    assert_eq!(bad.exit_code, 7);
}

#[test]
fn a_command_that_outruns_its_deadline_is_terminated_and_says_so() {
    let capture = super::shell::execute("sleep 30", Duration::from_millis(80));
    assert_eq!(capture.exit_code, super::shell::TIMED_OUT);
    assert!(
        capture.stderr.contains("still running after"),
        "stderr: {}",
        capture.stderr
    );
}

#[test]
fn a_signalled_command_reports_the_shells_own_convention() {
    let capture = super::shell::execute("kill -9 $$", Duration::from_secs(10));
    assert_eq!(capture.exit_code, 128 + 9);
}

#[test]
fn a_machine_with_no_interpreter_says_so_rather_than_hanging() {
    let capture = super::shell::execute_with(
        "definitely-not-an-interpreter",
        "echo hi",
        Duration::from_secs(10),
    );
    assert_eq!(capture.exit_code, 126);
    assert!(
        capture.stderr.starts_with("could not run a shell: "),
        "stderr: {}",
        capture.stderr
    );
}

#[test]
fn a_listing_whose_entry_raced_a_deletion_still_names_it() {
    // The race itself cannot be staged, so the shaping is asked directly:
    // a name the directory listed and the stat then found nothing for.
    assert_eq!(super::files::line("gone", None), "?             -  gone");
}

#[test]
fn a_read_answers_the_file_and_a_missing_one_refuses_naming_it() {
    let dir = scratch();
    let path = dir.join("note.txt");
    std::fs::write(&path, "hello\nthere").unwrap();
    let ok = run("read_file", &json!({ "path": path.to_str().unwrap() }));
    assert_eq!(ok.stdout, "hello\nthere");
    assert_eq!(ok.exit_code, 0);
    let missing = run(
        "read_file",
        &json!({ "path": dir.join("nope").to_str().unwrap() }),
    );
    assert_eq!(missing.exit_code, 1);
    assert!(missing.stderr.contains("nope"), "{}", missing.stderr);
    assert_eq!(run("read_file", &json!({})).exit_code, BAD_INPUT);
}

#[test]
fn a_truncated_read_says_so_and_still_succeeds() {
    let dir = scratch();
    let path = dir.join("long.txt");
    std::fs::write(&path, "abcdefghij").unwrap();
    let capture = run(
        "read_file",
        &json!({ "path": path.to_str().unwrap(), "limit": 4 }),
    );
    assert_eq!(capture.stdout, "abcd");
    assert_eq!(capture.stderr, "truncated to 4 characters\n");
    assert_eq!(capture.exit_code, 0);
    // A cap of zero is not a cap: the default stands rather than an empty read.
    let zero = run(
        "read_file",
        &json!({ "path": path.to_str().unwrap(), "limit": 0 }),
    );
    assert_eq!(zero.stdout, "abcdefghij");
}

#[test]
fn a_read_of_bytes_no_string_can_name_replaces_them() {
    let dir = scratch();
    let path = dir.join("bytes.bin");
    std::fs::write(&path, [0x68, 0xff, 0x69]).unwrap();
    let capture = run("read_file", &json!({ "path": path.to_str().unwrap() }));
    assert_eq!(capture.stdout, "h\u{fffd}i");
}

#[test]
fn a_write_creates_its_parents_and_reports_what_it_wrote() {
    let dir = scratch();
    let path = dir.join("deep").join("nested").join("f.txt");
    let capture = run(
        "write_file",
        &json!({ "path": path.to_str().unwrap(), "content": "body" }),
    );
    assert_eq!(capture.exit_code, 0);
    assert!(capture.stdout.starts_with("wrote 4 bytes to "));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "body");
    assert_eq!(
        run("write_file", &json!({ "path": "/x" })).exit_code,
        BAD_INPUT
    );
}

#[test]
fn a_write_that_the_filesystem_refuses_says_which_path() {
    let dir = scratch();
    let wall = dir.join("wall");
    std::fs::write(&wall, "not a directory").unwrap();
    // A parent that is a file, not a directory: the create refuses.
    let capture = run(
        "write_file",
        &json!({ "path": wall.join("under").to_str().unwrap(), "content": "x" }),
    );
    assert_eq!(capture.exit_code, 1);
    assert!(capture.stderr.contains("wall"), "{}", capture.stderr);
}

#[test]
fn a_write_to_an_unwritable_path_refuses() {
    // The root of a read-only filesystem view: no parent to create, and the
    // write itself is refused — the arm a bad parent does not reach.
    let capture = run(
        "write_file",
        &json!({ "path": "/proc/version", "content": "x" }),
    );
    assert_eq!(capture.exit_code, 1);
    assert!(
        capture.stderr.contains("/proc/version"),
        "{}",
        capture.stderr
    );
}

#[test]
fn a_listing_is_sorted_marked_and_sized() {
    let dir = scratch();
    std::fs::write(dir.join("b.txt"), "12345").unwrap();
    std::fs::create_dir(dir.join("a-dir")).unwrap();
    let capture = run("list_dir", &json!({ "path": dir.to_str().unwrap() }));
    let lines: Vec<&str> = capture.stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("dir "), "{}", lines[0]);
    assert!(lines[0].ends_with("a-dir"), "{}", lines[0]);
    assert!(lines[1].starts_with("file "), "{}", lines[1]);
    assert!(lines[1].ends_with("b.txt"), "{}", lines[1]);
    assert!(lines[1].contains('5'), "{}", lines[1]);
    assert_eq!(capture.exit_code, 0);
}

#[test]
fn a_listing_past_its_cap_says_how_many_there_were() {
    let dir = scratch();
    for n in 0..4 {
        std::fs::write(dir.join(format!("f{n}")), "").unwrap();
    }
    let capture = run(
        "list_dir",
        &json!({ "path": dir.to_str().unwrap(), "limit": 2 }),
    );
    assert_eq!(capture.stdout.lines().count(), 2);
    assert_eq!(capture.stderr, "4 entries, showing 2\n");
}

#[test]
fn a_listing_of_what_is_not_a_directory_refuses() {
    let dir = scratch();
    let capture = run(
        "list_dir",
        &json!({ "path": dir.join("absent").to_str().unwrap() }),
    );
    assert_eq!(capture.exit_code, 1);
    assert_eq!(run("list_dir", &json!({})).exit_code, BAD_INPUT);
}
