//! **What one conversation row says** (DESIGN §13.20, bl-4d17), as the lines
//! a list paints — split from the order and the indent because it is the
//! third reading this module takes of one row, and the only one about words.

use crate::codec::ConvRow;

/// **The attention mark** (bl-f34b): the one glyph that says *something is
/// waiting on you*, painted on the roster's queue entry, the workspace rows
/// and the conversation rows — one constant, because a mark that differs by
/// screen is three marks. U+2022 BULLET, because the bundled proportional
/// face (egui's Ubuntu-Light) carries it; U+25CF BLACK CIRCLE, which stood
/// here before, is in none of the four bundled faces and painted as the
/// missing-glyph box on the one screen an operator lands on. Checked against
/// the face's own `cmap`, and by the walk's picture (`02-roster.png`).
///
/// It lives here rather than beside the screens that paint it (bl-4d17)
/// because [`lines`] spends it in the middle of a row's first line: one fact,
/// one home, and the home is the module that says what a row says.
pub const ATTENTION_MARK: &str = " \u{2022}";

/// **What a conversation row says, one line apiece** (§13.20, bl-4d17): who
/// and when, what it last said, and — where the engine reddened the row —
/// that its latest call failed.
///
/// **Every line is folded to one line here**, and that is the whole of what
/// this function adds over a format string. REMOTE §9.10 carries *the
/// provider's own first clause*, and a clause may be three sentences with
/// hard breaks in them: unfolded, one failing row took four lines of a phone
/// list and pushed the rest off the glass. The list is an index and the whole
/// text is on the conversation screen, one tap away.
///
/// A `Bad` tone with no clause still says nothing extra, which is the third
/// thing it is — a call that failed and left no words.
pub fn lines(row: &ConvRow, now_unix: i64) -> Vec<String> {
    let mark = if row.attention > 0 {
        ATTENTION_MARK
    } else {
        ""
    };
    let mut lines = vec![format!(
        "{}{mark} \u{b7} {}",
        row.display,
        super::stamp(row.last_active_unix, now_unix)
    )];
    let preview = one_line(&row.preview);
    if !preview.is_empty() {
        lines.push(preview);
    }
    if let Some(why) = &row.failure {
        lines.push(format!("failed \u{b7} {}", one_line(why)));
    }
    lines
}

/// Every run of whitespace — a newline included — as one space.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests;
