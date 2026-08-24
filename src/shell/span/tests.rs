use super::{char_index, utf16_index};

#[test]
fn ascii_agrees_in_both_directions() {
    let s = "hello";
    for i in 0..=5 {
        assert_eq!(char_index(s, i), i);
        assert_eq!(utf16_index(s, i), i);
    }
}

#[test]
fn emoji_diverges() {
    // "a😀b": 'a'=1 unit, '😀'=2 units (surrogate pair), 'b'=1 unit.
    let s = "a\u{1F600}b";
    assert_eq!(char_index(s, 0), 0);
    assert_eq!(char_index(s, 1), 1);
    // Inside the surrogate pair: rounds up to the next char boundary.
    assert_eq!(char_index(s, 2), 2);
    assert_eq!(char_index(s, 3), 2);
    assert_eq!(utf16_index(s, 0), 0);
    assert_eq!(utf16_index(s, 1), 1);
    assert_eq!(utf16_index(s, 2), 3);
    assert_eq!(utf16_index(s, 3), 4);
}

#[test]
fn offsets_past_the_end_clamp() {
    let s = "a\u{1F600}";
    assert_eq!(char_index(s, 99), 2);
    assert_eq!(utf16_index(s, 99), 3);
    assert_eq!(char_index("", 4), 0);
    assert_eq!(utf16_index("", 4), 0);
}

#[test]
fn round_trip_on_char_boundaries() {
    let s = "n\u{00E9}\u{1F600} ok";
    for chars in 0..=s.chars().count() {
        assert_eq!(char_index(s, utf16_index(s, chars)), chars);
    }
}
