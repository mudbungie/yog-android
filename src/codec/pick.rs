//! **The provider/model family** (bl-0267): the two reads that populate the
//! composer's selectors, and the one act that spends them.
//!
//! Three ops, and the shape of the three is the whole design. `providers`
//! and `models` are reads **of a workspace** — provider sign-ins live per
//! workspace upstream, so neither is a global fact this device could cache
//! once and spend everywhere. `model` is the act, and it names all four of
//! role, provider, model and workspace: a pick is not a provider and then a
//! model, it is one assignment stated whole, which is why the glass makes the
//! provider a *path* and the model tap the act (DESIGN §13.2).
//!
//! **A provider row's credential fact is the engine's sentence, carried and
//! never re-derived.** `fact` is what the row says about itself and `blocked`
//! is present only when something stops it being used; the glass greys a row
//! by them and states them, and computes neither.
//!
//! **What this family does NOT carry is the current assignment.** No shape
//! here answers *which* model the workspace is on — the pick lives in the
//! workspace's own config file, which is the engine's to read. So the seat
//! shows what it SET (a fact it owns, because it just did it) and never a
//! guess at what is set (DESIGN §8's rule against inventing world state).

use serde_json::{Map, Value, json};

use super::fields::{bool_of, opt, str_of};

/// One provider as the engine lists it — the name it is picked by, and the
/// two facts a surface may state about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRow {
    /// The provider's name, and what a pick names it by.
    pub name: String,
    /// Its credential standing, in the engine's own words.
    pub fact: String,
    /// Why it cannot be used, when something does — a real null otherwise.
    pub blocked: Option<String>,
    /// **Whether this provider can be asked for a reasoning level** (REMOTE
    /// §9.4's tuning pair, bl-dfbb). The capability is the engine's to state
    /// per provider and this seat never derives it: a control shown for a
    /// provider that cannot take the setting is a control that earns a
    /// refusal, and §8's rule is that a client re-deriving world state is
    /// inventing it.
    pub effort: bool,
    /// Whether it can be asked for its priority lane.
    pub priority: bool,
}

/// **A reasoning level**, in the three words the wire, the config and the
/// slash line all spell it with. `off` is not a fourth word here for the
/// reason it is not one upstream: it is the ABSENCE of a level, carried as a
/// real null, so the vocabulary has three members and the option type says
/// the rest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effort {
    Low,
    Medium,
    High,
}

/// The four choices a seat offers, `off` last — the vocabulary is closed and
/// fixed, so it is stated here rather than read from a wire that carries no
/// listing of it.
pub const LEVELS: [Option<Effort>; 4] = [
    Some(Effort::Low),
    Some(Effort::Medium),
    Some(Effort::High),
    None,
];

impl Effort {
    /// The word this level is written as, on the wire and on the glass.
    pub fn as_str(&self) -> String {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
        .to_owned()
    }

    /// One level word read back, or `None` for anything else — including
    /// `off`, which is not a level but the absence of one and rides as a
    /// real null.
    pub fn parse(word: &str) -> Option<Self> {
        match word {
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            _ => None,
        }
    }

    /// What a chooser shows for a level or its absence.
    pub fn label(level: Option<Self>) -> String {
        level.map_or_else(|| "off".to_owned(), |level| level.as_str())
    }
}

/// **What a role is actually set to** (REMOTE §9.4's read, bl-e9f9): one row
/// per role the workspace's lineage tip assigns, read from the same place the
/// tuning gestures write — so a seat reads its own write back.
///
/// **`effort` is the FILE's own word, not this codec's vocabulary.** The
/// config may hold a level the gesture set does not spell, and flattening
/// such a word to absent would say *nothing is set* — the exact thing this
/// read exists to end. So it rides as the string it is, and a surface shows
/// it as itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleRow {
    pub role: String,
    pub provider: String,
    pub model: String,
    pub effort: Option<String>,
    /// Plain, because `false` is what an omitted line means upstream.
    pub priority: bool,
}

/// One assignment row.
pub(crate) fn role(v: &Value) -> Result<RoleRow, String> {
    let o = v.as_object().ok_or("role row: not a JSON object")?;
    Ok(RoleRow {
        role: str_of(o, "role")?,
        provider: str_of(o, "provider")?,
        model: str_of(o, "model")?,
        effort: opt(o, "effort", str_of)?,
        priority: bool_of(o, "priority")?,
    })
}

/// **The row a phone tunes**: the worker's, or none — an engine that has
/// nothing set answers the empty list rather than refusing, so "no rows" is
/// an answer and not a failure.
pub fn worker(rows: &[RoleRow]) -> Option<RoleRow> {
    rows.iter().find(|row| row.role == WORKER).cloned()
}

/// The one role a phone assigns, named here because the read and the two
/// gestures must agree about which row is theirs.
pub const WORKER: &str = "worker";

/// **What the selected provider will take** (bl-dfbb): the two capability
/// booleans off the row the engine listed, and both false for a provider
/// this seat has not picked or does not know. It is a read of the wire's own
/// row and never a guess — the same fact `blocked` greying spends, asked a
/// different way — and it lives here rather than in the paint because the
/// paint is not under the coverage floor.
pub fn tunable(rows: &[ProviderRow], provider: Option<&str>) -> (bool, bool) {
    let Some(provider) = provider else {
        return (false, false);
    };
    rows.iter()
        .find(|row| row.name == provider)
        .map_or((false, false), |row| (row.effort, row.priority))
}

/// The role a pick assigns. The wire takes a free token; this seat spends
/// exactly one of them and says so at the call site, but the field rides as
/// what the engine wrote so a frame naming another role round-trips whole
/// rather than being flattened into this device's one.
pub(crate) fn row(v: &Value) -> Result<ProviderRow, String> {
    let o = v.as_object().ok_or("provider row: not a JSON object")?;
    Ok(ProviderRow {
        name: str_of(o, "name")?,
        fact: str_of(o, "fact")?,
        blocked: opt(o, "blocked", str_of)?,
        effort: bool_of(o, "effort")?,
        priority: bool_of(o, "priority")?,
    })
}

/// The models listing: bare names, in the engine's order.
pub(crate) fn names(o: &Map<String, Value>) -> Result<Vec<String>, String> {
    super::fields::strings_of(o, "rows", "models")
}

/// `{"op":"effort", …}` — the level, or a real null for off. The key is
/// written always: a peer that omits it has said the one thing an absent
/// optional can honestly mean, which is the same thing null says.
pub(crate) fn encode_effort(workspace: &str, role: &str, level: Option<Effort>) -> Value {
    json!({ "op": "effort", "workspace": workspace, "role": role,
            "level": level.map(|level| level.as_str()) })
}

/// `{"op":"priority", …}` — a checkbox and not a tri-state: off removes the
/// line, because asking for the *standard* lane is a different intent that no
/// config key expresses (REMOTE §9.4).
pub(crate) fn encode_priority(workspace: &str, role: &str, on: bool) -> Value {
    json!({ "op": "priority", "workspace": workspace, "role": role, "on": on })
}

/// One level word off a request envelope, strictly: the vocabulary is closed,
/// so a word outside it is a codec that has drifted rather than a typo.
pub(crate) fn level_of(o: &Map<String, Value>) -> Result<Option<Effort>, String> {
    match opt(o, "level", str_of)? {
        None => Ok(None),
        Some(word) => Effort::parse(&word).map(Some).ok_or_else(|| {
            format!("effort: level must be one of low|medium|high|off, got {word:?}")
        }),
    }
}

/// `{"op":"model", …}` — the pick, stated whole.
pub(crate) fn encode_pick(workspace: &str, role: &str, provider: &str, model: &str) -> Value {
    json!({ "op": "model", "workspace": workspace, "role": role,
            "provider": provider, "model": model })
}

#[cfg(test)]
mod tests;
