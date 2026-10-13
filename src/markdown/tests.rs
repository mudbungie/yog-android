//! The truth table: what each mark becomes, and that a mark which does not
//! close keeps every byte.

use super::*;

fn plain(text: &str) -> Span {
    Span {
        style: Style::Plain,
        text: text.into(),
    }
}

fn styled(style: Style, text: &str) -> Span {
    Span {
        style,
        text: text.into(),
    }
}

#[test]
fn a_fence_is_a_code_block_without_its_fences_and_prose_parts_it() {
    let said = blocks("before\n```rust\nlet x = 1;\n  y\n```\nafter");
    assert_eq!(
        said,
        [
            Block::Prose(vec![plain("before")]),
            Block::Code("let x = 1;\n  y".into()),
            Block::Prose(vec![plain("after")]),
        ]
    );
}

#[test]
fn a_fence_still_open_runs_to_the_end_and_empty_prose_is_no_block() {
    assert_eq!(
        blocks("```\nstreaming\n\n"),
        [Block::Code("streaming\n".into())]
    );
    assert_eq!(blocks("\n\n"), []);
    assert_eq!(blocks(""), []);
}

#[test]
fn a_heading_loses_its_marks_and_a_hash_that_is_not_one_stays() {
    assert_eq!(
        blocks("## The `fix`\nbody\n#hashtag\n####### seven"),
        [
            Block::Heading(vec![plain("The "), styled(Style::Code, "fix")]),
            Block::Prose(vec![plain("body\n#hashtag\n####### seven")]),
        ]
    );
    assert_eq!(
        blocks("```\n# a comment\n```"),
        [Block::Code("# a comment".into())]
    );
}

#[test]
fn inline_marks_become_styles_without_their_marks() {
    assert_eq!(
        spans("run `cargo test` **now**, *gently*"),
        [
            plain("run "),
            styled(Style::Code, "cargo test"),
            plain(" "),
            styled(Style::Strong, "now"),
            plain(", "),
            styled(Style::Italic, "gently"),
        ]
    );
    assert_eq!(
        spans("`a * b`"),
        [styled(Style::Code, "a * b")],
        "no emphasis inside code"
    );
}

#[test]
fn a_mark_that_does_not_close_or_hugs_a_space_keeps_every_byte() {
    for literal in [
        "2 * 3 * 4",
        "a stray ` tick",
        "**unclosed",
        "``",
        "* not a list item *",
        "snake_case_name",
        "é * ü",
    ] {
        assert_eq!(spans(literal), [plain(literal)], "{literal}");
    }
}
