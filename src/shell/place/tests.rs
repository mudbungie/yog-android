use super::{Band, Fit, fit, list};

/// A phone with a status bar and a gesture-nav bar, in points: the tappable
/// area `app::pass` hands every screen.
const AREA: Band = Band {
    top: 24.0,
    bottom: 820.0,
};

/// A control standing on the floor — the controls row, which is the last
/// thing the bottom-up layout adds and therefore the lowest thing on the
/// glass (`shell/controls.rs`). This is the operator's own sighting.
const ON_THE_FLOOR: Band = Band {
    top: 776.0,
    bottom: 820.0,
};

#[test]
fn a_list_that_fits_below_its_control_opens_below() {
    let anchor = Band {
        top: 200.0,
        bottom: 244.0,
    };
    let placed = fit(AREA, anchor, 4.0, 100.0).expect("room below");
    assert_eq!(
        placed,
        Fit {
            above: false,
            height: 100.0
        }
    );
    assert_eq!(
        list(AREA, anchor, 4.0, placed),
        Band {
            top: 248.0,
            bottom: 348.0
        }
    );
}

#[test]
fn a_control_on_the_floor_opens_upward() {
    let placed = fit(AREA, ON_THE_FLOOR, 0.0, 100.0).expect("room above");
    assert_eq!(
        placed,
        Fit {
            above: true,
            height: 100.0
        }
    );
    let band = list(AREA, ON_THE_FLOOR, 0.0, placed);
    assert_eq!(
        band,
        Band {
            top: 676.0,
            bottom: 776.0
        }
    );
    assert!(AREA.holds(band));
}

#[test]
fn a_list_taller_than_either_side_takes_the_roomier_one_and_is_capped() {
    // Below has 76 points, above has 176: the list goes up and scrolls
    // inside what it was given rather than painting past the floor.
    let anchor = Band {
        top: 200.0,
        bottom: 244.0,
    };
    let area = Band {
        top: 24.0,
        bottom: 320.0,
    };
    let placed = fit(area, anchor, 0.0, 2000.0).expect("room somewhere");
    assert_eq!(
        placed,
        Fit {
            above: true,
            height: 176.0
        }
    );
    assert!(area.holds(list(area, anchor, 0.0, placed)));
}

#[test]
fn a_list_taller_than_both_sides_stays_below_when_below_is_the_roomier() {
    let anchor = Band {
        top: 60.0,
        bottom: 104.0,
    };
    let area = Band {
        top: 24.0,
        bottom: 320.0,
    };
    let placed = fit(area, anchor, 0.0, 2000.0).expect("room below");
    assert_eq!(
        placed,
        Fit {
            above: false,
            height: 216.0
        }
    );
    assert!(area.holds(list(area, anchor, 0.0, placed)));
}

#[test]
fn an_unmeasured_list_wants_everything_and_gets_the_room() {
    // The first open: egui has laid the popup out zero times, so its size is
    // unknown. Infinity is the honest answer and the cap is the room.
    let placed = fit(AREA, ON_THE_FLOOR, 0.0, f32::INFINITY).expect("room above");
    assert_eq!(
        placed,
        Fit {
            above: true,
            height: 752.0
        }
    );
    assert!(AREA.holds(list(AREA, ON_THE_FLOOR, 0.0, placed)));
}

#[test]
fn a_control_with_no_room_on_either_side_opens_nothing() {
    let area = Band {
        top: 63.5,
        bottom: 100.25,
    };
    let anchor = Band {
        top: 40.0,
        bottom: 120.0,
    };
    assert_eq!(fit(area, anchor, 12.0, 30.0), None);
}

#[test]
fn a_control_outside_the_area_cannot_hand_back_a_list_outside_it() {
    // A control painted below the floor is a layout defect one level up, but
    // it must not become a list in the dead zone: the anchor is held inside
    // the band before either room is measured.
    let anchor = Band {
        top: 900.0,
        bottom: 950.0,
    };
    let placed = fit(AREA, anchor, 0.0, 300.0).expect("room above");
    let band = list(AREA, anchor, 0.0, placed);
    assert!(placed.above);
    assert_eq!(
        band,
        Band {
            top: 520.0,
            bottom: 820.0
        }
    );
    assert!(AREA.holds(band));
}

#[test]
fn holds_is_the_two_edges_and_a_hundredth_of_a_point_of_slack() {
    assert!(AREA.holds(AREA));
    assert!(AREA.holds(Band {
        top: 24.0 - 0.005,
        bottom: 820.0 + 0.005
    }));
    assert!(!AREA.holds(Band {
        top: 23.0,
        bottom: 800.0
    }));
    assert!(!AREA.holds(Band {
        top: 100.0,
        bottom: 821.0
    }));
}

/// **The ratchet** (bl-78c2). The class was fixed twice at other sites and
/// came back a third time because nothing asserted it. This is the
/// assertion: for every control position on three screen shapes, every list
/// height including *unmeasured*, and every gap, an opened list lies inside
/// the tappable area. Composing `fit` with `list` is the whole point — `fit`
/// decides and `list` says what egui will paint from that decision, so the
/// two disagreeing is exactly the defect.
#[test]
fn an_opened_list_never_leaves_the_tappable_area() {
    let areas = [
        AREA,
        Band {
            top: 0.0,
            bottom: 320.0,
        },
        Band {
            top: 63.5,
            bottom: 100.25,
        },
    ];
    let wants = [0.0, 18.0, 100.0, 2000.0, f32::INFINITY];
    let gaps = [0.0, 4.0, 12.0];
    let (mut opened, mut flipped, mut refused) = (0_u32, 0_u32, 0_u32);
    for area in areas {
        let reach = area.bottom - area.top + 40.0;
        for step in 0..40_u16 {
            let top = area.top - 20.0 + f32::from(step) * reach / 40.0;
            let anchor = Band {
                top,
                bottom: top + 44.0,
            };
            for wanted in wants {
                for gap in gaps {
                    let Some(placed) = fit(area, anchor, gap, wanted) else {
                        refused += 1;
                        continue;
                    };
                    opened += 1;
                    flipped += u32::from(placed.above);
                    let band = list(area, anchor, gap, placed);
                    assert!(
                        area.holds(band),
                        "{band:?} escaped {area:?}: anchor {anchor:?}, gap {gap}, wanted {wanted}"
                    );
                }
            }
        }
    }
    // A sweep that reached only one outcome would be a green light for
    // arithmetic nobody ran: all three have to happen.
    assert!(opened > 0, "no list ever opened");
    assert!(flipped > 0, "no list ever opened upward");
    assert!(refused > 0, "no control was ever out of room");
}
