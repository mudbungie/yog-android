use super::*;

#[test]
fn the_spellings_are_the_desktops() {
    assert_eq!(
        ALL,
        [
            "›", "—", "|", "¦", "·", "•", "✔", "✖", "−", "+", "…", "×", "⏶"
        ]
    );
}

#[test]
fn no_glyph_is_one_this_tree_measured_as_tofu() {
    for glyph in ALL {
        assert!(!TOFU.contains(&glyph), "{glyph}");
        assert_eq!(glyph.chars().count(), 1, "{glyph}");
    }
}

#[test]
fn a_server_sentence_is_made_paintable_and_nothing_else_moves() {
    assert_eq!(
        paintable("every ball⇄workspace binding fact"),
        "every ball↔workspace binding fact"
    );
    assert_eq!(paintable("plain words"), "plain words");
    for (tofu, stand) in SWAPPED {
        assert!(!ALL.contains(&tofu) && !TOFU.contains(&stand), "{tofu}");
    }
}
