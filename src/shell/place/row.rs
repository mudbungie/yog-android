//! **How one row of controls divides the width it has** (DESIGN §13.2,
//! bl-0691): every control says how wide it would like to be, and the row
//! answers with what each one gets.
//!
//! **One row, not two.** The controls under the composer used to be laid out
//! in two allocated bands, because three selectors and a toggle beside the
//! conversation acts left a model selector too narrow to read a model name in
//! (measured, bl-dfbb). The operator's ruling is that the second band cost
//! more than it bought — it is the transcript's own height, on the screen the
//! transcript is the point of — so the controls share one row and the two
//! selectors are what elides.
//!
//! **Two kinds of claim, and the second is honestly infinite.** A control
//! that names a verb claims exactly the width of its words; a selector that
//! shows whatever the workspace is set to claims *the remainder*, spelled
//! [`f32::INFINITY`] rather than as a sentinel. Where the row can pay every
//! finite claim it pays them ALL — the words that name an act are the ones an
//! operator cannot re-read by tapping — and the remainder is split evenly
//! among the selectors, which elide inside their share. That is the ruling's
//! own shape: *the two pickers elide their text*.
//!
//! **When even the finite claims do not fit, everyone falls to a max-min
//! fair share**: each control gets the LESSER of what it asked for and an
//! equal share of what is left once the smaller claims are met, so a narrow
//! display truncates the longest label first and never starves a short one to
//! feed it. One fallback, no per-control policy, and nothing in the row has
//! to know which kind of chip it is.
//!
//! Pure and host-tested, for `place`'s own reason: the paint stack is
//! `cfg(target_os = "android")` and no test in this suite can reach a line of
//! it, so a width rule that lived at the call site could not be asserted at
//! all.

/// The width each control gets, in the order they were asked for.
///
/// `available` is the row's whole width, `gap` the space between two
/// neighbours, and `wants` what each control would take if nothing stopped
/// it. The answer never sums past what a row of this many controls has room
/// for, and no control is given more than it asked for.
pub fn widths(available: f32, gap: f32, wants: &[f32]) -> Vec<f32> {
    let Some(gaps) = wants.len().checked_sub(1) else {
        return Vec::new();
    };
    let room = (available - gap * gaps as f32).max(0.0);
    let claimed: f32 = wants.iter().filter(|want| want.is_finite()).sum();
    let greedy = wants.iter().filter(|want| !want.is_finite()).count();
    if greedy > 0 && claimed.max(0.0) <= room {
        // Every named width is paid, and what is left is the selectors'.
        let each = (room - claimed.max(0.0)) / greedy as f32;
        return wants
            .iter()
            .map(|want| {
                if want.is_finite() {
                    want.max(0.0)
                } else {
                    each
                }
            })
            .collect();
    }
    let cap = share(room, wants);
    wants.iter().map(|want| want.max(0.0).min(cap)).collect()
}

/// **The widest any one control may be**: the largest number `t` for which
/// giving every control `min(want, t)` still fits the row.
///
/// `room` is the width the controls have between them, the gaps already taken
/// off. Found by handing out the smallest wants first — each in turn is either
/// satisfied outright, leaving more for the rest, or is the first that cannot
/// be, at which point the equal share of what remains is the answer for it
/// and for every want above it. A row where everything fits has no cap at
/// all, which is [`f32::INFINITY`] and is what makes a selector's want a
/// number rather than a special case.
fn share(room: f32, wants: &[f32]) -> f32 {
    let mut left = room;
    let mut rest = wants.len() as f32;
    let mut asked: Vec<f32> = wants.iter().map(|want| want.max(0.0)).collect();
    asked.sort_by(f32::total_cmp);
    for want in asked {
        let each = left / rest;
        if want > each {
            return each;
        }
        left -= want;
        rest -= 1.0;
    }
    f32::INFINITY
}

#[cfg(test)]
mod tests {
    use super::widths;

    /// The row this rule was written for: a phone's content width, the `S`
    /// gap, and six controls — four that want their own words and two
    /// selectors that want everything.
    const ROW: f32 = 373.0;
    const GAP: f32 = 8.0;

    /// What a row of `widths` actually occupies, gaps included — the number
    /// that must never exceed the row.
    fn spent(widths: &[f32], gap: f32) -> f32 {
        let gaps = gap * (widths.len().saturating_sub(1)) as f32;
        widths.iter().sum::<f32>() + gaps
    }

    /// The widths, to the tenth of a point: these are points of glass, and
    /// the arithmetic that produced them divides and re-adds the same
    /// numbers, which f32 does not promise exactly (`place`'s own SLACK, one
    /// module over).
    fn same(given: &[f32], want: &[f32]) {
        assert_eq!(given.len(), want.len(), "{given:?} vs {want:?}");
        for (given, want) in given.iter().zip(want) {
            assert!((given - want).abs() < 0.1, "{given} vs {want}");
        }
    }

    #[test]
    fn no_controls_take_no_width() {
        assert!(widths(ROW, GAP, &[]).is_empty());
    }

    #[test]
    fn controls_that_fit_get_exactly_what_they_asked_for() {
        let given = widths(ROW, GAP, &[40.0, 60.0, 50.0]);
        same(&given, &[40.0, 60.0, 50.0]);
        assert!(spent(&given, GAP) <= ROW);
    }

    #[test]
    fn one_greedy_control_takes_what_the_others_left() {
        let given = widths(ROW, GAP, &[40.0, 60.0, f32::INFINITY]);
        same(&given, &[40.0, 60.0, ROW - 40.0 - 60.0 - GAP * 2.0]);
    }

    #[test]
    fn two_greedy_controls_split_the_remainder_evenly() {
        let given = widths(ROW, GAP, &[40.0, f32::INFINITY, f32::INFINITY]);
        let each = (ROW - 40.0 - GAP * 2.0) / 2.0;
        same(&given, &[40.0, each, each]);
        assert!(spent(&given, GAP) <= ROW);
    }

    /// **A long label is truncated before a short one is** — the whole point
    /// of the fair share: the two words that fit keep their width, and the
    /// one that cannot be satisfied gets the share that is left.
    #[test]
    fn a_want_too_big_for_its_share_is_capped_and_the_small_ones_are_not() {
        let given = widths(200.0, GAP, &[30.0, 30.0, 400.0]);
        same(&given, &[30.0, 30.0, 200.0 - 60.0 - GAP * 2.0]);
    }

    /// Every control asking for more than the row has: equal shares, and the
    /// row is still not overspent.
    #[test]
    fn controls_that_all_want_everything_share_equally() {
        let given = widths(ROW, GAP, &[f32::INFINITY; 6]);
        let each = (ROW - GAP * 5.0) / 6.0;
        same(&given, &[each; 6]);
        assert!(spent(&given, GAP) <= ROW);
    }

    /// A row with no room at all — the keyboard-up extreme, or a control band
    /// on a display narrower than its own gaps. Nothing is negative, and
    /// nothing is painted past the edge.
    #[test]
    fn a_row_with_no_room_gives_every_control_nothing() {
        let given = widths(4.0, GAP, &[40.0, f32::INFINITY, 20.0]);
        same(&given, &[0.0, 0.0, 0.0]);
    }

    /// **The row this rule exists for, measured on the device** (bl-0691):
    /// five controls at a Pixel 6's content width — the nudge act, the two
    /// tuning faces wearing their values, and the two selectors. Every named
    /// width is paid in full and the selectors take what is left, which is
    /// the ruling: the pickers are what elides. Under a plain max-min share
    /// the two faces would be capped at the selectors' own share and
    /// `effort: off` would read `effor…`, hiding the value the face exists
    /// to carry.
    #[test]
    fn the_phones_own_row_pays_every_named_width_and_the_selectors_take_the_rest() {
        let (nudge, effort, priority) = (49.0, 91.0, 78.0);
        let given = widths(
            391.0,
            GAP,
            &[nudge, effort, priority, f32::INFINITY, f32::INFINITY],
        );
        let each = (391.0 - GAP * 4.0 - nudge - effort - priority) / 2.0;
        same(&given, &[nudge, effort, priority, each, each]);
        assert!(each > 60.0, "a selector still reads as a name: {each}");
        assert!(spent(&given, GAP) <= 391.0);
    }

    /// A control that asks for a negative width is asking for nothing, and
    /// the room it did not take is the others'.
    #[test]
    fn a_negative_want_is_nothing_and_frees_the_room_it_did_not_take() {
        let given = widths(100.0, 0.0, &[-10.0, f32::INFINITY]);
        same(&given, &[0.0, 100.0]);
    }
}
