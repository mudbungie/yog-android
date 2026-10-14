use super::*;

const ALL: [Overlay; 16] = [
    Overlay::Queue,
    Overlay::Trail,
    Overlay::Candidates,
    Overlay::Fleet,
    Overlay::Clients,
    Overlay::Work,
    Overlay::Admin,
    Overlay::SignIn,
    Overlay::Help,
    Overlay::Balls(View::Everywhere),
    Overlay::Balls(View::Here),
    Overlay::Balls(View::Board),
    Overlay::Records,
    Overlay::Files,
    Overlay::Search,
    Overlay::Switcher,
];

fn under() -> Under {
    Under {
        workspace: "main".into(),
        conversation: "fixer".into(),
        needle: "flake".into(),
    }
}

#[test]
fn every_overlay_has_its_own_name() {
    let names: std::collections::BTreeSet<_> = ALL.iter().map(|o| o.name()).collect();
    assert_eq!(names.len(), ALL.len());
    assert_eq!(Overlay::Queue.name(), "waiting");
    assert_eq!(Overlay::Balls(View::Here).name(), "workspace-balls");
}

#[test]
fn the_close_line_reaches_the_read_under_it() {
    assert_eq!(Overlay::Trail.reaches(), "workspaces");
    assert_eq!(Overlay::Search.reaches(), "workspaces");
    assert_eq!(Overlay::Balls(View::Here).reaches(), "workspaces");
    assert_eq!(Overlay::Fleet.reaches(), "conversations");
    assert_eq!(Overlay::Records.reaches(), "transcript");
    assert_eq!(Overlay::Files.reaches(), "transcript");
}

#[test]
fn the_subject_is_what_the_overlay_is_open_on() {
    let at = under();
    let subject = |o: Overlay| o.line(false, &at).subject;
    for wide in [
        Overlay::Queue,
        Overlay::Trail,
        Overlay::Help,
        Overlay::Balls(View::Board),
    ] {
        assert_eq!(subject(wide), "", "{wide:?}");
    }
    assert_eq!(subject(Overlay::Balls(View::Everywhere)), "all workspaces");
    assert_eq!(subject(Overlay::Balls(View::Here)), "main");
    assert_eq!(subject(Overlay::Admin), "main");
    assert_eq!(subject(Overlay::Records), "fixer");
    assert_eq!(subject(Overlay::Files), "fixer");
    assert_eq!(subject(Overlay::Search), "flake");
}

#[test]
fn closing_leaves_except_where_something_else_is_standing() {
    let at = under();
    for overlay in ALL {
        let line = overlay.line(false, &at);
        assert_eq!(line.name, overlay.name());
        assert_eq!(line.reaches, overlay.reaches());
        let want = if overlay == Overlay::Search {
            Close::Unsearch
        } else {
            Close::Leave
        };
        assert_eq!(line.close, want, "{overlay:?}");
        assert_eq!(line.back, overlay != Overlay::Switcher, "{overlay:?}");
    }
    let minted = Overlay::Admin.line(true, &at);
    assert_eq!(
        (minted.name, minted.reaches, minted.close),
        ("minted", "config", Close::Forget)
    );
    // Minted material covers only the surface that minted it.
    assert_eq!(Overlay::Fleet.line(true, &at).close, Close::Leave);
}

#[test]
fn a_conversation_depth_stands_only_over_its_conversation() {
    assert_eq!(
        standing(Some(Overlay::Records), Depth::Workspace, false),
        None
    );
    assert_eq!(
        standing(Some(Overlay::Files), Depth::Workspace, false),
        None
    );
    // Over nothing focused, the switcher is what stands instead.
    assert_eq!(
        standing(Some(Overlay::Files), Depth::Roster, false),
        Some(Overlay::Switcher)
    );
    assert_eq!(
        standing(Some(Overlay::Files), Depth::Conversation, false),
        Some(Overlay::Files)
    );
}

#[test]
fn a_world_surface_stands_at_every_depth_and_over_a_search() {
    for depth in [Depth::Roster, Depth::Workspace, Depth::Conversation] {
        assert_eq!(
            standing(Some(Overlay::Trail), depth, true),
            Some(Overlay::Trail)
        );
    }
}

#[test]
fn the_hits_stand_over_the_roster_while_an_answer_does() {
    assert_eq!(standing(None, Depth::Roster, true), Some(Overlay::Search));
    assert_eq!(standing(None, Depth::Workspace, false), None);
    assert_eq!(standing(None, Depth::Workspace, true), None);
    // A held records screen the focus has left gives way to the hits.
    assert_eq!(
        standing(Some(Overlay::Records), Depth::Roster, true),
        Some(Overlay::Search)
    );
    // Search is never held: holding it is nothing.
    assert_eq!(
        standing(Some(Overlay::Search), Depth::Workspace, false),
        None
    );
}

#[test]
fn nothing_focused_is_the_switchers_screen_and_closing_it_there_leaves() {
    assert_eq!(
        standing(None, Depth::Roster, false),
        Some(Overlay::Switcher)
    );
    // Held, it stands over the chat at every depth.
    for depth in [Depth::Workspace, Depth::Conversation] {
        assert_eq!(
            standing(Some(Overlay::Switcher), depth, false),
            Some(Overlay::Switcher)
        );
    }
    let roster = Overlay::Switcher.line(false, &Under::default());
    assert_eq!((roster.name, roster.close), ("conversations", Close::Exit));
    assert!(!roster.back);
    assert_eq!(Overlay::Switcher.line(false, &under()).close, Close::Leave);
}

#[test]
fn the_line_says_the_phones_words_not_the_op_token() {
    let at = under();
    let title = |o: Overlay| o.line(false, &at).title;
    assert_eq!(title(Overlay::Work), "diff");
    assert_eq!(title(Overlay::Balls(View::Here)), "balls");
    assert_eq!(title(Overlay::Balls(View::Everywhere)), "balls");
    assert_eq!(title(Overlay::Queue), "waiting");
    assert_eq!(Overlay::Admin.line(true, &at).title, MINTED);
    // The screen keeps its name for the probe.
    assert_eq!(Overlay::Work.line(false, &at).name, "work-diff");
    // No title is a wire op's token with a hyphen in it, and the line's
    // own word is not the ball act's.
    assert!(ALL.iter().all(|o| !title(*o).contains('-')));
    assert_ne!(DONE, "close");
}
