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
//! with the state. The engine still decides whether a wide answer is allowed,
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
/// How many cells stand on one row: six at 48 points apiece would be more
/// than the room above a keyboard (§22 item 8).
pub const ACROSS: usize = 2;

/// **One choice**: `seen`, or a verdict at a scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Choice {
    Seen,
    Answer(Verdict, Scope),
}

impl Choice {
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
    if row.held.is_some() {
        out.extend(Scope::ALL.map(|scope| Choice::Answer(Verdict::Pass, scope)));
        out.push(Choice::Answer(Verdict::Refuse, Scope::Call));
        out.push(Choice::Answer(Verdict::Hold, Scope::Call));
    }
    out
}

/// **The cells, numbered from one and laid `ACROSS` to a row.** The numbers
/// are the desktop's spelling and carry no input meaning here.
#[must_use]
pub fn cells(row: &QueueRow) -> Vec<Vec<(String, Choice)>> {
    let numbered: Vec<(String, Choice)> = choices(row)
        .into_iter()
        .enumerate()
        .map(|(at, choice)| (format!("{}. {}", at + 1, choice.word()), choice))
        .collect();
    numbered.chunks(ACROSS).map(<[_]>::to_vec).collect()
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

/// **The parked call, folded at the engine's clause** (`codec::queue`): the
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
    let mut out = Vec::new();
    if let Some(flag) = &row.flag {
        out.push((
            format!("flagged · {}", flag.reason),
            theme::accent(State::Annotation),
        ));
    }
    let why = if row.says.is_empty() {
        row.signals.join(" · ")
    } else {
        row.says.clone()
    };
    if !why.is_empty() {
        out.push((why, theme::INK_WEAK));
    }
    if let Some(failure) = &row.failure {
        out.push((format!("failed · {failure}"), theme::accent(State::Error)));
    }
    if !row.preview.is_empty() {
        out.push((row.preview.clone(), theme::INK));
    }
    out
}

#[cfg(test)]
mod tests;
