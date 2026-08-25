//! Every variant's user-visible spelling, pinned. These strings are shared
//! with the desktop seat, so a drift here is a drift between two clients
//! painting one record.

use super::{call, delivered, ended, go, model, prefixes, raw, result, streaming, text, thought};
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
    assert_eq!(prefixes(&both), ["thinking:", "live:"]);
    assert_eq!(both[0].class, RowClass::Other);
    assert_eq!(both[0].role, None);
    assert_eq!(both[1].class, RowClass::Response);
    assert_eq!(both[1].role, Some(Role::Model));
    assert!(both.iter().all(|row| row.tone == Tone::Live));

    assert_eq!(
        prefixes(&go(&[streaming("001", "mulling", "")])),
        ["thinking:"]
    );
    assert_eq!(prefixes(&go(&[streaming("001", "", "words")])), ["live:"]);
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
