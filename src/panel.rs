//! **The conversation panel** (DESIGN §22 item 4; lernie `ui::chat::panel`):
//! the framed card at the head of a transcript, the conversation's facts one
//! line each — its name, the workspace it is on, the model that answered and
//! the tokens it has spent, and where it hangs in its subtree. Pure and
//! host-tested; `shell/chat/panel.rs` paints it.
//!
//! **A long fact stands behind `+`**: at a phone's width a subtree path or a
//! model id can be most of a line, so a fact longer than [`LONG`] characters
//! is painted elided with the fold glyph and unfolds in place. Which one is
//! open is viewport state and dies with the screen, like every fold.
//!
//! With nothing focused there is no conversation to describe; the seat's own
//! panel stands at the switcher's head (bl-3edc) and is not repeated here.

use crate::codec::{ConvRow, Entry, EntryKind};
use crate::theme::{self, Rgb};

/// The characters a fact may run to before it folds behind `+`.
pub const LONG: usize = 36;

/// What parts the steps of where a conversation hangs.
const DESCENT: &str = " / ";

/// One fact on the card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fact {
    pub text: String,
    pub ink: Rgb,
    /// Whether it stands behind `+` until unfolded.
    pub long: bool,
}

impl Fact {
    fn of(text: String, ink: Rgb) -> Self {
        Self {
            long: text.chars().count() > LONG,
            text,
            ink,
        }
    }
}

/// **The card's facts**, in order. `rows` is the focused workspace's list in
/// the engine's descent, `agent` the focused conversation's id.
pub fn conversation(
    rows: &[ConvRow],
    agent: &str,
    workspace: &str,
    transcript: &[Entry],
) -> Vec<Fact> {
    let at = rows.iter().position(|row| row.root_id == agent);
    let name = at
        .and_then(|at| rows.get(at))
        .map_or_else(|| agent.to_owned(), |row| row.display.clone());
    let mut out = vec![
        Fact::of(name, theme::INK),
        Fact::of(format!("on {workspace}"), theme::INK_WEAK),
    ];
    let mut cost = Vec::new();
    if let Some(model) = answered_by(transcript) {
        cost.push(model);
    }
    if let Some((input, output)) = crate::status::tokens(transcript) {
        cost.push(format!(
            "{} in · {} out",
            crate::status::count(input),
            crate::status::count(output)
        ));
    }
    if !cost.is_empty() {
        out.push(Fact::of(cost.join(" · "), theme::INK_WEAK));
    }
    if let Some(path) = at.and_then(|at| hangs(rows, at)) {
        out.push(Fact::of(format!("under {path}"), theme::INK_WEAK));
    }
    out
}

/// The model that last answered on the transcript, where one did.
fn answered_by(transcript: &[Entry]) -> Option<String> {
    transcript.iter().rev().find_map(|entry| match &entry.kind {
        EntryKind::Model { model_id, .. } => Some(model_id.clone()),
        _ => None,
    })
}

/// **Where the row at `at` hangs**: its ancestors' names, root first, or
/// `None` for a root. The list is the engine's pre-order descent (§13.20), so
/// a row's parent is the nearest row above it one level shallower.
fn hangs(rows: &[ConvRow], at: usize) -> Option<String> {
    let mut depth = rows.get(at)?.depth;
    let mut path = Vec::new();
    for row in rows.get(..at)?.iter().rev() {
        if depth > 0 && row.depth + 1 == depth {
            path.push(row.display.clone());
            depth = row.depth;
        }
    }
    path.reverse();
    (!path.is_empty()).then(|| path.join(DESCENT))
}

#[cfg(test)]
mod tests;
