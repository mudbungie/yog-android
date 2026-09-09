//! Every reader, both directions: the named value comes back, and every
//! refusal names its offender — the strictness the whole codec leans on.

use super::{arr_of, bool_of, i64_of, opt, opt_val, pick, str_of, u64_of, unknown, usize_of};
use serde_json::{Map, Value, json};

fn obj(v: Value) -> Map<String, Value> {
    v.as_object().cloned().unwrap()
}

#[test]
fn str_of_reads_and_refuses() {
    let o = obj(json!({ "s": "x", "n": 3 }));
    assert_eq!(str_of(&o, "s").unwrap(), "x");
    assert_eq!(
        str_of(&o, "n").unwrap_err(),
        "missing or non-string field \"n\""
    );
    assert!(str_of(&o, "gone").unwrap_err().contains("\"gone\""));
}

#[test]
fn bool_of_reads_and_refuses() {
    let o = obj(json!({ "b": true, "s": "x" }));
    assert!(bool_of(&o, "b").unwrap());
    assert_eq!(
        bool_of(&o, "s").unwrap_err(),
        "missing or non-boolean field \"s\""
    );
}

#[test]
fn i64_of_reads_and_refuses() {
    let o = obj(json!({ "n": -4, "s": "x" }));
    assert_eq!(i64_of(&o, "n").unwrap(), -4);
    assert_eq!(
        i64_of(&o, "s").unwrap_err(),
        "missing or non-integer field \"s\""
    );
}

#[test]
fn u64_of_reads_and_refuses() {
    let o = obj(json!({ "n": 4, "neg": -1 }));
    assert_eq!(u64_of(&o, "n").unwrap(), 4);
    assert_eq!(
        u64_of(&o, "neg").unwrap_err(),
        "missing or non-integer field \"neg\""
    );
}

#[test]
fn usize_of_narrows() {
    let o = obj(json!({ "n": 7 }));
    assert_eq!(usize_of(&o, "n").unwrap(), 7);
    assert!(usize_of(&o, "gone").is_err());
}

#[test]
fn arr_of_reads_and_refuses() {
    let o = obj(json!({ "a": [1, 2], "s": "x" }));
    assert_eq!(arr_of(&o, "a").unwrap(), vec![json!(1), json!(2)]);
    assert_eq!(
        arr_of(&o, "s").unwrap_err(),
        "missing or non-array field \"s\""
    );
}

#[test]
fn opt_reads_absent_null_present_and_mismatch() {
    let o = obj(json!({ "s": "x", "n": 3, "z": null }));
    assert_eq!(opt(&o, "gone", str_of).unwrap(), None);
    assert_eq!(opt(&o, "z", str_of).unwrap(), None);
    assert_eq!(opt(&o, "s", str_of).unwrap(), Some("x".to_owned()));
    assert!(opt(&o, "n", str_of).is_err());
}

#[test]
fn opt_val_reads_absent_null_present_and_mismatch() {
    fn s(v: &Value) -> Result<String, String> {
        v.as_str()
            .map(str::to_owned)
            .ok_or("not a string".to_owned())
    }
    let o = obj(json!({ "s": "x", "n": 3, "z": null }));
    assert_eq!(opt_val(&o, "gone", s).unwrap(), None);
    assert_eq!(opt_val(&o, "z", s).unwrap(), None);
    assert_eq!(opt_val(&o, "s", s).unwrap(), Some("x".to_owned()));
    assert!(opt_val(&o, "n", s).is_err());
}

/// **The grows-only pick** (REMOTE §3.2): the table's answer for a word this
/// build knows, the vocabulary's own catch-all carrying the word for anything
/// else, and still a refusal for a field that is missing or is not a string —
/// a word this build cannot spell is an engine that grew, a field that is not
/// there at all is a frame that is wrong.
#[test]
fn pick_matches_its_table_and_carries_a_stray_word() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Word {
        Two,
        Unknown(String),
    }
    const T: [(&str, Word); 1] = [("two", Word::Two)];
    let o = obj(json!({ "k": "two", "bad": "three", "n": 3 }));
    assert_eq!(pick(&o, "k", &T, Word::Unknown).unwrap(), Word::Two);
    assert_eq!(
        pick(&o, "bad", &T, Word::Unknown).unwrap(),
        Word::Unknown("three".to_owned())
    );
    assert_eq!(
        pick(&o, "n", &T, Word::Unknown).unwrap_err(),
        "missing or non-string field \"n\""
    );
}

/// One home for the sentence, so eleven vocabularies say *unknown* alike.
#[test]
fn a_catch_all_reads_as_the_noun_and_the_word() {
    assert_eq!(unknown("framing", "cindered"), "unknown framing: cindered");
}
