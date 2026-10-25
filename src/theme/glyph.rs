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
/// A tab's close (DESIGN §23 item 2), the desktop's spelling.
pub const CLOSE: &str = "×";
/// **A knob on the rule** (DESIGN §23 item 14): it opens upward, and the
/// mark stands inside its door. The desktop's glyph — the bundled icon face
/// carries U+23F6, where the proportional face has no `▾`, `▴` or `▼`.
pub const DROP: &str = "⏶";
/// **The drawer's door** at the chat's top-left (DESIGN §24 item 3): the
/// hamburger every phone app opens its drawer from. U+2630, which the bundled
/// icon face carries; `≡` (U+2261) is in the monospace face alone and the
/// proportional family paints it as tofu.
pub const MENU: &str = "☰";
/// **A conversation row's state** (DESIGN §24 item 2; lernie `convs::glyph`):
/// working, at rest, stopped — and the asking mark that stands before the
/// name in the attention accent.
pub const WORKING: &str = "▶";
pub const AT_REST: &str = "✔";
pub const STOPPED: &str = "■";
pub const ASKING: &str = "?";

/// Every glyph the table spells, for the tests that judge them as a set.
pub const ALL: [&str; 18] = [
    PROMPT, LEAD, SEP, MODEL, NOTE, CALL, DONE, FAILED, OPEN, SHUT, ELIDED, CLOSE, DROP, MENU,
    WORKING, AT_REST, STOPPED, ASKING,
];

/// **What this tree has measured as tofu** in the bundled proportional face
/// (`roster::ATTENTION_MARK`'s `●`, the rails of `rows/thread.rs`, the fold
/// mark of `theme/anatomy.rs`, and `≡`, read off the faces' own `cmap` for
/// the drawer's door). No glyph in the table may be one of them.
pub const TOFU: [&str; 5] = ["●", "└", "│", "▼", "≡"];

/// **What a server sentence carries that the bundled proportional face
/// paints as tofu, and what stands in for it** (bl-8353). Measured against
/// `epaint_default_fonts`' charsets: `⇄` (U+21C4) is in the monospace face
/// alone, while `↔` (U+2194) is in the emoji face the proportional family
/// falls back to — the face `✔` and `✖` already come from.
pub const SWAPPED: [(&str, &str); 1] = [("⇄", "↔")];

/// **The server's words, made paintable**: each [`SWAPPED`] glyph replaced,
/// everything else byte for byte. The corpus is never edited — the word is
/// the server's — so the substitution is made where the words are read for
/// the glass.
#[must_use]
pub fn paintable(words: &str) -> String {
    SWAPPED.iter().fold(words.to_owned(), |out, (tofu, stand)| {
        out.replace(tofu, stand)
    })
}

#[cfg(test)]
mod tests;
