//! **The file tools**: read one, write one, list a directory.
//!
//! They are the shell tool's siblings rather than its subset, and the reason
//! is the capture: a `cat` through a shell answers with the file's bytes
//! *interleaved with whatever the shell said*, and a caller that must parse a
//! prompt out of an answer is a caller that will get it wrong. A read that
//! names its own failure — no such file, not a file, not text — is worth the
//! three tools it costs.
//!
//! **A read is bounded and says when it truncated.** A model's context is
//! finite and a file is not; a tool that quietly returned the first N bytes
//! would be lying by omission, so the cap is stated in the schema, the caller
//! may raise it, and a truncated answer says so on stderr while still exiting
//! zero — the read worked, it was just not the whole file.

use std::path::Path;

use serde_json::{Map, Value, json};

use super::{BAD_INPUT, answered, arg, cap, object_schema, refused};
use crate::codec::{Capture, Tool};

pub(crate) const READ: &str = "read_file";
pub(crate) const WRITE: &str = "write_file";
pub(crate) const LIST: &str = "list_dir";

/// The verdict a path this machine could not use earns. One code for every
/// filesystem refusal: which one it was is the sentence's job, and a caller
/// branching on the difference between "absent" and "unreadable" would be
/// making a decision the platform already made.
const NO_PATH: i32 = 1;

/// How much of a file a read hands back when the caller states no cap. Large
/// enough for an ordinary source file, small enough that an accidental read of
/// something enormous does not fill a context window.
const READ_CAP: usize = 64 * 1024;

/// How many entries a listing hands back when the caller states no cap.
const LIST_CAP: usize = 500;

pub(crate) fn read_tool() -> Tool {
    super::tool(
        READ,
        "Read a text file from this Android device's filesystem, as the app's own user. \
         Returns at most 65536 characters unless a larger `limit` is given; a truncated \
         read says so on stderr and still succeeds. A file that is not valid UTF-8 comes \
         back with the unreadable bytes replaced.",
        object_schema(
            json!({ "path": { "type": "string", "description": "absolute path to the file" },
                    "limit": { "type": "integer",
                               "description": "maximum characters to return" } }),
            &["path"],
        ),
    )
}

pub(crate) fn write_tool() -> Tool {
    super::tool(
        WRITE,
        "Write a text file on this Android device, as the app's own user, creating it if \
         needed and replacing whatever was there. Parent directories are created. Most of \
         the filesystem is not writable by an app; the app's own storage is.",
        object_schema(
            json!({ "path": { "type": "string", "description": "absolute path to write" },
                    "content": { "type": "string", "description": "the file's new content" } }),
            &["path", "content"],
        ),
    )
}

pub(crate) fn list_tool() -> Tool {
    super::tool(
        LIST,
        "List a directory on this Android device, as the app's own user. One entry per \
         line, each marked `dir` or `file` with its size in bytes. Returns at most 500 \
         entries unless a larger `limit` is given.",
        object_schema(
            json!({ "path": { "type": "string", "description": "absolute path to the directory" },
                    "limit": { "type": "integer",
                               "description": "maximum entries to return" } }),
            &["path"],
        ),
    )
}

pub(crate) fn read(o: &Map<String, Value>) -> Capture {
    let path = match arg(o, "path") {
        Ok(path) => path,
        Err(why) => return refused(BAD_INPUT, &why),
    };
    let limit = cap(o, "limit", READ_CAP);
    match std::fs::read(&path) {
        Err(e) => refused(NO_PATH, &format!("{path}: {e}")),
        Ok(bytes) => clipped(&String::from_utf8_lossy(&bytes), limit),
    }
}

/// The text, cut to `limit` characters — and told about when it was cut.
/// Characters rather than bytes because the cap is a statement about how much
/// a reader is being handed, and a cut mid-character would hand back bytes no
/// string can name.
fn clipped(text: &str, limit: usize) -> Capture {
    if text.chars().count() <= limit {
        return answered(text.to_owned());
    }
    Capture {
        stdout: text.chars().take(limit).collect(),
        stderr: format!("truncated to {limit} characters\n"),
        exit_code: 0,
    }
}

pub(crate) fn write(o: &Map<String, Value>) -> Capture {
    let (path, content) = match (arg(o, "path"), arg(o, "content")) {
        (Ok(path), Ok(content)) => (path, content),
        (Err(why), _) | (_, Err(why)) => return refused(BAD_INPUT, &why),
    };
    if let Some(parent) = Path::new(&path).parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        return refused(NO_PATH, &format!("{}: {e}", parent.display()));
    }
    match std::fs::write(&path, &content) {
        Ok(()) => answered(format!("wrote {} bytes to {path}\n", content.len())),
        Err(e) => refused(NO_PATH, &format!("{path}: {e}")),
    }
}

pub(crate) fn list(o: &Map<String, Value>) -> Capture {
    let path = match arg(o, "path") {
        Ok(path) => path,
        Err(why) => return refused(BAD_INPUT, &why),
    };
    let limit = cap(o, "limit", LIST_CAP);
    let reader = match std::fs::read_dir(&path) {
        Ok(reader) => reader,
        Err(e) => return refused(NO_PATH, &format!("{path}: {e}")),
    };
    // Sorted, because a directory's order is the filesystem's and a listing
    // that reshuffled between two reads would look like a change.
    let mut lines: Vec<String> = reader
        .flatten()
        .map(|entry| {
            line(
                &entry.file_name().to_string_lossy(),
                entry.metadata().ok().as_ref(),
            )
        })
        .collect();
    lines.sort();
    let total = lines.len();
    lines.truncate(limit);
    let stdout = lines.join("\n");
    if total > limit {
        return Capture {
            stdout,
            stderr: format!("{total} entries, showing {limit}\n"),
            exit_code: 0,
        };
    }
    answered(stdout)
}

/// One entry as its line: what it is, how big, and its name.
///
/// The metadata is a parameter rather than a lookup because `None` — a name
/// the directory listed whose stat then found nothing, which is a race with a
/// deletion — is otherwise a branch no test can reach. Saying so beats
/// dropping the row or refusing the whole listing over one vanished name.
pub(super) fn line(name: &str, meta: Option<&std::fs::Metadata>) -> String {
    let (mark, size) = match meta {
        Some(meta) if meta.is_dir() => ("dir ", "-".to_owned()),
        Some(meta) => ("file", meta.len().to_string()),
        None => ("?   ", "-".to_owned()),
    };
    format!("{mark} {size:>10}  {name}")
}
