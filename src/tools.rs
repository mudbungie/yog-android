//! **What this machine can do**, and the dispatch that does it (REMOTE §5).
//!
//! **The one lawful deviation from §5.2, stated where it is taken.** Upstream
//! derives a tool host's advertisement from an operator-authored `tools.json`
//! naming an argv per tool, and spawns that argv. A phone has no
//! operator-authored argv and nowhere to install one: the deliverable is an
//! APK, and a config file naming executables that do not exist on the device
//! would be a fiction the advertisement then published. So the table is
//! **built into the app** — the same three advertised facts per tool
//! (REMOTE §5.1), projected from here, and dispatch to a function rather than
//! a spawn. Everything the wire sees is unchanged, which is the test of
//! whether a deviation is lawful; DESIGN §6 records it.
//!
//! **Every tool answers in the capture's three facts** — stdout, stderr, exit
//! code (REMOTE §5.3) — because a capture becomes a model's tool result and
//! nothing downstream carries a second shape. A tool with nothing to say on
//! stderr says nothing there rather than inventing a shape, and the exit code
//! is the verdict: zero is the answer, non-zero is the refusal.
//!
//! **The whole table is host-testable and is tested**, because it is ordinary
//! Rust over the standard library — the same code runs under the suite and on
//! the device, and a tool whose behaviour only the phone could witness would
//! be a tool nothing verifies.

use serde_json::{Map, Value, json};

use crate::codec::{Capture, Tool};

mod files;
mod shell;
mod ui;

#[cfg(test)]
mod tests;

/// The verdict a name this machine does not carry earns — the shell's own
/// convention for "command not found", and REMOTE §5's *"a client refuses a
/// tool it no longer carries"* answered at the end that actually knows.
pub const NO_SUCH_TOOL: i32 = 127;

/// The verdict a call this machine could not read earns. A schema is a
/// statement to a model and a model that mis-called one gets told which field,
/// in band, rather than a silent empty answer.
pub const BAD_INPUT: i32 = 2;

/// What this machine presents on connect. Order is the table's, which is the
/// order an operator reads them in; nothing downstream depends on it.
///
/// The interface tools are here whether or not their platform service is
/// enabled: an advertisement is a fact about what this machine offers, and
/// whether it can act right now is a refusal in band (REMOTE §5's own
/// staleness correction). Two tables would put a right-now fact into a
/// durable document.
pub fn advertisement() -> Vec<Tool> {
    let mut set = vec![
        shell::tool(),
        files::read_tool(),
        files::write_tool(),
        files::list_tool(),
    ];
    set.extend(ui::tools());
    set
}

/// Run one invocation locally. **Total**: every outcome is a capture, because
/// an invocation that earned no answer would be the hang the whole routing leg
/// exists to exclude. A name this machine does not carry, and a call whose
/// arguments it cannot read, are two exit codes and two sentences — never two
/// kinds of failure a caller must tell apart.
///
/// `data_dir` is this app's own storage — the one directory this uid can
/// always write, and where a screenshot goes when the caller names no path.
pub fn run_in(tool: &str, input: &Value, data_dir: &str) -> Capture {
    let Some(o) = input.as_object() else {
        return refused(BAD_INPUT, "the arguments are not a JSON object");
    };
    match tool {
        shell::NAME => shell::run(o),
        files::READ => files::read(o),
        files::WRITE => files::write(o),
        files::LIST => files::list(o),
        ui::READ | ui::TAP | ui::TYPE | ui::KEY | ui::SHOT => ui::run(tool, o, data_dir),
        other => refused(
            NO_SUCH_TOOL,
            &format!("this machine carries no tool called {other:?}"),
        ),
    }
}

/// A refusal as a capture: the sentence on stderr, where a tool's own
/// diagnostics go, and the verdict in the code.
pub(crate) fn refused(exit_code: i32, why: &str) -> Capture {
    Capture {
        stdout: String::new(),
        stderr: format!("{why}\n"),
        exit_code,
    }
}

/// An answer as a capture: what the tool produced, and the zero that says it
/// worked.
pub(crate) fn answered(stdout: String) -> Capture {
    Capture {
        stdout,
        stderr: String::new(),
        exit_code: 0,
    }
}

/// A required string argument, or the sentence naming it. Arguments are read
/// strictly for the schema's reason: the model was told what this takes.
pub(crate) fn arg(o: &Map<String, Value>, key: &str) -> Result<String, String> {
    o.get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("missing or non-string argument {key:?}"))
}

/// An optional positive-integer argument, defaulted — a cap a caller may
/// state and usually does not.
pub(crate) fn cap(o: &Map<String, Value>, key: &str, fallback: usize) -> usize {
    o.get(key)
        .and_then(Value::as_u64)
        .and_then(|n| usize::try_from(n).ok())
        .filter(|n| *n > 0)
        .unwrap_or(fallback)
}

/// One tool's advertised element. The schema is written here as the JSON it
/// is: it is this machine's statement to a model, and a builder that composed
/// it from parts would be a second grammar to keep in step with the first.
pub(crate) fn tool(name: &str, description: &str, schema: Value) -> Tool {
    Tool {
        name: name.to_owned(),
        description: description.to_owned(),
        input_schema: schema,
        // PROTOCOL 2's optional fourth fact, and this table states it nowhere:
        // `false` rides as an absent key (`codec::tools::one`), so the bytes
        // this machine advertises are the three facts it advertised before the
        // version moved. What consent would mean on a phone is a decision, and
        // it is not this constructor's.
        subject_cwd: false,
    }
}

/// The `{"type":"object", ...}` envelope every schema in this table shares.
pub(crate) fn object_schema(properties: Value, required: &[&str]) -> Value {
    json!({ "type": "object", "properties": properties, "required": required })
}
