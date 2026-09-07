//! **The tool window on the follow lane** (REMOTE §5.5, PROTOCOL 15): what is
//! running, where, and what it came back with — the half of the lane that is
//! not prose.
//!
//! The lane carried the model's words alone, which is defensible for a
//! conversation editing its own worktree — the work is on disk and a diff is a
//! read away — and wrong for one administering a MACHINE, because the box is
//! precisely the thing an operator cannot inspect afterwards. The transcript
//! recorded every capture the whole time; it was the live view that dropped
//! them.
//!
//! **Two transitions per call, and the closing one restates nothing.** REMOTE
//! §5.5: litany lands `input.json` immediately before it dispatches a call and
//! `output.json` when the capture returns, so the pair of file existences *is*
//! the window opening and closing —
//!
//! ```text
//! {"tool_use": "toolu_01", "tool": "box2_Bash", "input": "{\"command\":\"…\"}"}
//! {"tool_use": "toolu_01", "exit_code": 0}
//! ```
//!
//! — and a follower holds the opening entry the name and input rode on, keyed
//! by `tool_use`. So one type spells both: a transition and a call have the
//! same four fields, and the fold that turns a list of the first into a list of
//! the second is a **merge by id** ([`window`]) rather than a second vocabulary
//! with a conversion between them.
//!
//! **`exit_code`'s presence is the status, and its value is not a verdict.**
//! Absent is a call in flight, present is one whose capture landed; §5.5 states
//! no third arm, so the two readings cannot disagree. What the number *means*
//! is nowhere on this lane — the trail's own reading (`codec::trail`) is the
//! shape yog uses when it wants a seat to know that, and inventing one here
//! would be the disagreement that ruling exists to prevent.
//!
//! **The machine a routed call ran on is already in the name.** REMOTE §5.1
//! presents a loaded remote tool as `<client>_<tool>`, always, so `box2_Bash`
//! says where it went and this seat neither splits that composition apart nor
//! asks the registry — which answers where a call would route *now*, a
//! different question from where it ran.
//!
//! **`tool` and `input` are optional on the wire and required by nothing
//! here.** The engine reads them out of a record that can be caught mid-write,
//! and it states the same tolerance every other reader of those files has;
//! `tool_use` is the identity a follower keys on and is the one strict field.

use serde_json::{Map, Value};

use super::super::fields::{i64_of, opt, str_of};

/// One call of the tool window — a transition off the wire, or the merged
/// state of a call once [`window`] has folded its transitions together.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    /// The provider's `tool_use` id: the call's identity, and the key both
    /// transitions and the transcript's own committed block share.
    pub tool_use: String,
    /// What the tool is called, verbatim as the engine recorded it — carrying
    /// its client for a routed call (module doc). `None` on a closing
    /// transition, and on a record with no readable name.
    pub tool: Option<String>,
    /// The call's input, already bounded and one-lined upstream. This seat
    /// does not summarize a second time: two truncations of one string are two
    /// answers to one question.
    pub input: Option<String>,
    /// The captured exit code. Its **presence** is the whole of "this call
    /// closed"; its value is the engine's record and not a verdict.
    pub exit_code: Option<i64>,
}

impl Call {
    /// Fold a later transition of the same call onto this one. Every field is
    /// last-stated-wins over what stands, which is the same shape
    /// [`Stream::absorb`](super::Stream::absorb) has for the prose: a closing
    /// transition states only `exit_code`, so the name and input it does not
    /// restate are the ones already held.
    fn absorb(&mut self, later: &Self) {
        self.tool = later.tool.clone().or_else(|| self.tool.clone());
        self.input = later.input.clone().or_else(|| self.input.clone());
        self.exit_code = later.exit_code.or(self.exit_code);
    }
}

/// **The window's calls, in the order they opened** — the fold of a read's
/// transitions, merged by `tool_use`.
///
/// A transition naming a call this fold has not seen opens one; every later
/// transition of that id folds onto it. That is total by construction, so a
/// closing transition whose opening is not held is not an error arm here: it
/// opens a call whose name and input are simply absent, which is what the wire
/// said. It cannot arise on a whole read — a read starts holding nothing and
/// its first frame carries the window from zero — and a rule with no case for
/// it is one fewer thing to get wrong than a case that never runs.
pub(crate) fn window(events: &[Call]) -> Vec<Call> {
    let mut calls: Vec<Call> = Vec::new();
    for event in events {
        match calls
            .iter_mut()
            .find(|call| call.tool_use == event.tool_use)
        {
            Some(held) => held.absorb(event),
            None => calls.push(event.clone()),
        }
    }
    calls
}

/// The `tools` list of one follow frame. **Required, empty list included**
/// (REMOTE §5.5): absent would make *"this build has no tool window"* and
/// *"nothing ran since the last frame"* one shape, so a frame in the pre-15
/// spelling refuses by name rather than decoding into a comfortable empty
/// list.
pub(crate) fn calls_of(o: &Map<String, Value>) -> Result<Vec<Call>, String> {
    o.get("tools")
        .and_then(Value::as_array)
        .ok_or_else(|| named("missing or non-array field \"tools\""))?
        .iter()
        .map(call_of)
        .collect()
}

/// One transition, strictly on its identity and forgiving of the three fields
/// the engine itself states as optional.
fn call_of(v: &Value) -> Result<Call, String> {
    let o = v
        .as_object()
        .ok_or_else(|| named("tool event is not an object"))?;
    Ok(Call {
        tool_use: str_of(o, "tool_use").map_err(named)?,
        tool: opt(o, "tool", str_of).map_err(named)?,
        input: opt(o, "input", str_of).map_err(named)?,
        exit_code: opt(o, "exit_code", i64_of).map_err(named)?,
    })
}

/// Every refusal here names the shape it refused, which is what makes a miss
/// locatable in a corpus replay rather than a decoder that fell over somewhere.
fn named(sentence: impl std::fmt::Display) -> String {
    format!("follow: {sentence}")
}

#[cfg(test)]
mod tests;
