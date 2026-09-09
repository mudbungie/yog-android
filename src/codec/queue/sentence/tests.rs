//! The split, both directions: the engine's real sentence comes apart at the
//! engine's own clause, and a sentence with no clause in it is kept whole.

use super::folded;

/// The engine's own shape (`control::reason::reason`): the tool, a clip of the
/// call's JSON input, the class clause, and the evidence in brackets. This is
/// the sentence from the operator's 2026-09-08 screenshot, shortened only in
/// the evidence — the head is verbatim, because the head is what must not
/// reach the band.
const REAL: &str = "multi_tool {\"execution\":\"parallel\",\"invocations\":[{\"input\":{\"client\":\"NickelBuzz\",\
    \"op\":\"get\"}}]} classified opaque (multi_tool is not a tool this control implements and its \
    input carries no command line)";

/// **The line names what is held and the class, and carries no input.** The
/// whole complaint was a JSON dump under the message field; the line is the
/// one thing that stands there now, so the assertion is on both halves — what
/// it says, and what it cannot say.
#[test]
fn the_line_is_the_tool_and_the_class_and_never_the_input() {
    let out = folded("multi_tool", REAL);
    assert_eq!(out.line, "held: multi_tool · opaque");
    assert!(
        !out.line.contains("NickelBuzz") && !out.line.contains('{'),
        "the call's input never reaches the line"
    );
}

/// **The decision is the engine's own words, from its own clause on.** Split,
/// never rewritten (REMOTE §8.1) — so the tail is byte-equal to the tail of
/// the sentence that crossed.
#[test]
fn the_why_is_the_sentence_from_the_clause_on() {
    let out = folded("multi_tool", REAL);
    assert!(out.why.starts_with("classified opaque ("));
    assert!(REAL.ends_with(&out.why), "not one word is rewritten");
    assert!(
        !out.why.contains("NickelBuzz"),
        "the input is before the clause, so it is not in the decision either"
    );
}

/// **The LAST clause wins**, which is `control::reason::class_of`'s own rule:
/// an input summary may quote the word, and the sentence's own clause is the
/// one at the end.
#[test]
fn the_last_clause_is_the_one_that_counts() {
    let out = folded(
        "Bash",
        "Bash {\"command\":\"grep ' classified secret ' notes\"} classified read (a read of one file)",
    );
    assert_eq!(out.line, "held: Bash · read");
    assert!(out.why.starts_with("classified read ("));
}

/// **A sentence with no clause of ours is the decision entire**, and the line
/// says only what is held: the seat cannot tell a call from prose in a
/// sentence it does not recognise, and dropping the engine's words would be
/// worse than showing an input behind a fold.
#[test]
fn a_sentence_with_no_clause_is_kept_whole() {
    let out = folded("box2_ping", "  the control holds anything it cannot read  ");
    assert_eq!(out.line, "held: box2_ping");
    assert_eq!(out.why, "the control holds anything it cannot read");
}

/// A clause with no class word after it is not a clause: the split does not
/// happen, rather than producing a line ending in a separator with nothing
/// behind it.
#[test]
fn a_clause_with_nothing_after_it_does_not_split() {
    let out = folded("Bash", "Bash {} classified ");
    assert_eq!(out.line, "held: Bash");
    assert_eq!(out.why, "Bash {} classified");
}

/// A clause at the very head of the sentence — no tool and no input before it
/// — is not this clause either: it is spelled with both its spaces, so a
/// sentence that simply opens with the word keeps its own words.
#[test]
fn a_bare_leading_classified_is_not_the_clause() {
    let out = folded("Bash", "classified loss — anything that destroys work");
    assert_eq!(out.line, "held: Bash");
    assert_eq!(out.why, "classified loss — anything that destroys work");
}
