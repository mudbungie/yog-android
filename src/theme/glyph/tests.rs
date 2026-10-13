use super::*;

#[test]
fn the_spellings_are_the_desktops() {
    assert_eq!(ALL, ["›", "—", "|", "¦", "·", "•", "✔", "✖", "−", "+", "…"]);
}

#[test]
fn no_glyph_is_one_this_tree_measured_as_tofu() {
    for glyph in ALL {
        assert!(!TOFU.contains(&glyph), "{glyph}");
        assert_eq!(glyph.chars().count(), 1, "{glyph}");
    }
}
