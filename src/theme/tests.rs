//! What the language promises, asserted: legible ink, an elevation ladder that
//! climbs, accents that are soft, distinct and readable, and a touch floor
//! that is a floor.

use super::*;

/// WCAG relative luminance of one channel.
fn channel(byte: u8) -> f64 {
    let c = f64::from(byte) / 255.0;
    if c <= 0.039_28 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn luminance(rgb: Rgb) -> f64 {
    0.2126 * channel(rgb[0]) + 0.7152 * channel(rgb[1]) + 0.0722 * channel(rgb[2])
}

/// WCAG contrast ratio, lighter over darker.
fn contrast(a: Rgb, b: Rgb) -> f64 {
    let (la, lb) = (luminance(a) + 0.05, luminance(b) + 0.05);
    la.max(lb) / la.min(lb)
}

/// HSV saturation: how far a colour is from grey.
fn saturation(rgb: Rgb) -> f64 {
    let max = f64::from(*rgb.iter().max().unwrap());
    let min = f64::from(*rgb.iter().min().unwrap());
    if max == 0.0 { 0.0 } else { (max - min) / max }
}

#[test]
fn ink_is_legible_on_every_surface() {
    for ground in [GROUND, SURFACE, RAISED] {
        assert!(contrast(INK, ground) >= 7.0, "ink on {ground:?}");
        assert!(contrast(INK_WEAK, ground) >= 4.5, "weak ink on {ground:?}");
        assert!(
            contrast(INK_FAINT, ground) >= 2.0,
            "faint ink on {ground:?}"
        );
    }
}

#[test]
fn elevation_climbs_and_ink_descends() {
    let ladder = [GROUND, SURFACE, RAISED, HAIRLINE, INK_FAINT, INK_WEAK, INK];
    for pair in ladder.windows(2) {
        assert!(luminance(pair[0]) < luminance(pair[1]), "{pair:?}");
    }
}

#[test]
fn accents_are_soft() {
    for state in STATES {
        assert!(saturation(accent(state)) <= 0.66, "{state:?}");
    }
    assert!(saturation(BRAND) <= 0.66);
}

#[test]
fn accents_read_on_every_surface() {
    for state in STATES {
        for ground in [GROUND, SURFACE, RAISED] {
            assert!(
                contrast(accent(state), ground) >= 4.5,
                "{state:?} on {ground:?}"
            );
        }
    }
    assert!(contrast(BRAND, GROUND) >= 4.5);
}

#[test]
fn six_states_six_accents_and_the_brand_is_one_of_them() {
    let seen: Vec<Rgb> = STATES.iter().map(|s| accent(*s)).collect();
    let distinct: std::collections::BTreeSet<Rgb> = seen.iter().copied().collect();
    assert_eq!(distinct.len(), 6);
    assert!(seen.contains(&BRAND), "the brand is not a seventh colour");
    assert_eq!(BRAND, accent(State::Working));
}

#[test]
fn tones_read_as_states() {
    assert_eq!(tone(Tone::Plain), INK);
    assert_eq!(tone(Tone::Weak), INK_WEAK);
    assert_eq!(tone(Tone::Good), accent(State::Rest));
    assert_eq!(tone(Tone::Bad), accent(State::Error));
    assert_eq!(tone(Tone::Live), accent(State::Inference));
    assert_eq!(tone(Tone::InFlight), accent(State::Working));
}

#[test]
fn speakers_differ_by_weight_not_hue() {
    assert_eq!(speaker(Role::User), BRAND);
    let rules = [
        speaker(Role::Model),
        speaker(Role::Peer),
        speaker(Role::Ended),
    ];
    // Less than half as saturated as the least saturated COLOURED accent (at
    // rest is ink by design): an ink may lean toward the ground's own violet,
    // but it must never read as a hue.
    let least = STATES
        .iter()
        .filter(|s| **s != State::Rest)
        .map(|s| saturation(accent(*s)))
        .fold(1.0, f64::min);
    for rule in rules {
        assert!(
            saturation(rule) < least / 2.0,
            "{rule:?} is a hue, and hue means state"
        );
    }
    for pair in rules.windows(2) {
        assert!(luminance(pair[0]) > luminance(pair[1]));
    }
}

#[test]
fn the_scales_climb_and_the_floor_is_a_floor() {
    let spaces = [space::XS, space::S, space::M, space::L, space::XL];
    let types = [
        type_scale::SMALL,
        type_scale::MONO,
        type_scale::BODY,
        type_scale::HEADING,
    ];
    for scale in [spaces.as_slice(), types.as_slice()] {
        for pair in scale.windows(2) {
            assert!(pair[0] < pair[1], "{pair:?}");
        }
    }
    const { assert!(TOUCH >= 48.0) }
    const { assert!(RULE > 0.0 && RADIUS > 0) }
}
