//! **The ask** (DESIGN §22 item 8, bl-289f): the focused conversation's own
//! row in what is waiting, as the box that stands between the transcript and
//! the rule — which choices it offers, the words on each, and the queue row's
//! words above them. Pure and host-tested; `shell/ask.rs` paints the box and
//! spends a tap. The desktop original is lernie's `ui::ask` (its §4.42 item
//! 8), whose words these are.
//!
//! **The choices are the queue's own acts and no others**: `seen`, and where
//! a call is parked the three verdicts — `pass` for this call, for this
//! conversation and for this workspace (PROTOCOL 18's scopes), then `refuse`
//! and `hold`, which settle the call alone. **A tap is the answer**: the
//! verdict and its reach are one cell, so §13.7's scope chooser — and the
//! *reset when the parked call changes* rule that guarded its state — is gone
//! with the state. The server still decides whether a wide answer is allowed,
//! and says so in band. `go to it` is not among them: the operator is there.
//!
//! **It stands while the focused conversation's row is in the queue.** That
//! is the desktop's `Standing::asking`; here the queue is the attention
//! lane's, which this seat holds at every depth (§14.1), so its read already
//! stands wherever the box can.

use crate::codec::queue::{Folded, folded};
use crate::codec::{QueueRow, Scope, Verdict};
use crate::theme::{self, Rgb, State};

/// The box's name, in the attention accent.
pub const HEADING: &str = "it is asking";
/// The fold's word on the box's first line, beside the fold glyph: what it
/// opens is the control's decision about the parked call (§13.7, bl-8c94).
pub const WHY: &str = "why";
/// **The narrowest a cell may be for three to stand on a row** (bl-58ca):
/// `3. pass for this` at `SMALL` type (88.7 points in the bundled face) and
/// the button's padding either side — so every cell's words wrap onto at most
/// two small lines inside its 48 points and none is elided, because
/// `pass for this…` would make cells 3 and 4 the same words.
pub const CELL: f32 = 104.0;

/// **One choice**: `seen`, or a verdict at a scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Choice {
    Seen,
    Answer(Verdict, Scope),
}

impl Choice {
    /// **What the cell does**, told on a long press (DESIGN §23 item 10).
    #[must_use]
    pub fn help(&self) -> String {
        match self {
            Self::Seen => "marks this row seen; nothing is answered",
            Self::Answer(Verdict::Pass, Scope::Call) => "lets this one call run",
            Self::Answer(Verdict::Pass, Scope::Conversation) => {
                "lets this call's class run for this conversation from now on"
            }
            Self::Answer(Verdict::Pass, _) => {
                "lets this call's class run for every conversation in the workspace"
            }
            Self::Answer(Verdict::Refuse, _) => "refuses this call; the turn goes on without it",
            Self::Answer(Verdict::Hold, _) => "keeps the call parked for later",
            // A verdict this build cannot spell is never offered as a cell
            // (`choices` lists three); said as itself where it ever is.
            Self::Answer(Verdict::Unknown(_), _) => "answers in a word this build cannot spell",
        }
        .to_owned()
    }

    /// The words on the cell, as the desktop spells them: a verdict on the
    /// call alone is its bare word, a wider one says its reach.
    #[must_use]
    pub fn word(&self) -> String {
        match self {
            Self::Seen => "seen".to_owned(),
            Self::Answer(verdict, Scope::Call) => verdict.word(),
            Self::Answer(verdict, scope) => format!("{} for this {}", verdict.word(), scope.word()),
        }
    }

    /// The op the cell fires, which is its parity tag.
    #[must_use]
    pub fn op(&self) -> String {
        match self {
            Self::Seen => "seen",
            Self::Answer(..) => "answer",
        }
        .to_owned()
    }
}

/// **The choices a row offers**, in the order they are numbered.
#[must_use]
pub fn choices(row: &QueueRow) -> Vec<Choice> {
    let mut out = vec![Choice::Seen];
    if let Some(held) = &row.held {
        // **A pass is offered only at the scopes the server lists** (DESIGN
        // §23 item 9; lernie bl-da16): a tap is never an answer the server
        // will refuse in band. A row that lists none is an older server's,
        // and every reach stands as it did before the key existed.
        let permits = |scope: &Scope| held.scopes.is_empty() || held.scopes.contains(scope);
        out.extend(
            Scope::ALL
                .into_iter()
                .filter(permits)
                .map(|scope| Choice::Answer(Verdict::Pass, scope)),
        );
        out.push(Choice::Answer(Verdict::Refuse, Scope::Call));
        out.push(Choice::Answer(Verdict::Hold, Scope::Call));
    }
    out
}

/// **How many cells stand on one row** of `room` points with `gap` between
/// neighbours: three where a third is at least [`CELL`] (at `SMALL` type),
/// else two — six at 48 points apiece would be more than the room above a
/// keyboard (§22 item 8), and three rows of two took a third of the glass.
#[must_use]
pub fn across(room: f32, gap: f32) -> usize {
    if (room - gap * 2.0) / 3.0 >= CELL {
        3
    } else {
        2
    }
}

/// **The cells, numbered from one and laid `across` to a row.** The numbers
/// are the desktop's spelling and carry no input meaning here.
#[must_use]
pub fn cells(row: &QueueRow, across: usize) -> Vec<Vec<(String, Choice)>> {
    let numbered: Vec<(String, Choice)> = choices(row)
        .into_iter()
        .enumerate()
        .map(|(at, choice)| (format!("{}. {}", at + 1, choice.word()), choice))
        .collect();
    numbered.chunks(across.max(1)).map(<[_]>::to_vec).collect()
}

/// **The focused conversation's own row**, where the queue carries one —
/// both keys, because an agent id is unique only inside its workspace.
#[must_use]
pub fn asking(queue: &[QueueRow], workspace: &str, agent: &str) -> Option<QueueRow> {
    queue
        .iter()
        .find(|row| row.workspace == workspace && row.agent == agent)
        .cloned()
}

/// **The parked call, folded at the server's clause** (`codec::queue`): the
/// line the box says and the decision its `why` opens onto. None where
/// nothing is parked, and then the box has no fold.
#[must_use]
pub fn parked(row: &QueueRow) -> Option<Folded> {
    let held = row.held.as_ref()?;
    Some(folded(&held.tool, &held.reason))
}

/// **What the row says after the parked call**, in the queue pane's words
/// and in the inks the desktop gives them: the flag, why it fires, the
/// failure, the last words.
#[must_use]
pub fn words(row: &QueueRow) -> Vec<(String, Rgb)> {
    parts(row).into_iter().flatten().collect()
}

/// [`words`] by place — the flag, why it fires, the failure, the last
/// words — each absent where the row says nothing of it.
fn parts(row: &QueueRow) -> [Option<(String, Rgb)>; 4] {
    let why = if row.says.is_empty() {
        row.signals.join(" · ")
    } else {
        row.says.clone()
    };
    [
        row.flag.as_ref().map(|flag| {
            (
                format!("flagged · {}", flag.reason),
                theme::accent(State::Annotation),
            )
        }),
        (!why.is_empty()).then_some((why, theme::INK_WEAK)),
        row.failure
            .as_ref()
            .map(|failure| (format!("failed · {failure}"), theme::accent(State::Error))),
        (!row.preview.is_empty()).then(|| (row.preview.clone(), theme::INK)),
    ]
}

/// **The one line that stands under the name, folded or not** (bl-58ca):
/// the parked call and its class, else the failure, else the last words,
/// else whatever the row says first. None where the row says nothing.
#[must_use]
pub fn summary(row: &QueueRow) -> Option<(String, Rgb)> {
    if let Some(folded) = parked(row) {
        return Some((folded.line, theme::accent(State::Attention)));
    }
    let [flag, why, failure, preview] = parts(row);
    failure.or(preview).or(flag).or(why)
}

/// **What the name line's fold opens onto**, under the summary: the row's
/// words less the one the summary already says. The parked call's decision
/// is the fold's too, read off [`parked`].
#[must_use]
pub fn behind(row: &QueueRow) -> Vec<(String, Rgb)> {
    let first = summary(row);
    words(row)
        .into_iter()
        .filter(|said| Some(said) != first.as_ref())
        .collect()
}

/// Whether the name line carries a fold: something stands behind it.
#[must_use]
pub fn folds(row: &QueueRow) -> bool {
    parked(row).is_some() || !behind(row).is_empty()
}

#[cfg(test)]
mod tests;
