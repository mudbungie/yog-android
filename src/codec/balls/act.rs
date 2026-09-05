//! **The five acts on a ball** (DESIGN §13.9, bl-f36e): the half of the pane
//! that CHANGES the store. `balls.rs` beside it is the three reads; this is
//! `assign`, `release`, `close`, `create` and `update`.
//!
//! **One shape, because one subject.** All five carry a project and the `--as`
//! name and no workspace, and they arrive together — one pane, one row under
//! the thumb — so the address is stated once in `Act::Ball` and the choice is
//! [`BallAct`]. That is the server's own habit
//! rather than an invention here: yog folds the same five behind one
//! `Action::Ball(verb)`, and `codec::row` folds the conversation row's acts
//! the same way and says why.
//!
//! **The `--as` name is the WORKSPACE's, so this seat needs no identity**
//! (lernie DESIGN §4.35, whose ruling transfers whole). yog spells the field
//! as *"the ball's bound workspace name, never the operator `$USER`"*, and
//! binding a ball to a workspace IS that equality — so a seat that invented an
//! operator name would break the binding it was making. On this seat the name
//! is the focused workspace, which is also the only workspace the pane that
//! fires these is ever painted under.
//!
//! **Two of the five are DOORS rather than rows.** `create`'s body and each of
//! `update`'s three may be **absent**, and absence is a value: an empty string
//! asks the engine to blank a field nobody touched, so an encoder that wrote
//! `""` for an absent key would be saying something the operator did not.
//! Every optional key is therefore skipped rather than nulled — the same rule
//! `effort`'s string-or-null already keeps in this codec, third application.
//!
//! **The scheduling fields are NOT spelled, and that is a decision.** A
//! `create` or an `update` may carry `fields`: an ordered array of priority,
//! tag, parent and needs applications, each of which is a picker this pane
//! does not have. The desktop refused the same and recorded it by count and
//! reason; here the frame carrying one is **refused by name** rather than
//! read as the frame without it, because answering a bare edit to a gesture
//! that asked to reprioritise is exactly the silent misread REMOTE §3's third
//! rule forbids. `tests/conformance/requests.rs` records the count.

use serde_json::{Map, Value, json};

use super::super::Act;
use super::super::fields::{opt, str_of};

/// **What the ball pane fires.** The three that name only a ball, and the two
/// that carry words an operator typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BallAct {
    /// **Claim the ball for this workspace.** Undone by the `release` beside
    /// it, which is why nothing arms it.
    Assign { id: String },
    /// **Give it back.** Undone by an `assign`.
    Release { id: String },
    /// **Fold, squash and deliver.** yog's close merges the trunk into the
    /// worktree, squashes the work onto it and removes the worktree, and no
    /// verb reverses it — so this is the one act of the five that is ARMED
    /// (DESIGN §13.9; the idiom is `clear-trail`'s, §13.8).
    Close { id: String },
    /// **File a new ball**, in the project of the row it was fired from. The
    /// body is absent from this seat: a phone types a title, and a ball's
    /// prose is written where there is a keyboard.
    Create { title: String, body: Option<String> },
    /// **Amend one.** Undone by writing the old words back, so it is not
    /// armed. All three fields are optional on the wire and this seat spells
    /// the title; the other two round-trip so a frame that carries them is
    /// read rather than misread.
    Update {
        id: String,
        title: Option<String>,
        body: Option<String>,
        note: Option<String>,
    },
}

impl BallAct {
    /// **The wire's own op token**, which is also the control's label and the
    /// `act:` tag it carries (PARITY §4) — one name, so the paint cannot show
    /// a word and post another. `pub(crate)` for `RowAct::op`'s reason: it
    /// hands back a borrow and every caller is inside this crate.
    pub(crate) fn op(&self) -> &'static str {
        match self {
            Self::Assign { .. } => "assign",
            Self::Release { .. } => "release",
            Self::Close { .. } => "close",
            Self::Create { .. } => "create",
            Self::Update { .. } => "update",
        }
    }

    /// **The sentence a control states while it cannot fire**, or `None` when
    /// the act needs nothing typed. `RowAct::wants`' rule at a second site: a
    /// greyed control says a thing is not live and nothing about what would
    /// make it live.
    ///
    /// `pub` for `RowAct::wants`' reason too — it is a pure reading the
    /// ANDROID paint spends, so `pub(crate)` would be dead code on a host
    /// build.
    #[must_use]
    pub fn wants(&self) -> Option<&'static str> {
        match self {
            Self::Create { .. } => Some("type the title first"),
            Self::Update { .. } => Some("type the new title first"),
            Self::Assign { .. } | Self::Release { .. } | Self::Close { .. } => None,
        }
    }

    /// The same act carrying `text` as whatever field it takes, and the id of
    /// the row it was fired from. One site knows which field the text is,
    /// which is what keeps the paint from having to.
    #[must_use]
    pub fn on(&self, id: String, text: String) -> Self {
        match self {
            Self::Assign { .. } => Self::Assign { id },
            Self::Release { .. } => Self::Release { id },
            Self::Close { .. } => Self::Close { id },
            Self::Create { .. } => Self::Create {
                title: text,
                body: None,
            },
            Self::Update { .. } => Self::Update {
                id,
                title: Some(text),
                body: None,
                note: None,
            },
        }
    }
}

/// Encode one ball act, address first. The spellings are the server codec's,
/// field for field, and an absent optional key is **omitted** rather than
/// nulled — that is what makes absence a value here.
pub(crate) fn encode(project: &str, name: &str, act: &BallAct) -> Value {
    let op = act.op();
    match act {
        BallAct::Assign { id } | BallAct::Release { id } | BallAct::Close { id } => {
            json!({ "op": op, "project": project, "id": id, "name": name })
        }
        BallAct::Create { title, body } => {
            let mut map = addressed(op, project, name);
            said(&mut map, "title", Some(title));
            said(&mut map, "body", body.as_ref());
            Value::Object(map)
        }
        BallAct::Update {
            id,
            title,
            body,
            note,
        } => {
            let mut map = addressed(op, project, name);
            said(&mut map, "id", Some(id));
            for (key, value) in ["title", "body", "note"]
                .into_iter()
                .zip([title, body, note])
            {
                said(&mut map, key, value.as_ref());
            }
            Value::Object(map)
        }
    }
}

/// Read one back. The caller matches the five ops before it gets here, so the
/// last arm is not reachable from `request::decode` today — it refuses by name
/// anyway, for `codec::row`'s reason: a `_` that quietly answered some future
/// op with a release is the misread REMOTE §3's third rule forbids. Its own
/// test calls it directly.
pub(crate) fn decode(op: &str, o: &Map<String, Value>) -> Result<Act, String> {
    unscheduled(o, op)?;
    let act = match op {
        "assign" => BallAct::Assign {
            id: str_of(o, "id")?,
        },
        "release" => BallAct::Release {
            id: str_of(o, "id")?,
        },
        "close" => BallAct::Close {
            id: str_of(o, "id")?,
        },
        "create" => BallAct::Create {
            title: str_of(o, "title")?,
            body: said_of(o, "body")?,
        },
        "update" => BallAct::Update {
            id: str_of(o, "id")?,
            title: said_of(o, "title")?,
            body: said_of(o, "body")?,
            note: said_of(o, "note")?,
        },
        other => return Err(format!("ball: unknown op {other:?}")),
    };
    Ok(Act::Ball {
        project: str_of(o, "project")?,
        name: str_of(o, "name")?,
        act,
    })
}

/// The three keys every one of the five carries: the op, the project and the
/// `--as` stamp. Built as a map rather than through `json!` because the two
/// arms that use it go on inserting — an envelope finished in one expression
/// is written as one (`ball` above), and one that is not, is not.
fn addressed(op: &str, project: &str, name: &str) -> Map<String, Value> {
    [("op", op), ("project", project), ("name", name)]
        .into_iter()
        .map(|(key, said)| (key.to_owned(), Value::String(said.to_owned())))
        .collect()
}

/// Write one optional string, or write nothing at all.
fn said(map: &mut Map<String, Value>, key: &str, value: Option<&String>) {
    if let Some(said) = value {
        map.insert(key.to_owned(), Value::String(said.clone()));
    }
}

/// Read one optional string: absent is `None`, present must be a string.
fn said_of(o: &Map<String, Value>, key: &str) -> Result<Option<String>, String> {
    opt(o, key, str_of)
}

/// **The scheduling fields, refused by name.** A frame carrying `fields` asked
/// for a priority, a tag, a parent or a blocker, and this pane has no picker
/// for any of them — so it is refused rather than read as the edit without
/// them.
fn unscheduled(o: &Map<String, Value>, op: &str) -> Result<(), String> {
    if o.contains_key("fields") {
        return Err(format!("{op}: unimplemented ball fields"));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
