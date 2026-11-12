//! **The offer's four ops** (yog REMOTE §15.1 steps 1 and 5; DESIGN §13.18):
//! `offer` mints a one-time registration URL for a workspace, a name and a
//! grade; `offers` lists what is pending with each row's state and
//! confirmation word; `approve` seats a proved one; `decline` voids one.
//!
//! They replaced `enroll` and its envelope (REMOTE §15.4). What an operator
//! seat holds after minting is a URL that carries no key — the new device
//! makes its own (`crate::register`) — so nothing here is secret and nothing
//! has to be forgotten for safety's sake.
//!
//! **Grows-only on the way in** (REMOTE §3.2): a row's `state` this build
//! cannot spell is carried as the word it is and enables nothing, and a
//! `grade` word is said back as the server said it.

use serde_json::{Map, Value, json};

use super::Act;
use super::fields::{arr_of, i64_of, opt, str_of};
use crate::leaf::Grade;

/// What `offer` answered: the whole of what travels to the new device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offered {
    /// `https://<host>/yog/register/<code>` — good once.
    pub url: String,
    /// When an unclaimed code stops working, unix seconds.
    pub expires: i64,
}

/// Where a pending offer stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// Minted, and no device has claimed it.
    Open,
    /// A device claimed it and has not yet spoken on the wire.
    Claimed,
    /// The claimant dialled with its new certificate: the holder of the
    /// private key has spoken, and the row may be approved.
    Proved,
    /// A state this build has not heard of.
    Unknown(String),
}

impl State {
    /// The word the server said.
    pub fn word(&self) -> String {
        match self {
            Self::Open => "open".to_owned(),
            Self::Claimed => "claimed".to_owned(),
            Self::Proved => "proved".to_owned(),
            Self::Unknown(word) => word.clone(),
        }
    }
}

/// One pending offer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfferRow {
    pub name: String,
    /// The grade word, as the server said it.
    pub grade: String,
    pub state: State,
    pub expires: i64,
    /// The confirmation word — absent until a device has claimed.
    pub word: Option<String>,
}

impl OfferRow {
    /// **Whether `approve` may be fired at this row.** The server refuses an
    /// approval of anything but a proved offer; the control is dark for the
    /// same rows, so the refusal is never the first an operator hears of it.
    #[must_use]
    pub fn approvable(&self) -> bool {
        self.state == State::Proved
    }

    /// The row's one line: who, at what grade, where it stands, and the word
    /// the operator matches against the device in front of them.
    pub fn said(&self) -> String {
        let word = self
            .word
            .as_ref()
            .map_or_else(|| "no word yet".to_owned(), |word| format!("word {word}"));
        format!(
            "{} · {} · {} · {word}",
            self.name,
            self.grade,
            self.state.word()
        )
    }
}

/// **A workspace's pending offers, and the URL this seat just minted** — one
/// value because they are one surface (DESIGN §13.18), carrying the workspace
/// it was read for: the answer echoes none, so a listing must not paint under
/// another workspace's name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pending {
    pub workspace: String,
    /// The name and URL of the offer minted here, while its surface stands.
    pub minted: Option<(String, Offered)>,
    pub rows: Vec<OfferRow>,
}

impl Pending {
    /// Whether this is about the workspace now focused.
    #[must_use]
    pub fn about(&self, workspace: &str) -> bool {
        self.workspace == workspace
    }
}

/// Which of the two acts that settle a pending offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Settle {
    Approve,
    Decline,
}

impl Settle {
    /// The op, which is also the control's tag.
    pub fn op(self) -> String {
        match self {
            Self::Approve => "approve".to_owned(),
            Self::Decline => "decline".to_owned(),
        }
    }
}

/// The offer's acts: the mint, and the two that settle a pending one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OfferAct {
    /// Mint a one-time registration URL under a workspace, a name and a grade.
    Mint {
        workspace: String,
        name: String,
        grade: Grade,
    },
    /// Approve or decline the pending offer under `name`.
    Settle {
        settle: Settle,
        workspace: String,
        name: String,
    },
}

pub(crate) fn encode(act: &OfferAct) -> Value {
    match act {
        OfferAct::Mint {
            workspace,
            name,
            grade,
        } => json!({ "op": "offer", "workspace": workspace, "name": name, "grade": word(grade) }),
        OfferAct::Settle {
            settle,
            workspace,
            name,
        } => json!({ "op": settle.op(), "workspace": workspace, "name": name }),
    }
}

/// `offer`, read back — the conformance replay's door.
pub(crate) fn decode_offer(o: &Map<String, Value>) -> Result<Act, String> {
    Ok(Act::Offer(OfferAct::Mint {
        workspace: str_of(o, "workspace")?,
        name: str_of(o, "name")?,
        grade: match str_of(o, "grade")?.as_str() {
            "foot" => Grade::Foot,
            "operator" => Grade::Operator,
            other => Grade::Unknown(other.to_owned()),
        },
    }))
}

/// `approve` and `decline`, read back.
pub(crate) fn decode_settle(settle: Settle, o: &Map<String, Value>) -> Result<Act, String> {
    Ok(Act::Offer(OfferAct::Settle {
        settle,
        workspace: str_of(o, "workspace")?,
        name: str_of(o, "name")?,
    }))
}

/// The `offered` reply.
pub(super) fn offered(o: &Map<String, Value>) -> Result<Offered, String> {
    Ok(Offered {
        url: str_of(o, "url")?,
        expires: i64_of(o, "expires")?,
    })
}

/// The `offers` reply's rows.
pub(super) fn rows(o: &Map<String, Value>) -> Result<Vec<OfferRow>, String> {
    arr_of(o, "rows")?.iter().map(row).collect()
}

fn row(v: &Value) -> Result<OfferRow, String> {
    let o = v
        .as_object()
        .ok_or_else(|| "offers: row is not an object".to_owned())?;
    Ok(OfferRow {
        name: str_of(o, "name")?,
        grade: str_of(o, "grade")?,
        state: match str_of(o, "state")?.as_str() {
            "open" => State::Open,
            "claimed" => State::Claimed,
            "proved" => State::Proved,
            other => State::Unknown(other.to_owned()),
        },
        expires: i64_of(o, "expires")?,
        word: opt(o, "word", str_of)?,
    })
}

/// The word a grade wears on the wire.
pub(crate) fn word(grade: &Grade) -> String {
    match grade {
        Grade::Foot => "foot".to_owned(),
        Grade::Operator => "operator".to_owned(),
        Grade::Unknown(word) => word.clone(),
    }
}

#[cfg(test)]
mod tests;
