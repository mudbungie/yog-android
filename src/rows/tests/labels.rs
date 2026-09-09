//! Every variant's user-visible spelling, pinned. These strings are shared
//! with the desktop seat, so a drift here is a drift between two clients
//! painting one record.

use super::{
    SPEAKER, call, delivered, ended, go, go_open, model, named, parked, prefixes, raw, result,
    streaming, text, thought, windowed,
};
use crate::codec::{Block, Entry, EntryKind};
use crate::rows::{Fold, Role, RowClass, Tone};

#[test]
fn a_delivered_message_wears_its_sender_and_the_operator_role() {
    let rows = go(&[delivered("001", "user", "go")]);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].prefix, "user:");
    assert_eq!(rows[0].preview, "go");
    assert_eq!(rows[0].body, "");
    assert_eq!(rows[0].hover, "");
    assert_eq!(rows[0].class, RowClass::Response);
    assert_eq!(rows[0].tone, Tone::Plain);
    assert_eq!(rows[0].role, Some(Role::User));
    assert_eq!(rows[0].fold, Fold::Payload);
}

/// **The header takes the name and the role still reads the id** (PROTOCOL 17,
/// yog bl-6661): a phone's row header is the whole width it has, and a message
/// a child sent used to be attributed by sixty characters of timestamped hex.
#[test]
fn a_sender_wearing_a_name_is_headed_by_it() {
    let rows = go(&[named(
        "001",
        "20260814T000000Z-ab12",
        "DulcetMongoose",
        "found it",
    )]);
    assert_eq!(rows[0].prefix, "DulcetMongoose:");
    assert_eq!(
        rows[0].role,
        Some(Role::Peer),
        "the role is the id's reading"
    );
}

#[test]
fn any_other_sender_is_a_peer() {
    let rows = go(&[delivered("001", "scout", "found it")]);
    assert_eq!(rows[0].prefix, "scout:");
    assert_eq!(rows[0].role, Some(Role::Peer));
}

#[test]
fn an_epitaph_takes_the_prefix_seat_and_the_ending_role() {
    for token in [
        "final-response",
        "stopped",
        "budget-exhausted",
        "died",
        "invented-by-a-later-server",
    ] {
        let rows = go(&[ended("001", "scout", token, "done")]);
        assert_eq!(rows[0].prefix, format!("scout ended: {token}"));
        assert_eq!(rows[0].role, Some(Role::Ended));
        assert_eq!(rows[0].class, RowClass::Response);
    }
}

#[test]
fn an_empty_delivered_body_says_so_and_fades() {
    let rows = go(&[ended("001", "scout", "died", "")]);
    assert_eq!(rows[0].prefix, "scout ended: died");
    assert_eq!(rows[0].preview, "(no message body)");
    assert_eq!(rows[0].tone, Tone::Weak);
}

#[test]
fn a_model_text_block_speaks_as_the_agent_over_the_model_id() {
    let rows = go(&[model("001", vec![text("here you go")])]);
    assert_eq!(rows[0].prefix, "yog:");
    assert_eq!(rows[0].preview, "here you go");
    assert_eq!(
        rows[0].hover,
        "ran on sonnet-9 — the model is config, not the speaker"
    );
    assert_eq!(rows[0].class, RowClass::Response);
    assert_eq!(rows[0].role, Some(Role::Model));
}

#[test]
fn a_turn_with_no_content_blocks_is_machinery_that_says_so() {
    let rows = go(&[model("001", vec![])]);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].prefix, "yog:");
    assert_eq!(rows[0].preview, "(no content blocks)");
    assert_eq!(rows[0].class, RowClass::Other);
    assert_eq!(rows[0].tone, Tone::Weak);
    assert_eq!(rows[0].role, Some(Role::Model));
    assert_eq!(
        rows[0].hover,
        "ran on sonnet-9 — the model is config, not the speaker"
    );
}

#[test]
fn thinking_is_machinery_with_nobody_speaking() {
    let rows = go(&[model("001", vec![thought("weighing it")])]);
    assert_eq!(rows[0].prefix, "thinking:");
    assert_eq!(rows[0].preview, "weighing it");
    assert_eq!(rows[0].hover, "");
    assert_eq!(rows[0].class, RowClass::Other);
    assert_eq!(rows[0].tone, Tone::Weak);
    assert_eq!(rows[0].role, None);
}

#[test]
fn an_unretired_tool_call_says_running_beside_the_pulse() {
    let rows = go(&[model("001", vec![call("t1", "Read", "{\"path\":\"x\"}")])]);
    assert_eq!(rows[0].prefix, "⚙ Read — running");
    assert_eq!(rows[0].preview, "{\"path\":\"x\"}");
    assert_eq!(rows[0].tone, Tone::InFlight);
    assert_eq!(rows[0].role, None);
}

#[test]
fn a_retired_tool_call_drops_the_word_and_the_pulse() {
    let entries = [
        model("001", vec![call("t1", "Read", "{}")]),
        result("002", "t1", "ok", false),
    ];
    let rows = go(&entries);
    assert_eq!(rows[0].prefix, "⚙ Read");
    assert_eq!(rows[0].tone, Tone::Plain);
}

#[test]
fn only_a_matching_id_retires_a_call() {
    let entries = [
        model("001", vec![call("t1", "Read", "{}")]),
        result("002", "t2", "ok", false),
    ];
    assert_eq!(go(&entries)[0].prefix, "⚙ Read — running");
}

#[test]
fn a_tool_result_states_its_outcome_in_words() {
    let rows = go(&[result("001", "t1", "fine", false)]);
    assert_eq!(rows[0].prefix, "✔ tool result — ok");
    assert_eq!(rows[0].tone, Tone::Good);
    assert_eq!(rows[0].class, RowClass::Other);
    assert_eq!(rows[0].role, None);

    let rows = go(&[result("001", "t1", "boom", true)]);
    assert_eq!(rows[0].prefix, "✖ tool result — error");
    assert_eq!(rows[0].tone, Tone::Bad);
}

#[test]
fn the_live_tail_is_two_rows_and_an_empty_half_is_none() {
    let both = go(&[streaming("001", "mulling", "the answer so far")]);
    // The growing text is the SPEAKER's row, wearing the label its settled
    // counterpart will wear (bl-e3d1) — never a second speaker called `live`.
    assert_eq!(prefixes(&both), ["thinking:", &format!("{SPEAKER}:")]);
    assert_eq!(both[0].class, RowClass::Other);
    assert_eq!(both[0].role, None);
    assert_eq!(both[1].class, RowClass::Response);
    assert_eq!(both[1].role, Some(Role::Model));
    assert!(both.iter().all(|row| row.tone == Tone::Live));

    assert_eq!(
        prefixes(&go(&[streaming("001", "mulling", "")])),
        ["thinking:"]
    );
    assert_eq!(
        prefixes(&go(&[streaming("001", "", "words")])),
        [format!("{SPEAKER}:")]
    );
    assert!(go(&[streaming("001", "", "")]).is_empty());
}

#[test]
fn an_unparseable_entry_surfaces_under_its_own_filename() {
    let rows = go(&[raw("013-mystery.json", "{ not json")]);
    assert_eq!(rows[0].prefix, "013-mystery.json");
    assert_eq!(rows[0].preview, "{ not json");
    assert_eq!(rows[0].class, RowClass::Other);
    assert_eq!(rows[0].tone, Tone::Weak);
    assert_eq!(rows[0].role, None);
}

/// **The follow window wears the committed block's own words** (REMOTE §5.5):
/// the same `⚙ <tool> — running` a tool-use block wears while nothing has
/// retired it, so one call reads the same on either side of the commit.
#[test]
fn a_windowed_call_in_flight_reads_as_the_committed_block_does() {
    let rows = go(&[windowed("box2_Bash", "{\"command\":\"uptime\"}", None)]);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].prefix, "⚙ box2_Bash — running");
    assert_eq!(rows[0].preview, "{\"command\":\"uptime\"}");
    assert_eq!(rows[0].class, RowClass::Other);
    assert_eq!(rows[0].tone, Tone::InFlight);
    assert_eq!(rows[0].role, None);
    assert!(rows[0].expanded, "a step happening is the show");
}

/// **A parked call wears the one hue that means *asking for you***
/// (PROTOCOL 18, yog bl-58bb; STYLE.md §3). It is the only tone this
/// projection asks for that no wire token spells — a row tone says what a
/// conversation is doing, and a held call is a thing inside one.
///
/// The control's sentence is the row's PAYLOAD and goes first, because it is
/// what decides an answer; the command it is about opens under the fold, one
/// tap away.
#[test]
fn a_parked_call_asks_for_the_operator_and_says_why() {
    let rows = go(&[parked(
        "box2_Bash",
        "{\"command\":\"rm -rf build\"}",
        None,
        Some("classified loss — the control holds anything that destroys work"),
    )]);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].prefix, "⚙ box2_Bash — held for you");
    assert_eq!(
        rows[0].preview,
        "classified loss — the control holds anything that destroys work"
    );
    assert!(
        rows[0].body.contains("rm -rf build"),
        "the command opens under the fold"
    );
    assert_eq!(rows[0].tone, Tone::Held);
    assert!(rows[0].expanded, "a row asking for you is the show");
}

/// A hold with nothing to say about the input still says the control's
/// sentence, and shows no fold: an empty body IS the fact.
#[test]
fn a_parked_call_with_no_input_is_the_sentence_alone() {
    let rows = go(&[parked("box2_ping", "", None, Some("classified opaque"))]);
    assert_eq!(rows[0].preview, "classified opaque");
    assert_eq!(rows[0].body, "");
}

/// **A hold outranks a capture that landed after it.** The status is the
/// field's presence, the discipline `exit_code` already carries, and the two
/// cannot disagree because the hold is read first.
#[test]
fn a_hold_outranks_an_exit_code_on_the_same_row() {
    let rows = go(&[parked("box2_Bash", "{}", Some(0), Some("classified loss"))]);
    assert_eq!(rows[0].prefix, "⚙ box2_Bash — held for you");
    assert_eq!(rows[0].tone, Tone::Held);
}

/// **A closed call states the number and claims nothing about it.** REMOTE
/// §5.5 puts no verdict on this lane — `exit_code`'s PRESENCE is the status —
/// so the row goes plain rather than green or red, which is the reading
/// `codec::trail` refuses to invent one noun along.
#[test]
fn a_windowed_call_that_closed_states_the_number_and_no_verdict() {
    for code in [0, 1, 127] {
        let rows = go(&[windowed("box2_Bash", "{}", Some(code))]);
        assert_eq!(rows[0].prefix, format!("⚙ box2_Bash — exit {code}"));
        assert_eq!(rows[0].tone, Tone::Plain, "no hue reads the number");
    }
}

/// **An entry kind, and a block kind, this build has not heard of still paint**
/// (REMOTE §3.2). The entry says *unknown entry: `<word>`* over its own bytes —
/// `raw` rides beside the kind, so the transcript shows what the entry SAYS
/// even where it cannot say what it MEANS — and a stray block says its word in
/// resting ink. Refusing either would blank the whole transcript for one word
/// an engine added.
#[test]
fn a_kind_this_build_has_not_heard_of_paints_as_unknown() {
    let stray = Entry {
        name: "013-mystery.json".to_owned(),
        raw: "{ some newer shape".to_owned(),
        kind: EntryKind::Unknown("recital".to_owned()),
    };
    let rows = go(&[stray]);
    assert_eq!(rows[0].prefix, "unknown entry: recital");
    assert_eq!(rows[0].preview, "{ some newer shape");
    assert_eq!(rows[0].tone, Tone::Weak);

    let rows = go_open(&[model("001", vec![Block::Unknown("chorus".to_owned())])]);
    let said: Vec<&str> = rows.iter().map(|r| r.prefix.as_str()).collect();
    assert!(said.contains(&"unknown block: chorus"), "{said:?}");
}
