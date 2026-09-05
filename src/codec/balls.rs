//! **The ball pane's three reads** (yog §8.5, DESIGN §13.9): the task store
//! the conversations are working, which this seat carried the wire for and
//! painted nowhere.
//!
//! **Three reads at two widths, and the wire is what decides which.** `balls`
//! and `board` name no workspace — they are about everything this seat can
//! see — and `workspace-balls` names one, so it is asked under the focused
//! workspace and is unpaintable the moment the focus moves. That is the
//! §14 pairing law the cache already keeps, applied to a pane: rows are
//! paintable only under the view they were asked at, which is what [`Pane`]
//! carries its own [`View`] for.
//!
//! **Nothing here computes money and nothing re-derives a fleet's sentence**
//! (lernie DESIGN §4.31, which this pane is the twin of). `usd` was rendered
//! on the box that holds the price table, and a seat multiplying tokens by a
//! rate of its own would disagree with it quietly; a fleet's `label` is
//! upstream's own sentence about the cap, the tick and the lease. Both cross
//! as strings and are painted as they came. An absent figure is a fact and
//! never a zero — so it is an empty string here, which paints as nothing.
//!
//! **What is decoded is what this pane paints**, which is the codec's standing
//! grow-per-consumer rule (`codec.rs`). The token counters, the micro-dollar
//! integers and the attribution clauses ride in the same answers and are read
//! by nobody here; a phone that decoded them would be holding a ledger it has
//! no width to show.

use serde_json::{Map, Value};

use super::fields::{arr_of, i64_of, opt, str_of};

/// Which of the three reads a held answer came from — and, on the shell's
/// side, which one an operator opened. One enum for both, because the pane's
/// whole invariant is that those two agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// `balls`: every ball this seat can see, with the workspace holding it.
    Everywhere,
    /// `workspace-balls`: what the focused workspace holds.
    Here,
    /// `board`: the same table folded into columns, with the fleet lines.
    Board,
}

impl View {
    /// The screen this view paints — and the op it asks, which are one word
    /// because the pane has one control per read. Named here so the dispatch
    /// arm has one word to say and the probe (§15.2) derives no second one;
    /// `pub(crate)` and borrowed because the probe stores a `&'static str`.
    pub(crate) fn screen(self) -> &'static str {
        match self {
            Self::Everywhere => "balls",
            Self::Here => "workspace-balls",
            Self::Board => "board",
        }
    }
}

/// What the pane is holding, and which read answered it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pane {
    Everywhere(Vec<BallRow>),
    Here(Vec<WsBallRow>),
    Board(Board),
}

impl Pane {
    /// The view this answer belongs under. A pane holding another view's
    /// answer is not paintable — the same refusal `cache::read` makes of a
    /// depth that disagrees with its focus.
    #[must_use]
    pub fn view(&self) -> View {
        match self {
            Self::Everywhere(_) => View::Everywhere,
            Self::Here(_) => View::Here,
            Self::Board(_) => View::Board,
        }
    }
}

/// One ball, wherever it is (`balls`). Every field but the id and the state
/// may be absent — a ball nobody holds names no claimant and no workspace —
/// and absent reads as empty, which paints as nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BallRow {
    pub id: String,
    pub project: String,
    pub state: String,
    pub title: String,
    pub claimant: String,
    pub workspace: String,
}

/// One ball a workspace holds (`workspace-balls`), with the spend upstream
/// rendered for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WsBallRow {
    pub id: String,
    pub project: String,
    pub state: String,
    pub owner: String,
    pub badge: String,
    pub usd: String,
}

/// The board: the rows in their columns, and what each armed loop is doing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pub rows: Vec<BoardRow>,
    /// **Absent rather than empty when nothing is armed**, which is the one
    /// case where a `Vec` and an `Option<Vec>` are not two claims.
    pub fleet: Vec<String>,
}

/// One ball on the board, in the column the engine put it in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardRow {
    pub id: String,
    pub project: String,
    /// The engine's own column word, carried through: a column this build has
    /// not heard of still paints, because the word is the engine's to mint.
    pub column: String,
    pub title: String,
    pub claimant: String,
    pub priority: i64,
    /// The conversations working it, by the names an operator reads.
    pub drones: Vec<String>,
    /// The balls this one waits on, by id — the column says *gated*, and
    /// these say by what.
    pub gates: Vec<String>,
}

/// A `balls` row.
pub(super) fn row(v: &Value) -> Result<BallRow, String> {
    let o = &object(v, "balls")?;
    Ok(BallRow {
        id: str_of(o, "ball_id")?,
        project: str_of(o, "project")?,
        state: str_of(o, "state")?,
        title: said(o, "title"),
        claimant: said(o, "claimant"),
        workspace: said(o, "workspace"),
    })
}

/// A `workspace-balls` row.
pub(super) fn bound(v: &Value) -> Result<WsBallRow, String> {
    let o = &object(v, "workspace-balls")?;
    Ok(WsBallRow {
        id: str_of(o, "id")?,
        project: str_of(o, "project")?,
        state: str_of(o, "state")?,
        owner: said(o, "owner"),
        badge: said(o, "badge"),
        usd: o
            .get("spend")
            .and_then(Value::as_object)
            .map(|spend| said(spend, "usd"))
            .unwrap_or_default(),
    })
}

/// The whole `board` answer.
pub(super) fn board(o: &Map<String, Value>) -> Result<Board, String> {
    let rows = arr_of(o, "rows")?
        .iter()
        .map(column)
        .collect::<Result<Vec<BoardRow>, String>>()?;
    let fleet = match o.get("fleet") {
        None => Vec::new(),
        Some(_) => arr_of(o, "fleet")?
            .iter()
            .map(|line| object(line, "board").map(|loop_| said(&loop_, "label")))
            .collect::<Result<Vec<String>, String>>()?,
    };
    Ok(Board { rows, fleet })
}

/// One board row.
fn column(v: &Value) -> Result<BoardRow, String> {
    let o = &object(v, "board")?;
    Ok(BoardRow {
        id: str_of(o, "id")?,
        project: str_of(o, "project")?,
        column: str_of(o, "column")?,
        title: said(o, "title"),
        claimant: said(o, "claimant"),
        priority: i64_of(o, "priority")?,
        drones: names(o, "drones", "name")?,
        gates: names(o, "gates", "id")?,
    })
}

/// One string field of every object in an array field — the drones' names, the
/// gates' ids. An absent array is none of them, which is what the engine
/// writes for a row with neither.
fn names(o: &Map<String, Value>, key: &str, field: &str) -> Result<Vec<String>, String> {
    match o.get(key) {
        None => Ok(Vec::new()),
        Some(_) => arr_of(o, key)?
            .iter()
            .map(|each| object(each, key).and_then(|each| str_of(&each, field)))
            .collect(),
    }
}

/// A row, as an object, or the refusal naming the kind it was in.
///
/// **Owned, and the clone is the house standard's own answer** (rule 1):
/// handing a borrow back would need a NAME for its lifetime, and a row's map
/// is a handful of strings read once.
fn object(v: &Value, kind: &str) -> Result<Map<String, Value>, String> {
    v.as_object()
        .cloned()
        .ok_or_else(|| format!("{kind}: row is not an object"))
}

/// A string the engine may not have written. **Absence is a fact and never a
/// zero**: an unclaimed ball names no claimant and a workspace with no price
/// table renders no dollars, and both paint as nothing rather than as a value
/// this seat invented.
fn said(o: &Map<String, Value>, key: &str) -> String {
    opt(o, key, str_of).ok().flatten().unwrap_or_default()
}

#[cfg(test)]
mod tests;
