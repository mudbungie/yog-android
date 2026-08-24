//! `GameTextInput`'s selection spans index the Java `Editable`, so they count
//! UTF-16 code units; egui counts `char`s. The two agree on ASCII and
//! diverge at the first emoji — exactly the case a chat composer meets
//! (bl-014e). Pure math, host-tested; the android-only bridge is the caller.

/// The `char` offset in `text` for a UTF-16 code-unit offset, clamped to the
/// end of `text`. An offset landing inside a surrogate pair rounds up to the
/// next character boundary (an IME never produces one; the fence only has to
/// be safe, not split the atom).
pub fn char_index(text: &str, utf16: usize) -> usize {
    let mut units = 0;
    for (chars, c) in text.chars().enumerate() {
        if units >= utf16 {
            return chars;
        }
        units += c.len_utf16();
    }
    text.chars().count()
}

/// The UTF-16 code-unit offset for a `char` offset, clamped to the end of
/// `text` — the other direction of the same fence.
pub fn utf16_index(text: &str, chars: usize) -> usize {
    text.chars().take(chars).map(char::len_utf16).sum()
}

#[cfg(test)]
mod tests;
