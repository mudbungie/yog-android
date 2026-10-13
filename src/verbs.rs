//! **The slash verbs** (DESIGN §22 items 3 and 5, bl-0d15): every way into a
//! surface and every act on the focused conversation, as rows of one verb
//! sheet, and the line a draft that begins with `/` is read as. Pure and
//! host-tested; `shell/slash.rs` paints the sheet and performs what a row
//! does. The desktop's shape, kept (lernie `ui::slash`): parsed → rows → fire.
//!
//! **The sheet is open exactly while the draft begins with `/`** — derived
//! from the field's text, never a flag beside it. The prompt glyph is the
//! door: it puts the `/` at the head of the draft (or takes it off), and
//! whatever was already typed stays after it as the verb's words — §13.5's
//! *the composer is the parameter*: type the reason, tap `›`, tap `flag`.
//!
//! **A row's word is the word its control carried**, and the `act:` tags move
//! with it: nothing is renamed by moving a control into the sheet.

mod table;

pub use table::{Gates, Offers, rows};

use crate::overlay::Overlay;

/// The character a slash line starts with.
pub const LEAD: char = '/';

/// What taking a row does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Does {
    /// Float an overlay (§22 item 6).
    Open(Overlay),
    /// Search for the words (§13.6).
    Find,
    /// Stop the focused turn, or every turn under it.
    Stop {
        children: bool,
    },
    /// Wake a branch that stopped advancing.
    Nudge,
    /// One of the row acts (§13.5), addressed at the focused conversation.
    Interrupt,
    Retarget,
    Flag,
    Revoke,
    Restore,
    /// Open one of the rule's knobs (§22 item 2): the model's or effort's
    /// dropup, or flip priority — each reachable here however narrow the
    /// rule is.
    Knob(crate::status::Slot),
    /// Flip a transcript policy: show full responses, show steps.
    Full,
    Steps,
}

/// **One row of the sheet**: the word typed, what it does in the engine's own
/// words where it has them, the ops its firing spends (its `act:` tags), and
/// what words it needs, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub word: &'static str,
    pub summary: String,
    pub ops: &'static [&'static str],
    pub does: Does,
    /// What the verb's words are, said where they are missing.
    pub wants: Option<&'static str>,
}

impl Row {
    /// The row as one run of text: the word, then its summary.
    pub fn line(&self) -> String {
        format!("{LEAD}{}  {}", self.word, self.summary)
    }
}

/// What a tap or a send yields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fired {
    /// Do it, with these words; the draft is spent.
    Fire(Does, String),
    /// Complete the draft to this line and wait for the words and a send.
    Complete(String),
    /// Fire nothing, and say why.
    Refused(String),
}

/// Whether the sheet is open: the draft begins with `/`.
pub fn slashing(draft: &str) -> bool {
    draft.starts_with(LEAD)
}

/// **The prompt's door**: a `/` at the head of the draft, keeping what was
/// typed after it — or, with the sheet open, the `/` taken back off.
pub fn door(draft: &str) -> String {
    draft
        .strip_prefix(LEAD)
        .map_or_else(|| format!("{LEAD}{draft}"), str::to_owned)
}

/// **What a slash line names**: the longest row word the line begins with, at
/// a word's end, and the words after it.
pub fn parsed(rows: &[Row], draft: &str) -> Option<(Row, String)> {
    let line = draft.strip_prefix(LEAD)?.trim_start();
    rows.iter()
        .filter_map(|row| {
            let rest = line.strip_prefix(row.word)?;
            (rest.is_empty() || rest.starts_with(char::is_whitespace))
                .then(|| (row.clone(), rest.trim().to_owned()))
        })
        .max_by_key(|(row, _)| row.word.len())
}

/// **The rows the line narrows to**: those its first word begins; every row
/// when it begins none, because then the line is the words of a verb not yet
/// chosen; and the named row, once a line names one.
pub fn matching(rows: &[Row], draft: &str) -> Vec<Row> {
    if let Some((row, _)) = parsed(rows, draft) {
        return rows
            .iter()
            .filter(|r| r.word.starts_with(row.word))
            .cloned()
            .collect();
    }
    let typed = first(draft);
    let narrowed: Vec<Row> = rows
        .iter()
        .filter(|row| row.word.starts_with(typed))
        .cloned()
        .collect();
    if narrowed.is_empty() {
        rows.to_vec()
    } else {
        narrowed
    }
}

/// The first word after the slash.
fn first(draft: &str) -> &str {
    let line = draft.strip_prefix(LEAD).unwrap_or(draft).trim_start();
    line.split(char::is_whitespace).next().unwrap_or_default()
}

/// **A tap on a row.** The words are what follows the verb when the line names
/// this row, what follows a partial word of it, or else the whole line (what
/// was typed before the door was tapped).
pub fn tap(row: &Row, rows: &[Row], draft: &str) -> Fired {
    let line = draft.strip_prefix(LEAD).unwrap_or(draft).trim_start();
    let words = match parsed(rows, draft) {
        Some((named, rest)) if named.word == row.word => rest,
        _ => match line.strip_prefix(first(draft)) {
            Some(rest) if !first(draft).is_empty() && row.word.starts_with(first(draft)) => {
                rest.trim().to_owned()
            }
            _ => line.trim().to_owned(),
        },
    };
    match row.wants {
        Some(_) if words.is_empty() => Fired::Complete(format!("{LEAD}{} ", row.word)),
        _ => Fired::Fire(row.does, words),
    }
}

/// **`send` on a slash line.** A line that names no row fires nothing, and a
/// verb missing its words fires nothing; each says why.
pub fn send(rows: &[Row], draft: &str) -> Fired {
    match parsed(rows, draft) {
        None => Fired::Refused(format!("nothing here answers to {}", draft.trim())),
        Some((row, words)) => match row.wants {
            Some(wants) if words.is_empty() => {
                Fired::Refused(format!("{LEAD}{} needs {wants}", row.word))
            }
            _ => Fired::Fire(row.does, words),
        },
    }
}

#[cfg(test)]
mod tests;
