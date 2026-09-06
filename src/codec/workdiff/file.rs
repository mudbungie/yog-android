//! **Which file a patch is asked for** — the `work-diff` read's one
//! parameter, and the half of this family that crosses OUTWARD.
//!
//! It is its own file for the seam the module beside it is split on: the rows
//! are what an answer SAYS, and this is what the ask NAMES. Upstream states
//! the same distinction — *"as a parameter naming which thing you are asking
//! about"* — and the two never share a reader.
//!
//! **The handle is the attempt and empty is the claim**, exactly as a diff
//! row's own handle is, so the address an operator taps is spelled once and
//! composed from the row that carries it. Absent rather than empty on the
//! wire: a claim's frame states no handle at all (`codec::balls`' rule).

use serde_json::{Map, Value, json};

use super::super::fields::str_of;

/// The file a patch is asked for: the ball whose attempt holds it, the path
/// inside that attempt, and which attempt — empty for the ball's own claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkFile {
    pub ball: String,
    pub path: String,
    pub handle: String,
}

/// The `file` object of a `work-diff` frame.
pub(in crate::codec) fn encode(file: &WorkFile) -> Value {
    let mut map = Map::new();
    map.insert("ball".to_owned(), json!(file.ball));
    if !file.handle.is_empty() {
        map.insert("handle".to_owned(), json!(file.handle));
    }
    map.insert("path".to_owned(), json!(file.path));
    Value::Object(map)
}

/// The same object read back — the inverse the corpus round trip proves.
pub(in crate::codec) fn decode(v: &Value) -> Result<WorkFile, String> {
    let o = v
        .as_object()
        .ok_or("work-diff: \"file\" is not an object")?
        .clone();
    Ok(WorkFile {
        ball: str_of(&o, "ball")?,
        path: str_of(&o, "path")?,
        handle: o
            .get("handle")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
    })
}
