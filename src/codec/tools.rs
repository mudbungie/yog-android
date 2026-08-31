//! REMOTE §5's tool-host vocabulary, mirrored: what this machine presents
//! (`advertise`), what it is handed (`invocations`), and what it answers with
//! (`complete`). The parent spellings are the server's
//! `registry/tools.rs`, `registry/mailbox.rs` and `boundary/codec/tools.rs` —
//! **where the two disagree, one of them is a bug**, and the tests pin the
//! exact bytes so a disagreement is a red test rather than a refused gesture.
//!
//! Three types and no more, because REMOTE §5.1 is explicit that an
//! advertised element is three facts: a name that is one path component, a
//! description in the host's own words, and the JSON Schema **verbatim** —
//! neither validated nor rewritten here, because it is this machine's
//! statement to a model and narrowing it would be inventing a contract this
//! client does not own.

use serde_json::{Map, Value, json};

use super::fields::{i64_of, str_of};

/// One tool this machine offers (REMOTE §5.1).
#[derive(Debug, Clone, PartialEq)]
pub struct Tool {
    /// The name, a single path component — the handle a load act addresses.
    pub name: String,
    /// What it does, in this machine's own words.
    pub description: String,
    /// Its JSON Schema, verbatim.
    pub input_schema: Value,
}

/// `Eq` is written rather than derived for [`Value`]'s reason: it holds `f64`,
/// whose `NaN` is the one value equality is not reflexive over. A schema built
/// by this crate or read by a JSON decoder cannot hold one — the grammar has
/// no `NaN` literal — so equality here is reflexive by construction.
impl Eq for Tool {}

/// What this machine is handed to run: the engine's handle and the two facts
/// the far end needs. It carries no client — the read is answered to one
/// identity, and a host being told its own name would be a fact it holds.
#[derive(Debug, Clone, PartialEq)]
pub struct Invocation {
    /// The engine's handle, minted at the post and quoted by the completion.
    pub id: String,
    pub tool: String,
    /// The model's own `tool_use.input`, verbatim.
    pub input: Value,
}

impl Eq for Invocation {}

/// What running one produced — lernie's own tool contract, one for one: bytes
/// on stdout, bytes on stderr, the exit code the verdict. Text rather than
/// bytes because a capture becomes a model's tool result and a model's message
/// is text; every tool in this crate transcodes at its own edge, so nothing
/// downstream carries an encoding case.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Capture {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// [`one`]'s inverse, strict — the decode side the conformance corpus needs
/// (REMOTE §3: a client that only sends requests still decodes the request
/// fixtures). The schema comes back **verbatim**, exactly as it went out:
/// narrowing it on the way in would be this crate inventing a contract it does
/// not own, the same reason [`one`] does not validate it on the way out.
pub(crate) fn tool_of(v: &Value) -> Result<Tool, String> {
    let o = v.as_object().ok_or("tool: not an object")?;
    Ok(Tool {
        name: str_of(o, "name")?,
        description: str_of(o, "description")?,
        input_schema: o
            .get("input_schema")
            .cloned()
            .ok_or("tool: missing field \"input_schema\"")?,
    })
}

/// The advertised set as JSON — the one spelling, spent by the gesture
/// encoder and by nothing else in this crate.
pub(crate) fn encode_tools(tools: &[Tool]) -> Value {
    Value::Array(tools.iter().map(one).collect())
}

fn one(t: &Tool) -> Value {
    json!({ "name": t.name, "description": t.description,
            "input_schema": t.input_schema })
}

/// A capture as JSON — the one spelling, spent by the completing act.
pub(crate) fn capture_value(capture: &Capture) -> Value {
    json!({ "stdout": capture.stdout, "stderr": capture.stderr,
            "exit_code": capture.exit_code })
}

/// [`capture_value`]'s inverse, strict: a capture read back is a peer's
/// statement, so a missing field refuses rather than defaults.
pub(crate) fn capture_of(v: &Value) -> Result<Capture, String> {
    let o = v.as_object().ok_or("capture: not a JSON object")?;
    Ok(Capture {
        stdout: str_of(o, "stdout")?,
        stderr: str_of(o, "stderr")?,
        exit_code: exit_of(i64_of(o, "exit_code")?)?,
    })
}

/// An exit code narrowed to what a process can actually have exited with.
fn exit_of(code: i64) -> Result<i32, String> {
    i32::try_from(code).map_err(|_| format!("capture: exit_code {code} out of range"))
}

/// One queued invocation, read back — the follow-class read's row, on the
/// same strict terms.
pub(crate) fn invocation_of(v: &Value) -> Result<Invocation, String> {
    let o = v.as_object().ok_or("invocation: not an object")?;
    Ok(Invocation {
        id: str_of(o, "invocation")?,
        tool: str_of(o, "tool")?,
        input: input_of(o)?,
    })
}

/// The arguments, verbatim and required: a call with no input is not a call
/// with `{}` — an envelope that failed to say something is not a gesture.
fn input_of(o: &Map<String, Value>) -> Result<Value, String> {
    o.get("input")
        .cloned()
        .ok_or_else(|| "invocation: missing field \"input\"".to_owned())
}

#[cfg(test)]
mod tests;
