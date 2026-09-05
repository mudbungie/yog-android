//! **The search answer** (yog DESIGN §8.5, REMOTE §8.1) — the one read this
//! seat makes that is not about a place it is already looking.
//!
//! The request is one word (`{"op":"search","text":…}`) and the answer is a
//! ranked list of addresses the engine already selects by: a ball, a
//! workspace, a conversation. Nothing here is a coordinate invented for
//! search, which is what lets a hit be fed straight back as a focus — and
//! since upstream's bl-764a every address is a **wire name** (the §5.1 project
//! name, the §3.1 workspace leaf, the agent id) rather than an engine-local
//! path, which is what made the answer usable off the box at all.
//!
//! **The answer carries its own question** (upstream bl-648a). Without the
//! needle, "was a search asked?" and "did anything match?" would be the same
//! value exactly when a search found nothing — the one case that must be told
//! apart — so `needle` rides back and an empty one is the engine's own
//! spelling of *no search*.
//!
//! **`unreadable` is not an error and never a refusal.** A corner of the world
//! that could not be read shrinks the corpus; it does not make the world
//! unsearchable. Both halves ride back together and this seat paints both.

use serde_json::{Map, Value};

use super::fields::{arr_of, str_of, usize_of};

/// A whole answer: the question, the ranked hits, and what could not be read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Found {
    /// The needle the engine searched for, as it read it. Empty means no
    /// search was made.
    pub needle: String,
    pub hits: Vec<Hit>,
    /// Each source that could not be read, named with why.
    pub unreadable: Vec<String>,
}

/// One result: where it is, which field matched, where in that field, and the
/// matched line as the operator reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub at: Address,
    pub field: HitField,
    /// Byte offset of the match within the matched field's own text.
    pub offset: usize,
    pub excerpt: String,
}

/// Where a hit lives. **The `at` token is why the shape is readable at all**:
/// the keys are flattened onto the same words every gesture takes, so a
/// workspace hit and a conversation hit that named no agent would be
/// indistinguishable by keys alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Address {
    /// A ball in a project — the one address this seat has no surface for
    /// (the ball pane is bl-d587), so it is painted and not tappable.
    Ball { project: String, id: String },
    /// An enumerated workspace, by its leaf name: a focus this seat takes.
    Workspace { name: String },
    /// A conversation in a workspace: the deeper focus, also taken.
    Conversation { workspace: String, agent: String },
}

/// Which field matched — and, by the engine's own ranking, the tier: what a
/// thing **is** beats what it is **for** beats what it **says**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitField {
    /// A ball id, a workspace name, a conversation name or agent id.
    Name,
    /// A ball title, a conversation's goal.
    Summary,
    /// The bulk: a ball body, a transcript entry's bytes.
    Text,
}

impl HitField {
    /// Every tier, in the engine's own ranking order — what the decode
    /// searches and what a caller enumerates.
    pub(crate) const ALL: [Self; 3] = [Self::Name, Self::Summary, Self::Text];

    /// The engine's own word for this tier. **One home for the three words**:
    /// the decode below finds a token by asking each variant what it is
    /// called, and the shell paints the same word beside the hit — so a row
    /// whose excerpt repeats its own subject (a name matching a name) says
    /// why it is there rather than looking like a duplicate.
    pub(crate) fn word(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Summary => "summary",
            Self::Text => "text",
        }
    }
}

/// Read a whole search answer out of its envelope.
pub(crate) fn found_of(o: &Map<String, Value>) -> Result<Found, String> {
    Ok(Found {
        needle: str_of(o, "needle").map_err(named)?,
        hits: arr_of(o, "rows")
            .map_err(named)?
            .iter()
            .map(hit)
            .collect::<Result<Vec<Hit>, String>>()?,
        unreadable: arr_of(o, "unreadable")
            .map_err(named)?
            .iter()
            .map(|v| {
                v.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| named("non-string in \"unreadable\""))
            })
            .collect::<Result<Vec<String>, String>>()?,
    })
}

/// One hit, strictly — the address token first, because it says which of the
/// flat keys this row actually spells.
fn hit(v: &Value) -> Result<Hit, String> {
    let o = v.as_object().ok_or_else(|| named("hit is not an object"))?;
    let at = match str_of(o, "at").map_err(named)?.as_str() {
        "ball" => Address::Ball {
            project: str_of(o, "project").map_err(named)?,
            id: str_of(o, "id").map_err(named)?,
        },
        "workspace" => Address::Workspace {
            name: str_of(o, "workspace").map_err(named)?,
        },
        "conversation" => Address::Conversation {
            workspace: str_of(o, "workspace").map_err(named)?,
            agent: str_of(o, "agent").map_err(named)?,
        },
        other => return Err(named(format!("hit at unknown address {other:?}"))),
    };
    let word = str_of(o, "field").map_err(named)?;
    let field = HitField::ALL
        .into_iter()
        .find(|tier| tier.word() == word)
        .ok_or_else(|| named(format!("hit in unknown field {word:?}")))?;
    Ok(Hit {
        at,
        field,
        offset: usize_of(o, "offset").map_err(named)?,
        excerpt: str_of(o, "excerpt").map_err(named)?,
    })
}

/// Every refusal from this file names the shape it refused, which is what
/// REMOTE §3's third rule asks of a client that skips a fixture — and what
/// the conformance replay asserts.
fn named(why: impl std::fmt::Display) -> String {
    format!("search: {why}")
}

#[cfg(test)]
mod tests;
