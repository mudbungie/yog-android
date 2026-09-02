//! The conversation listing row — the mirror of the server's `conv_row`
//! spelling (`boundary/reply/rows.rs`). Two recorded narrowings, both named
//! in the crate-root codec doc: `alignment` rides through as raw JSON and a
//! ball chip's `state` as its bare token, each typed the day a surface here
//! paints it.

use serde_json::Value;

use super::fields::{bool_of, i64_of, opt, pick, str_of, usize_of};

/// One conversation-list row: a root at depth 0, a subtree member below.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvRow {
    pub root_id: String,
    /// The display ladder's answer — always paintable.
    pub display: String,
    /// The **addressable** name: a valid `message` target. Withheld by the
    /// server when only a display-only legacy rung exists.
    pub name: Option<String>,
    pub display_only: bool,
    pub state: AgentState,
    pub uncertain: bool,
    pub preview: String,
    pub age_secs: i64,
    /// **When the subtree last acted**, as a unix epoch second (REMOTE §9.9).
    /// Carried rather than derived: a seat holding `age_secs` alone could
    /// subtract it from its own clock, and then every client says a different
    /// time for one instant. It rides BESIDE the age and is not a second copy
    /// of it — the age is the distance from the ENGINE's clock at answer time
    /// and is that clock's only carrier, the stamp is absolute, and the pair
    /// says strictly more than either half.
    pub last_active_unix: i64,
    pub flight: Option<Flight>,
    pub attention: usize,
    pub members: usize,
    pub direct: usize,
    pub stoppable: bool,
    pub stop_children: bool,
    pub depth: usize,
    pub tone: Tone,
    /// **Why the latest model call failed** (REMOTE §9.10), the provider's
    /// own first clause. **Absent, not null**, and that is load-bearing: a
    /// reader must never have to tell *no failure* from *a failure with
    /// nothing to say*, so a `Bad` tone with no clause beside it reads as
    /// exactly the third thing it is — a call that failed and left no words.
    pub failure: Option<String>,
    /// The standing alignment verdict, untyped until painted here.
    pub alignment: Option<Value>,
    pub ball: Option<ConvBall>,
}

/// The row's ball chip. `state` is the §3.5 join token, untyped here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvBall {
    pub id: String,
    pub state: Option<String>,
    pub title: Option<String>,
    pub badge: Option<String>,
}

/// The §5.1 agent-state tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentState {
    Live,
    InFlight,
    Quiescent,
    Stopped,
}

/// What kind of work is in flight; `None` is a conversation at rest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flight {
    Inference,
    Tools,
    Subagents,
}

/// The §11 row tone, in the words the seats share.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Plain,
    Weak,
    Good,
    Bad,
    Live,
    InFlight,
}

const STATES: [(&str, AgentState); 4] = [
    ("live", AgentState::Live),
    ("in-flight", AgentState::InFlight),
    ("quiescent", AgentState::Quiescent),
    ("stopped", AgentState::Stopped),
];

const FLIGHTS: [(&str, Flight); 3] = [
    ("inference", Flight::Inference),
    ("tools", Flight::Tools),
    ("subagents", Flight::Subagents),
];

const TONES: [(&str, Tone); 6] = [
    ("plain", Tone::Plain),
    ("weak", Tone::Weak),
    ("good", Tone::Good),
    ("bad", Tone::Bad),
    ("live", Tone::Live),
    ("in-flight", Tone::InFlight),
];

/// Read one conversation row, strictly.
pub(crate) fn row(v: &Value) -> Result<ConvRow, String> {
    let o = v.as_object().ok_or("conversation row: not an object")?;
    let flight = match o.get("flight") {
        None | Some(Value::Null) => None,
        Some(_) => Some(pick(o, "flight", &FLIGHTS)?),
    };
    Ok(ConvRow {
        root_id: str_of(o, "root_id")?,
        display: str_of(o, "display")?,
        name: opt(o, "name", str_of)?,
        display_only: bool_of(o, "display_only")?,
        state: pick(o, "state", &STATES)?,
        uncertain: bool_of(o, "uncertain")?,
        preview: str_of(o, "preview")?,
        age_secs: i64_of(o, "age_secs")?,
        last_active_unix: i64_of(o, "last_active_unix")?,
        flight,
        attention: usize_of(o, "attention")?,
        members: usize_of(o, "members")?,
        direct: usize_of(o, "direct")?,
        stoppable: bool_of(o, "stoppable")?,
        stop_children: bool_of(o, "stop_children")?,
        depth: usize_of(o, "depth")?,
        tone: pick(o, "tone", &TONES)?,
        failure: opt(o, "failure", str_of)?,
        alignment: o.get("alignment").cloned(),
        ball: match o.get("ball") {
            None | Some(Value::Null) => None,
            Some(b) => Some(ball(b)?),
        },
    })
}

fn ball(v: &Value) -> Result<ConvBall, String> {
    let o = v.as_object().ok_or("ball chip: not an object")?;
    Ok(ConvBall {
        id: str_of(o, "id")?,
        state: opt(o, "state", str_of)?,
        title: opt(o, "title", str_of)?,
        badge: opt(o, "badge", str_of)?,
    })
}

#[cfg(test)]
mod tests;
