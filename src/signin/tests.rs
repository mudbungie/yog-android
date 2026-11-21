//! The reading of a sign-in's tail lines, held to the lines `bz --login`
//! actually prints: a device flow's address and code, a loopback flow's
//! plaintext address, the polling chatter and a failure's remedy. The
//! fixture is invented in the reserved documentation space — no transcript is
//! quoted and no real host is named.

use super::{
    CODE, COPIES, Machine, OPENS, PAGE, PAGE_OPENS, PASTE, Part, Read, SEND, SENDS, copy_word,
    doors, fact, is_code, page, pasted, starts,
};

fn prose(words: &str) -> Part {
    Part::Prose(words.to_owned())
}

fn url(address: &str) -> Part {
    Part::Url(address.to_owned())
}

/// What each kind of line a run prints earns: `(line, urls, code)`.
const FLOW: &[(&str, &[&str], Option<&str>)] = &[
    ("signing in acme for workspace home", &[], None),
    ("open this address in a browser:", &[], None),
    (
        "  https://example.com/device",
        &["https://example.com/device"],
        None,
    ),
    (
        "and enter the code WDJB-MJHT when it asks.",
        &[],
        Some("WDJB-MJHT"),
    ),
    (
        "or visit https://example.com/activate?user_code=WDJB-MJHT directly.",
        &["https://example.com/activate?user_code=WDJB-MJHT"],
        None,
    ),
    (
        "waiting for the provider to confirm (polling every 5s)...",
        &[],
        None,
    ),
    (
        "could not reach https://example.com/token: connection refused",
        &["https://example.com/token"],
        None,
    ),
    (
        "the browser flow listens at http://localhost:8765/callback",
        &[],
        None,
    ),
    (
        "error: the device code expired; run `bz --login acme` again",
        &[],
        None,
    ),
];

#[test]
fn every_kind_of_line_a_run_prints_earns_its_doors() {
    for (line, urls, code) in FLOW {
        let read = doors(line);
        let found: Vec<_> = read
            .parts
            .iter()
            .filter_map(|part| match part {
                Part::Url(address) => Some(address.as_str()),
                Part::Prose(_) => None,
            })
            .collect();
        assert_eq!(found, *urls, "{line}");
        assert_eq!(read.code.as_deref(), *code, "{line}");
        assert_eq!(read.plain(), urls.is_empty() && code.is_none(), "{line}");
    }
}

#[test]
fn a_line_with_no_door_is_one_run_of_prose_as_it_stood() {
    let line = "waiting for the provider to confirm (polling every 5s)...";
    assert_eq!(
        doors(line),
        Read {
            parts: vec![prose(line)],
            code: None,
        }
    );
}

#[test]
fn the_address_is_cut_from_the_punctuation_leaning_on_it() {
    assert_eq!(
        doors("see (https://example.com/device). then wait"),
        Read {
            parts: vec![
                prose("see ("),
                url("https://example.com/device"),
                prose("). then wait"),
            ],
            code: None,
        }
    );
    assert_eq!(
        doors("`https://example.com/a`"),
        Read {
            parts: vec![prose("`"), url("https://example.com/a"), prose("`")],
            code: None,
        }
    );
}

#[test]
fn the_code_stays_in_the_sentence_and_the_first_one_is_the_door() {
    let read = doors("enter ABCD-1234 (not EFGH-5678):");
    assert_eq!(read.parts, vec![prose("enter ABCD-1234 (not EFGH-5678):")]);
    assert_eq!(read.code.as_deref(), Some("ABCD-1234"));
    assert!(!read.plain());
}

#[test]
fn a_line_may_carry_both_doors() {
    let read = doors("go to https://example.com/device and type ABCD-1234");
    assert_eq!(
        read.parts,
        vec![
            prose("go to"),
            url("https://example.com/device"),
            prose("and type ABCD-1234"),
        ]
    );
    assert_eq!(read.code.as_deref(), Some("ABCD-1234"));
}

#[test]
fn a_bare_scheme_and_a_plaintext_address_are_prose() {
    assert!(doors("https://").plain());
    assert!(doors("http://example.com/device").plain());
    assert!(doors("").plain());
}

#[test]
fn the_template_is_the_shape_and_prose_is_not() {
    assert!(is_code(CODE));
    for code in ["WDJB-MJHT", "1234-5678", "A1B2-C3D4"] {
        assert!(is_code(code), "{code}");
    }
    for word in [
        "abcd-1234",
        "open-this",
        "sign-in",
        "ABCDE-123",
        "ABCD_1234",
        "ABCD-12345",
        "ABCD-123",
        "ABCD 1234",
    ] {
        assert!(!is_code(word), "{word}");
    }
}

#[test]
fn the_words_on_the_doors_and_under_the_row() {
    assert_eq!(copy_word("ABCD-1234"), "copy ABCD-1234");
    assert_eq!(
        fact("home", &Machine::Server),
        "signs in the server, for every seat of home"
    );
    assert!(OPENS.contains("browser") && OPENS.contains("long press"));
    assert!(COPIES.starts_with("copies"));
}

/// **A row on another machine says so, twice** (REMOTE §5.7): whose
/// credential lands, and which box the run starts on.
#[test]
fn a_row_on_another_machine_names_that_machine() {
    let foot = Machine::Client("foot".to_owned());
    assert_eq!(
        fact("home", &foot),
        "signs in foot, for the model calls home routes to it"
    );
    assert_eq!(
        starts(&foot),
        "starts this provider's sign-in on foot, and follows what it says"
    );
    assert_eq!(
        starts(&Machine::Server),
        "starts this provider's sign-in on the server, and follows what it says"
    );
}

/// **A waiting sign-in's address is a door under the one address rule**, and
/// what goes back is the paste whole: only the whitespace around it is
/// dropped, because a page may hand over `code#state` or a whole address.
#[test]
fn the_page_is_opened_over_tls_and_the_paste_goes_back_whole() {
    let url = "https://provider.invalid/authorize?state=s&challenge=c";
    assert_eq!(page(url).as_deref(), Some(url));
    assert_eq!(page("http://provider.invalid/authorize"), None);
    assert_eq!(page("https://"), None);
    assert_eq!(pasted("  abc#def \n").as_deref(), Some("abc#def"));
    let address = "http://localhost/callback?code=a b&state=s#f";
    assert_eq!(
        pasted(&format!("\t{address}\r\n")).as_deref(),
        Some(address)
    );
    assert_eq!(pasted(" \n\t"), None);
    assert!(PAGE_OPENS.contains("browser") && PAGE_OPENS.contains("long press"));
    assert!(PASTE.starts_with("paste") && SENDS.contains("exactly as"));
    assert_eq!((PAGE, SEND), ("open the sign-in page", "send"));
}
