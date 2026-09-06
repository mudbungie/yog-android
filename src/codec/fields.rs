//! The codec's shared field readers — the mirror of the server's
//! `boundary/codec/fields.rs`, trimmed to what this slice spends. Strictness
//! lives here: a missing field, a mistyped value and an out-of-range number
//! each refuse **by name**, because a reply read off the wire is a peer's
//! statement about a world this process cannot see, and a guessed answer is
//! worse than none.

use serde_json::{Map, Value};

/// A required string field, or the refusal naming it.
pub(crate) fn str_of(obj: &Map<String, Value>, key: &str) -> Result<String, String> {
    obj.get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("missing or non-string field {key:?}"))
}

/// A required boolean field.
pub(crate) fn bool_of(obj: &Map<String, Value>, key: &str) -> Result<bool, String> {
    obj.get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("missing or non-boolean field {key:?}"))
}

/// A required signed-integer field — an age, an exit status.
pub(crate) fn i64_of(obj: &Map<String, Value>, key: &str) -> Result<i64, String> {
    obj.get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("missing or non-integer field {key:?}"))
}

/// A required unsigned-integer field.
pub(crate) fn u64_of(obj: &Map<String, Value>, key: &str) -> Result<u64, String> {
    obj.get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("missing or non-integer field {key:?}"))
}

/// A required unsigned-integer field, narrowed to an index or count.
pub(crate) fn usize_of(obj: &Map<String, Value>, key: &str) -> Result<usize, String> {
    let n = u64_of(obj, key)?;
    usize::try_from(n).map_err(|_| format!("field {key:?} out of range"))
}

/// A required array field, cloned out — owned elements for owned rows.
pub(crate) fn arr_of(obj: &Map<String, Value>, key: &str) -> Result<Vec<Value>, String> {
    obj.get(key)
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| format!("missing or non-array field {key:?}"))
}

/// A required array field whose elements are all strings, named by the shape
/// asking for it. Two readers wanted the same six lines — a models listing
/// and a work diff's missing refs — and one of them would have drifted.
pub(crate) fn strings_of(
    obj: &Map<String, Value>,
    key: &str,
    kind: &str,
) -> Result<Vec<String>, String> {
    arr_of(obj, key)?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("{kind}: non-string element in field {key:?}"))
        })
        .collect()
}

/// An **optional** field read by a keyed reader: absent or `null` is `None` —
/// the one field shape where "not stated" is a value rather than a malformed
/// envelope — and anything else refuses by name on a mismatch.
pub(crate) fn opt<T>(
    obj: &Map<String, Value>,
    key: &str,
    read: fn(&Map<String, Value>, &str) -> Result<T, String>,
) -> Result<Option<T>, String> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => read(obj, key).map(Some),
    }
}

/// An optional field read by a value reader — for the object-shaped options.
pub(crate) fn opt_val<T>(
    obj: &Map<String, Value>,
    key: &str,
    read: fn(&Value) -> Result<T, String>,
) -> Result<Option<T>, String> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => read(v).map(Some),
    }
}

/// A token matched against its table, or the refusal naming both the key and
/// the stray token. The table is the parser; the encoder's `match` upstream is
/// the compile gate; the fixture tests here hold the two together.
pub(crate) fn pick<T: Copy>(
    obj: &Map<String, Value>,
    key: &str,
    table: &[(&str, T)],
) -> Result<T, String> {
    let token = str_of(obj, key)?;
    table
        .iter()
        .find(|(word, _)| *word == token)
        .map(|(_, value)| *value)
        .ok_or_else(|| format!("field {key:?}: unknown token {token:?}"))
}

#[cfg(test)]
mod tests;
