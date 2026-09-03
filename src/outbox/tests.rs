//! The one rule, and the weakness it is honest about.

use super::taken;
use crate::codec::{Entry, EntryKind};

fn delivered(body: &str) -> Entry {
    Entry {
        name: "001".to_owned(),
        raw: String::new(),
        kind: EntryKind::Delivered {
            sender: "op".to_owned(),
            epitaph: None,
            body: body.to_owned(),
        },
    }
}

fn other() -> Entry {
    Entry {
        name: "002".to_owned(),
        raw: String::new(),
        kind: EntryKind::Raw,
    }
}

#[test]
fn a_delivered_row_at_the_tail_takes_the_echo() {
    assert!(taken(&[delivered("hello")], "hello"));
    assert!(taken(&[other(), delivered("hello")], "hello"));
    // Whitespace is not the difference between a message and its echo: the
    // composer's text and the engine's row agree once trimmed.
    assert!(taken(&[delivered("hello\n")], "  hello "));
}

#[test]
fn nothing_matching_leaves_the_echo_standing() {
    assert!(!taken(&[], "hello"));
    assert!(!taken(&[other()], "hello"));
    assert!(!taken(&[delivered("goodbye")], "hello"));
    // A model turn saying the same words is not this message coming back.
    assert!(!taken(&[other(), other()], "hello"));
}

/// The tail is the window, and it is what keeps an identical message far up
/// the conversation from dissolving a fresh echo.
#[test]
fn an_identical_message_far_up_the_transcript_is_not_this_one() {
    let mut long = vec![delivered("ok")];
    long.extend((0..8).map(|_| other()));
    assert!(!taken(&long, "ok"));
    // …and once it is back inside the window, it is.
    let short = vec![delivered("ok"), other(), other()];
    assert!(taken(&short, "ok"));
}

/// **The echo's three fates and the two ways it goes** (bl-66fb, widened for
/// the lost-reply contract in bl-07b1). The counters are the only receipt this
/// end has, so each test moves exactly one of them and reads what the echo
/// became.
mod echo {
    use super::delivered;
    use crate::outbox::{Echo, Fate, Settled};
    use crate::seat::{Focus, Snapshot};

    fn focused() -> Snapshot {
        Snapshot {
            focus: Focus {
                workspace: Some("home".to_owned()),
                agent: Some("a1".to_owned()),
            },
            ..Snapshot::default()
        }
    }

    fn standing(echo: Echo, snap: &Snapshot) -> Echo {
        match echo.settle(snap) {
            Settled::Standing(echo) => echo,
            Settled::Gone => panic!("the echo dissolved"),
            Settled::Draft(text) => panic!("the echo went back to the composer: {text}"),
        }
    }

    #[test]
    fn a_sent_echo_stands_muted_until_a_counter_moves() {
        let snap = focused();
        let echo = Echo::sent("hello".to_owned(), &snap);
        assert_eq!(echo.fate, Fate::Sent);
        assert_eq!(standing(echo, &snap).fate, Fate::Sent);
    }

    #[test]
    fn a_receipt_inks_it_and_the_row_it_becomes_dissolves_it() {
        let snap = focused();
        let echo = Echo::sent("hello".to_owned(), &snap);
        let mut answered = focused();
        answered.landed = 1;
        let echo = standing(echo, &answered);
        assert_eq!(echo.fate, Fate::Landed);
        answered.transcript = vec![delivered("hello")];
        assert!(matches!(echo.settle(&answered), Settled::Gone));
    }

    /// The engine said no, so the text goes back to the field: saying it again
    /// is a first attempt rather than a repeat.
    #[test]
    fn a_refusal_hands_the_draft_back() {
        let snap = focused();
        let echo = Echo::sent("hello".to_owned(), &snap);
        let mut refused = focused();
        refused.refused = 1;
        match echo.settle(&refused) {
            Settled::Draft(text) => assert_eq!(text, "hello"),
            _ => panic!("a refusal must give the composer its text back"),
        }
    }

    /// **The defect this ball closes.** A lost reply is not a refusal: the
    /// engine may have taken the message, so the draft must NOT come back —
    /// one tap on a restored draft is the resend REMOTE §3 forbids. The echo
    /// stands, saying what it is.
    #[test]
    fn a_lost_reply_stands_in_doubt_and_never_becomes_a_draft() {
        let snap = focused();
        let echo = Echo::sent("hello".to_owned(), &snap);
        let mut doubted = focused();
        doubted.doubted = 1;
        let echo = standing(echo, &doubted);
        assert_eq!(echo.fate, Fate::InDoubt);
        // …and the read is the recovery: the transcript settles it with no
        // gesture at all, whichever way it went.
        doubted.transcript = vec![delivered("hello")];
        assert!(matches!(echo.settle(&doubted), Settled::Gone));
    }

    /// An echo is a message in ONE conversation, and leaving that conversation
    /// takes it — including one in doubt, whose read lives in the transcript
    /// the operator walked away from.
    #[test]
    fn leaving_the_conversation_takes_the_echo_with_it() {
        let snap = focused();
        let echo = Echo::sent("hello".to_owned(), &snap);
        assert!(matches!(echo.settle(&Snapshot::default()), Settled::Gone));
    }
}
