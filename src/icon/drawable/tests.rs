//! The pin: the committed assets ARE what the walk emits, both directions.

use crate::test_support::scratch;

/// Where the two layers live, relative to this crate's root — one home for
/// the paths, spent by the pin and by its failure message.
const FOREGROUND: &str = "android/app/src/main/res/drawable/ic_launcher_foreground.xml";
const BACKGROUND: &str = "android/app/src/main/res/drawable/ic_launcher_background.xml";

/// Both layers, pinned byte for byte. The emitted text is written out first
/// and its path named in the failure, so regenerating is a copy rather than a
/// transcription: move a constant in `icon` and this test tells you which
/// file to replace with which bytes.
#[test]
fn the_committed_layers_are_what_the_walk_emits() {
    let out = scratch();
    for (name, emitted, committed) in [
        (
            FOREGROUND,
            super::foreground(),
            include_str!("../../../android/app/src/main/res/drawable/ic_launcher_foreground.xml"),
        ),
        (
            BACKGROUND,
            super::background(),
            include_str!("../../../android/app/src/main/res/drawable/ic_launcher_background.xml"),
        ),
    ] {
        let at = out.join(name.rsplit('/').next().unwrap_or(name));
        std::fs::write(&at, &emitted).unwrap();
        assert_eq!(
            emitted,
            committed,
            "the walk moved: copy {} over {name}",
            at.display()
        );
    }
}

/// The other direction: an emission that stopped emitting must not pass as a
/// match against a file that was emptied with it. The mark's own census is
/// `icon::tests`; what this asserts is that the DOCUMENT is a document — a
/// vector element carrying one path per shape the walk hands over.
#[test]
fn the_emission_is_a_vector_of_the_walks_own_shapes() {
    let fore = super::foreground();
    assert!(fore.starts_with("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n"));
    assert!(fore.trim_end().ends_with("</vector>"));
    assert_eq!(fore.matches("<path ").count(), crate::icon::mark().len());
    // The hues are the walk's, written the way Android reads them.
    assert!(fore.contains("android:strokeLineCap=\"round\""));
    assert!(super::background().contains("android:fillColor=\"#FF0A080F\""));
}
