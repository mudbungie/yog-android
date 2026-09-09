//! The codec's shared field readers — the mirror of the server's
//! `boundary/codec/fields.rs`, trimmed to what this slice spends. A missing
//! field, a mistyped value and an out-of-range number each refuse **by name**,
//! because a reply read off the wire is a peer's statement about a world this
//! process cannot see, and a guessed answer is worse than none.
//!
//! **But strictness is bounded by the grows-only rule** (yog REMOTE §3.2,
//! bl-e598): `PROTOCOL` is a major and additions ship inside one, so a reader
//! must be able to meet a frame written by a newer engine of the same major
//! without falling over. Three parts, and this module is where two of them
//! live:
//!
//! 1. **An unknown KEY is ignored.** Structural here — every read names the
//!    key it wants, so a field nobody asks for is never seen. Nothing to do,
//!    and nothing that may be added: a `deny_unknown_fields` anywhere in this
//!    codec would break the rule.
//! 2. **A key stamped above the corpus `floor` reads as its DEFAULT when
//!    absent, and the default is the fact before the field existed.** Such a
//!    key is optional to read: [`opt`]/[`opt_val`] on the way in, and the
//!    default chosen so an engine that cannot spell it says the same thing it
//!    said before the field was minted. No field in today's vendored corpus is
//!    stamped above the floor, so this is the rule every future one is added
//!    under rather than a branch anything takes now. Where the absence must
//!    NOT read as the default — where the operator would be told a
//!    reassuring falsehood — [`crate::ledger::spells`] answers whether the
//!    engine could have said it at all, and the surface says *this engine
//!    cannot say* instead.
//! 3. **An unknown WORD in a vocabulary becomes that vocabulary's named
//!    catch-all**, carrying the word — see [`pick`]. The two exceptions are
//!    the reply envelope's `kind` and a request envelope's `op`: a reader
//!    asks only what it paints and an engine refuses only what it cannot act
//!    on, so those two stay strict and refuse by name, in band.

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

/// **A vocabulary word, read grows-only** (REMOTE §3.2): the table's answer
/// for a word this build knows, and `unknown(word)` — the vocabulary's own
/// named catch-all, carrying the token itself — for anything else.
///
/// It used to refuse the stray token by name, and that was the wrong of the
/// three answers. Reading it as the nearest known word is a lie; refusing the
/// frame throws away every field beside it, so one added word upstream blanks
/// a whole screen. The third answer keeps the row and says the truth about the
/// one field: the surface paints *unknown …: `<word>`* in resting ink, which
/// is neither a lie nor a blank.
pub(crate) fn pick<T: Clone>(
    obj: &Map<String, Value>,
    key: &str,
    table: &[(&str, T)],
    unknown: fn(String) -> T,
) -> Result<T, String> {
    let token = str_of(obj, key)?;
    match table.iter().find(|(word, _)| *word == token) {
        Some((_, value)) => Ok(value.clone()),
        None => Ok(unknown(token)),
    }
}

/// **How a catch-all reads on the glass** (REMOTE §3.2): the noun the
/// vocabulary is, and the word this build has never heard of. One home, so
/// every vocabulary says *unknown* the same way and no surface invents a
/// second phrasing — and so the sentence never reads as a value the engine
/// might really have sent.
pub(crate) fn unknown(noun: &str, word: &str) -> String {
    format!("unknown {noun}: {word}")
}

#[cfg(test)]
mod tests;
