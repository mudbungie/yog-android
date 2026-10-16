//! The table's invariants and the file's two projections.

use super::*;

#[test]
fn every_knob_has_a_distinct_key_a_sentence_and_a_default_inside_its_bounds() {
    let keys: BTreeSet<_> = KNOBS.iter().map(|knob| knob.key).collect();
    assert_eq!(keys.len(), KNOBS.len());
    let rest = Settings::default();
    for knob in KNOBS {
        assert!(
            !knob.help.is_empty() && !knob.label.is_empty(),
            "{}",
            knob.key
        );
        match (knob.kind, (knob.get)(&rest)) {
            (Kind::Toggle, Knobbed::On(_)) => {}
            (Kind::Number { floor, ceiling, .. }, Knobbed::At(at)) => {
                assert!((floor..=ceiling).contains(&at), "{}", knob.key);
            }
            (kind, held) => unreachable!("{}: {kind:?} holds {held:?}", knob.key),
        }
    }
}

#[test]
fn only_what_is_off_its_default_is_written_and_the_round_trip_is_exact() {
    let mut set = Settings::default();
    assert_eq!(to_json(&set), json!({}));
    set.scale = 1.25;
    set.set(Toggle::Bubbles, false);
    set.set(Toggle::Steps, true);
    let written = to_json(&set);
    assert_eq!(
        written,
        json!({"scale": 1.25, "bubbles": false, "steps": true})
    );
    assert_eq!(from_json(&written), set);
}

#[test]
fn a_wrong_shape_an_unknown_key_and_a_null_are_each_the_default() {
    let read = from_json(&json!({"scale": "big", "help": 3, "dock": null, "wheel": 9}));
    assert_eq!(read, Settings::default());
    let other = from_json(&json!({"scale": true, "help": false}));
    assert!(!other.on(Toggle::Help));
    assert!((other.scale - SCALE).abs() < f32::EPSILON);
}

#[test]
fn the_policy_is_read_off_the_three_switches() {
    let mut set = Settings::default();
    assert_eq!(
        set.auto(),
        AutoExpand {
            responses: true,
            others: false,
            fold: true
        }
    );
    set.set(Toggle::Full, false);
    set.set(Toggle::Fold, false);
    assert!(!set.auto().responses && !set.auto().fold);
    assert!((knob("scale").is_some()) && knob("wheel").is_none());
}

#[test]
fn the_file_reads_back_what_was_written_and_an_absent_or_broken_one_is_the_defaults() {
    let dir = std::env::temp_dir().join(format!("settings-{}", std::process::id()));
    let path = dir.join("deep").join(FILE);
    assert_eq!(read(&path), Settings::default());
    let set = Settings {
        scale: 1.5,
        ..Settings::default()
    };
    write(&path, &set).unwrap();
    assert_eq!(read(&path), set);
    std::fs::write(&path, "{ not json").unwrap();
    assert_eq!(read(&path), Settings::default());
    let blocked = dir.join("deep").join(FILE).join("under-a-file");
    assert!(write(&blocked, &set).is_err());
    std::fs::remove_dir_all(&dir).unwrap();
}
