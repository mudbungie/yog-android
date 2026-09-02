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

use super::fields::{opt, str_of};

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
    })
}

/// The models listing: bare names, in the engine's order.
pub(crate) fn names(o: &Map<String, Value>) -> Result<Vec<String>, String> {
    super::fields::arr_of(o, "rows")?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .ok_or_else(|| "models: non-string row".to_owned())
        })
        .collect()
}

/// `{"op":"model", …}` — the pick, stated whole.
pub(crate) fn encode_pick(workspace: &str, role: &str, provider: &str, model: &str) -> Value {
    json!({ "op": "model", "workspace": workspace, "role": role,
            "provider": provider, "model": model })
}

#[cfg(test)]
mod tests;
