use super::{Edges, Px};

/// The sighting's own geometry (bl-f557): a tall phone, 1280 by 2856 device
/// pixels at three to the point, with three-button navigation.
const PPP: f32 = 3.0;

fn same(given: f32, want: f32) {
    assert!((given - want).abs() < 0.01, "{given} vs {want}");
}

/// A control `wide` points across whose right edge stands `margin` points in
/// from the right of the rectangle it is laid out in — the composer's `send`
/// and the tab strip's `+`, both of which are the last thing on their row.
fn at_the_right(rect: [f32; 4], wide: f32, margin: f32) -> (f32, f32) {
    (rect[2] - margin - wide, rect[2] - margin)
}

/// **Held flat, the navigation bar is a strip down the right-hand side, and
/// nothing tappable is under it.** This is the defect: the probe carried
/// `top` and `bottom` only, so the strip was never subtracted and the two
/// right-anchored controls were painted beneath it, where a tap is the
/// platform's.
#[test]
fn a_bar_on_the_right_keeps_the_right_anchored_controls_clear_of_it() {
    let (wide, tall) = (2856.0 / PPP, 1280.0 / PPP);
    let bars = Px {
        left: 96,
        top: 0,
        right: 120,
        bottom: 0,
    };
    let strip = wide - 120.0 / PPP;
    let rect = bars.points(PPP).inside(wide, tall);
    same(rect[0], 32.0);
    same(rect[1], 0.0);
    same(rect[2], strip);
    same(rect[3], tall);
    // `send` and `+`: a touch target wide, the shell's own gutter in.
    let (left, right) = at_the_right(rect, 44.0, 10.0);
    assert!(left >= rect[0] && right <= strip, "{left}..{right}");
    // And the same control laid out in the WINDOW, which is what the shell
    // did: its right half is under the strip.
    let (_, was) = at_the_right([0.0, 0.0, wide, tall], 44.0, 10.0);
    assert!(was > strip, "{was} was already clear of {strip}");
}

/// The same phone upright, keyboard down and keyboard up: the top and the
/// bottom are spent by the same rule, with no path of their own.
#[test]
fn upright_the_same_rule_spends_the_status_bar_and_the_floor() {
    let (wide, tall) = (1280.0 / PPP, 2856.0 / PPP);
    let down = Px {
        left: 0,
        top: 150,
        right: 0,
        bottom: 144,
    };
    let rect = down.points(PPP).inside(wide, tall);
    same(rect[0], 0.0);
    same(rect[1], 50.0);
    same(rect[2], wide);
    same(rect[3], tall - 48.0);
    let up = Px {
        bottom: 1100,
        ..down
    };
    same(up.points(PPP).inside(wide, tall)[3], tall - 1100.0 / PPP);
}

/// A probe that answered nonsense is not a reason to lay out past the glass,
/// and a scale that is not a positive number covers nothing at all.
#[test]
fn nonsense_from_the_probe_covers_nothing() {
    let odd = Px {
        left: -30,
        top: 90,
        right: -1,
        bottom: -300,
    };
    assert_eq!(
        odd.points(PPP),
        Edges {
            left: 0.0,
            top: 30.0,
            right: 0.0,
            bottom: 0.0
        }
    );
    assert_eq!(odd.points(0.0), Edges::default());
    assert_eq!(odd.points(f32::NAN), Edges::default());
    assert_eq!(Px::default().points(PPP), Edges::default());
}

/// Insets that between them cover the whole window leave an empty rectangle
/// rather than an inverted one — a keyboard taller than a landscape window
/// less its bars is a real geometry, not a hypothetical.
#[test]
fn edges_that_cover_the_window_meet_rather_than_cross() {
    let all = Edges {
        left: 300.0,
        top: 200.0,
        right: 300.0,
        bottom: 400.0,
    };
    let rect = all.inside(400.0, 300.0);
    assert!(rect[0] <= rect[2] && rect[1] <= rect[3], "{rect:?}");
    same(rect[0], 300.0);
    same(rect[2], 300.0);
    same(rect[1], 200.0);
    same(rect[3], 200.0);
    let past = Edges {
        left: 900.0,
        top: 900.0,
        ..Edges::default()
    };
    for (given, want) in past
        .inside(400.0, 300.0)
        .into_iter()
        .zip([400.0, 300.0, 400.0, 300.0])
    {
        same(given, want);
    }
}
