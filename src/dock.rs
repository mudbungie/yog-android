//! **The live dock** (DESIGN §22 item 9, bl-365c): what is running under the
//! focused workspace, folded to one tap line above the rule and unfolded by a
//! tap. Pure and host-tested — which rows, in which order, what the line and
//! each row say; `shell/dock.rs` paints them and performs the taps. The
//! desktop's shape (lernie `ui::dock`), at a thumb's size.
//!
//! **The dock asks nothing of its own.** Running is the engine's word on the
//! conversation rows this seat already holds for the focused workspace —
//! `live` or `in-flight` (`AgentState`) — so the dock is a reading of those
//! rows, in the switcher's order (`crate::roster::ordered`).
//!
//! Folded, it is one line: `+ N running · <the first's name>`. Unfolded, one
//! touch-tall row per running conversation — its name, its state in its
//! tone's ink, how long since it acted — and a tap on a row focuses it.
//! Whether it is unfolded is viewport state: egui's temporary memory, which
//! dies with the process like a fold.

use crate::codec::{AgentState, ConvRow};
use crate::theme::{self, Rgb, glyph};

/// **The conversations running under the focused workspace**, in the
/// switcher's order.
pub fn running(conversations: &[ConvRow]) -> Vec<ConvRow> {
    crate::roster::ordered(conversations.to_vec())
        .into_iter()
        .filter(|row| matches!(row.state, AgentState::Live | AgentState::InFlight))
        .collect()
}

/// **The folded line's words**, or nothing when nothing runs — which is the
/// whole of when the dock stands.
pub fn line(rows: &[ConvRow], unfolded: bool) -> Option<String> {
    let first = rows.first()?;
    let fold = if unfolded { glyph::OPEN } else { glyph::SHUT };
    Some(format!("{fold} {} running · {}", rows.len(), first.display))
}

/// **One running conversation's row**: its name in ink, its state in its
/// tone's ink, how long since it acted in weak ink.
pub fn row_words(row: &ConvRow, now_unix: i64) -> Vec<(String, Rgb)> {
    vec![
        (row.display.clone(), theme::INK),
        (format!("  {}", row.state.word()), theme::tone(&row.tone)),
        (
            format!("  {}", crate::roster::stamp(row.last_active_unix, now_unix)),
            theme::INK_WEAK,
        ),
    ]
}

#[cfg(test)]
mod tests;
