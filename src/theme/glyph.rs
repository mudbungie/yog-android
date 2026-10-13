//! **The glyph set** (DESIGN §22 item 12): the desktop's table (lernie
//! `ui::theme::glyph`), moved here so its spellings are host-tested and pinned
//! to the desktop's. Every substitution the desktop made against the bundled
//! egui faces is made here too: `❯`→`›`, `┊`→`¦`, `─`→`—`, `│`→`|`, `●`→`•`,
//! `✓`→`✔`, `✗`→`✖`, `▾ ▸`→`− +`; a stem is a painted stroke, never a glyph.
//!
//! The phone cannot run the desktop's `has_glyphs` assertion on the host (egui
//! is compiled for android only), so the proof is two-sided: the desktop's
//! test proves this table against the same `default_fonts` faces, and this
//! crate's test refuses every glyph this tree has already MEASURED as tofu.

/// **The prompt**: the glyph at the left edge of the line the operator types
/// into, in the brand — and the door to the verb sheet.
pub const PROMPT: &str = "›";
/// The rule's lead, at both of its ends.
pub const LEAD: &str = "—";
/// What parts the rule's facts.
pub const SEP: &str = "|";
/// The model's gutter.
pub const MODEL: &str = "¦";
/// A note's gutter: a peer, a tool's answer, the machinery's own word.
pub const NOTE: &str = "·";
/// A call on the trail.
pub const CALL: &str = "•";
/// A call that returned, and one that failed.
pub const DONE: &str = "✔";
pub const FAILED: &str = "✖";
/// A fold, open and shut.
pub const OPEN: &str = "−";
pub const SHUT: &str = "+";
/// Where a line was cut.
pub const ELIDED: &str = "…";

/// Every glyph the table spells, for the tests that judge them as a set.
pub const ALL: [&str; 11] = [
    PROMPT, LEAD, SEP, MODEL, NOTE, CALL, DONE, FAILED, OPEN, SHUT, ELIDED,
];

/// **What this tree has measured as tofu** in the bundled proportional face
/// (`roster::ATTENTION_MARK`'s `●`, the rails of `rows/thread.rs`, the fold
/// mark of `theme/anatomy.rs`). No glyph in the table may be one of them.
pub const TOFU: [&str; 4] = ["●", "└", "│", "▼"];

/// **What an engine sentence carries that the bundled proportional face
/// paints as tofu, and what stands in for it** (bl-8353). Measured against
/// `epaint_default_fonts`' charsets: `⇄` (U+21C4) is in the monospace face
/// alone, while `↔` (U+2194) is in the emoji face the proportional family
/// falls back to — the face `✔` and `✖` already come from.
pub const SWAPPED: [(&str, &str); 1] = [("⇄", "↔")];

/// **The engine's words, made paintable**: each [`SWAPPED`] glyph replaced,
/// everything else byte for byte. The corpus is never edited — the word is
/// the engine's — so the substitution is made where the words are read for
/// the glass.
#[must_use]
pub fn paintable(words: &str) -> String {
    SWAPPED.iter().fold(words.to_owned(), |out, (tofu, stand)| {
        out.replace(tofu, stand)
    })
}

#[cfg(test)]
mod tests;
