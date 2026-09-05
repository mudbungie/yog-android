//! **The conversation's own row** (`agent`): what it is, what it is doing,
//! and what it has spent — the header every other half of the records screen
//! is about.
//!
//! It is the largest shape on the reply surface, and the reader takes what a
//! phone's width can hold. What it declines and why is the module doc next
//! door; nothing is decided twice here.
//!
//! **Three of these facts are the engine's own rendering and none is
//! recomputed** (lernie DESIGN §4.32): the flight strip's characteristics are
//! prose one derivation assembles with per-segment omission rules, the money
//! is the engine's spelling of the integer beside it, and the context percent
//! is its own rounding — deliberately unclamped, so a context that has
//! outgrown its window reads as `140%`. A seat that divided the two figures
//! itself would be re-taking a decision upstream took on purpose.

use serde_json::{Map, Value};

use super::super::fields::{bool_of, i64_of, opt, str_of};

/// The conversation, as the engine states it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agent {
    /// The display ladder's answer — what the transcript's bar already says.
    pub display: String,
    /// The root of the subtree this conversation belongs to.
    pub root: String,
    /// **The §5.1 state and the §11 flight class, as the engine's own
    /// words.** Nothing on this screen branches on either — it paints them —
    /// so they are carried whole exactly as a step's `framing`, its `wound`
    /// and a child card's `state` are. The conversation ROW picks the same two
    /// against tables (`codec::conv`) because its controls DO branch, which is
    /// the difference between the two readings rather than an inconsistency
    /// between them. Empty `flight` is a conversation at rest.
    pub state: String,
    pub flight: String,
    /// Whether the engine can see the conversation at all, and whether its
    /// latest model call was refused. Two independent readings, both the
    /// engine's, and `failure` is what the provider actually said — absent
    /// rather than empty, so *no failure* and *a failure with nothing to
    /// say* stay apart.
    pub present: bool,
    pub refused: bool,
    pub failure: Option<String>,
    /// **The §6 marks** in the engine's badge order. A fork point is one of
    /// the things these name (`parity.toml`'s `fork` line), which is why an
    /// unknown token rides through as itself rather than being picked against
    /// a table this build would have to grow for.
    pub marks: Vec<String>,
    /// The read-state commit the config derivations take. Empty for a
    /// conversation that has recorded none.
    pub tip: String,
    /// **The in-flight strip's characteristics, as prose.** Absent for a
    /// conversation at rest; carried as the engine wrote it.
    pub strip: Option<String>,
    /// **The §5.1 live mark**: one entry per agent in the conversation, named
    /// and with what it is doing.
    pub seats: Vec<SeatRow>,
    /// **What it has spent, rendered upstream.** Empty is a workspace with no
    /// price table — a fact, never a zero (`codec::balls`' rule).
    pub usd: String,
    /// How full the window is, when anything measured can be said.
    pub context: Option<Context>,
}

/// One agent of the conversation and what it is doing, in the engine's words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatRow {
    pub name: String,
    pub doing: String,
}

/// The context fullness: the model whose window it is, and the engine's own
/// percentage of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub model: String,
    pub percent: i64,
}

/// Read the `agent` answer.
pub(in super::super) fn agent_of(o: &Map<String, Value>) -> Result<Agent, String> {
    Ok(Agent {
        display: str_of(o, "display")?,
        root: str_of(o, "root")?,
        state: str_of(o, "state")?,
        flight: opt(o, "flight", str_of)?.unwrap_or_default(),
        present: bool_of(o, "present")?,
        refused: bool_of(o, "refused")?,
        failure: opt(o, "failure", str_of)?,
        marks: words(o, "marks")?,
        tip: str_of(o, "tip")?,
        strip: match o.get("strip") {
            None => None,
            Some(strip) => Some(str_of(&object(strip, "strip")?, "facts")?),
        },
        seats: seats(o)?,
        usd: match o.get("spend") {
            None => String::new(),
            Some(spend) => opt(&object(spend, "spend")?, "usd", str_of)?.unwrap_or_default(),
        },
        context: match o.get("context") {
            None => None,
            Some(full) => {
                let full = object(full, "context")?;
                Some(Context {
                    model: str_of(&full, "model")?,
                    percent: i64_of(&full, "percent")?,
                })
            }
        },
    })
}

/// The seats array, absent for a conversation whose mark is at rest.
fn seats(o: &Map<String, Value>) -> Result<Vec<SeatRow>, String> {
    let Some(_) = o.get("seats") else {
        return Ok(Vec::new());
    };
    super::super::fields::arr_of(o, "seats")?
        .iter()
        .map(|seat| {
            let seat = object(seat, "seats")?;
            Ok(SeatRow {
                name: str_of(&seat, "name")?,
                doing: str_of(&seat, "doing")?,
            })
        })
        .collect()
}

/// An optional array of bare strings — absent is none of them, which is what
/// the engine writes for a conversation carrying no marks at all.
pub(super) fn words(o: &Map<String, Value>, key: &str) -> Result<Vec<String>, String> {
    let Some(_) = o.get(key) else {
        return Ok(Vec::new());
    };
    super::super::fields::arr_of(o, key)?
        .iter()
        .map(|word| {
            word.as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("field {key:?}: a non-string entry"))
        })
        .collect()
}

/// A nested object, owned — rule 1's answer to handing a borrow back out.
pub(super) fn object(v: &Value, key: &str) -> Result<Map<String, Value>, String> {
    v.as_object()
        .cloned()
        .ok_or_else(|| format!("field {key:?} is not an object"))
}
