use super::table::Offers;
use super::*;

fn words(rows: &[Row]) -> Vec<&'static str> {
    rows.iter().map(|row| row.word).collect()
}

fn running() -> Gates {
    Gates {
        workspace: true,
        conversation: Some(Offers {
            stoppable: true,
            stop_children: true,
            at_rest: false,
        }),
    }
}

fn find(rows: &[Row], word: &str) -> Row {
    rows.iter()
        .find(|row| row.word == word)
        .cloned()
        .unwrap_or_else(|| unreachable!("{word}"))
}

#[test]
fn the_door_puts_the_slash_on_and_takes_it_off() {
    assert_eq!(door(""), "/");
    assert_eq!(door("it is broken"), "/it is broken");
    assert_eq!(door("/it is broken"), "it is broken");
    assert!(slashing("/x") && !slashing("x /") && !slashing(""));
}

#[test]
fn with_nothing_focused_only_the_world_is_offered() {
    let rows = rows(&Gates::default());
    assert_eq!(
        words(&rows),
        [
            "conversations",
            "workspace",
            "attention",
            "trail",
            "balls all",
            "board",
            "help",
            "find",
            "settings"
        ]
    );
}

/// The drawer's chips are the sheet's own rows (§24 item 2): the two world
/// surfaces, the seven aimed ones — dark with nothing focused — and help.
#[test]
fn the_drawers_places_are_the_sheets_rows_in_its_order() {
    let places = places();
    let said: Vec<(&str, bool)> = places
        .iter()
        .map(|place| (place.row.word, place.aimed))
        .collect();
    assert_eq!(
        said,
        [
            ("attention", false),
            ("trail", false),
            ("balls", true),
            ("science", true),
            ("diff", true),
            ("fleet", true),
            ("clients", true),
            ("config", true),
            ("login", true),
            ("help", false),
        ]
    );
    let all = rows(&running());
    for place in &places {
        assert_eq!(find(&all, place.row.word), place.row, "{}", place.row.word);
        assert!(matches!(place.row.does, Does::Open(_)));
    }
    let first = rows(&Gates::default());
    assert_eq!(find(&first, "conversations").does, Does::Drawer);
    assert_eq!(find(&first, "conversations").ops, ["conversations"]);
    assert_eq!(
        find(&first, "workspace").does,
        Does::Knob(crate::status::Slot::Workspace)
    );
}

#[test]
fn a_running_conversation_offers_the_stops_and_not_nudge() {
    let rows = rows(&running());
    let words = words(&rows);
    for word in [
        "records",
        "files",
        "balls",
        "config",
        "stop",
        "stop all",
        "interrupt",
    ] {
        assert!(words.contains(&word), "{word}");
    }
    assert!(!words.contains(&"nudge"));
    assert_eq!(find(&rows, "settings").summary, "this device's own knobs");
    assert_eq!(find(&rows, "stop all").ops, ["stop"]);
    assert_eq!(find(&rows, "diff").ops, ["work-diff"]);
}

#[test]
fn a_resting_conversation_offers_nudge_and_not_the_stops() {
    let gates = Gates {
        conversation: Some(Offers {
            at_rest: true,
            ..Offers::default()
        }),
        ..running()
    };
    let words = words(&rows(&gates));
    assert!(words.contains(&"nudge"));
    assert!(!words.contains(&"stop") && !words.contains(&"stop all"));
}

#[test]
fn a_summary_is_the_op_tables_unless_the_row_has_its_own() {
    let rows = rows(&running());
    let table = crate::help::rows(crate::help::TABLE).unwrap_or_default();
    let stop = table
        .iter()
        .find(|row| row.verb == "stop")
        .map(|r| r.summary.clone());
    assert_eq!(Some(find(&rows, "stop").summary), stop);
    assert_eq!(
        find(&rows, "records").summary,
        "the conversation's own ledger"
    );
    assert!(find(&rows, "flag").line().starts_with("/flag  "));
}

#[test]
fn a_line_names_the_longest_word_it_begins_with() {
    let rows = rows(&running());
    let named = |draft: &str| parsed(&rows, draft).map(|(row, rest)| (row.word, rest));
    assert_eq!(named("/stop"), Some(("stop", String::new())));
    assert_eq!(named("/stop all"), Some(("stop all", String::new())));
    assert_eq!(
        named("/ flag  it drifted "),
        Some(("flag", "it drifted".into()))
    );
    assert_eq!(named("/stopp"), None);
    assert_eq!(named("stop"), None);
}

#[test]
fn the_sheet_narrows_by_the_first_word_and_shows_all_when_nothing_answers() {
    let rows = rows(&running());
    assert_eq!(words(&matching(&rows, "/st")), ["stop", "stop all"]);
    assert_eq!(words(&matching(&rows, "/stop")), ["stop", "stop all"]);
    assert_eq!(words(&matching(&rows, "/flag why")), ["flag"]);
    assert_eq!(matching(&rows, "/it drifted").len(), rows.len());
    assert_eq!(matching(&rows, "/").len(), rows.len());
}

#[test]
fn a_tap_fires_a_wordless_row_and_spends_the_line() {
    let rows = rows(&running());
    let fired = |word: &str, draft: &str| tap(&find(&rows, word), &rows, draft);
    assert_eq!(
        fired("records", "/rec"),
        Fired::Fire(Does::Open(Overlay::Records), String::new())
    );
    assert_eq!(
        fired("stop", "/"),
        Fired::Fire(Does::Stop { children: false }, String::new())
    );
}

#[test]
fn a_tap_on_a_verb_takes_the_remainder_as_its_words() {
    let rows = rows(&running());
    let fired = |word: &str, draft: &str| tap(&find(&rows, word), &rows, draft);
    // Typed first, then the door, then the row.
    assert_eq!(
        fired("flag", "/it drifted"),
        Fired::Fire(Does::Flag, "it drifted".into())
    );
    // A partial word, then the words.
    assert_eq!(
        fired("flag", "/fl it drifted"),
        Fired::Fire(Does::Flag, "it drifted".into())
    );
    // The line already names it.
    assert_eq!(
        fired("flag", "/flag it drifted"),
        Fired::Fire(Does::Flag, "it drifted".into())
    );
    // No words: complete the line and wait for them.
    assert_eq!(fired("flag", "/"), Fired::Complete("/flag ".into()));
    assert_eq!(fired("find", "/fi"), Fired::Complete("/find ".into()));
}

#[test]
fn send_fires_a_named_line_and_refuses_what_it_cannot() {
    let rows = rows(&running());
    assert_eq!(
        send(&rows, "/find flake"),
        Fired::Fire(Does::Find, "flake".into())
    );
    assert_eq!(
        send(&rows, "/trail"),
        Fired::Fire(Does::Open(Overlay::Trail), String::new())
    );
    assert_eq!(
        send(&rows, "/flag "),
        Fired::Refused("/flag needs a reason".into())
    );
    assert_eq!(
        send(&rows, "/zap"),
        Fired::Refused("nothing here answers to /zap".into())
    );
}
