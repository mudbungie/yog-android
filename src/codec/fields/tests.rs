//! Every reader, both directions: the named value comes back, and every
//! refusal names its offender — the strictness the whole codec leans on.

use super::{arr_of, bool_of, i64_of, opt, opt_val, pick, str_of, u64_of, usize_of};
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

#[test]
fn pick_matches_its_table_and_names_a_stray() {
    const T: [(&str, u8); 2] = [("one", 1), ("two", 2)];
    let o = obj(json!({ "k": "two", "bad": "three" }));
    assert_eq!(pick(&o, "k", &T).unwrap(), 2);
    assert_eq!(
        pick(&o, "bad", &T).unwrap_err(),
        "field \"bad\": unknown token \"three\""
    );
}
