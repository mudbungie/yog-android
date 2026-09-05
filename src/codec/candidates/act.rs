//! **The three acts on an obligation** (DESIGN §13.12): spread it over n
//! candidates, accept one, release one.
//!
//! **Two of them are a family and the third is a start.** `deliver` and
//! `retire` name a handle and nothing else, so they are one address and one
//! enum, for `codec::balls::act`'s reason exactly. `fan` carries a prepared
//! body — *"a fan is the start with n in the middle, not a second start path"*
//! (lernie DESIGN §4.36) — so it is its own [`Act`] variant beside the start
//! family rather than a third arm here holding a body the other two have no
//! use for.
//!
//! **The ball is always named.** All three take an optional `ball` upstream,
//! and omitting it is the bare project-repo gesture aimed at the integration
//! branch — a subject this seat has no row for. Every gesture here is composed
//! off a science row that names one, so the ball-less frames are refused **by
//! name** rather than read as the frame with a ball, which is the silent
//! misread REMOTE §3's third rule forbids.
//!
//! **The count is not floored here.** Upstream reads 1 and 0 as *materialize
//! nothing and hand back the ordinary claim binding*, which is a start and
//! this seat already has one — so the floor is the CONTROL's (DESIGN §13.12),
//! exactly as an arming is a property of the glass. A frame stating either
//! still reads: this table is the inverse of an encoder, and refusing a number
//! it can spell would be refusing a shape it understands.

use serde_json::{Map, Value, json};

use super::super::start::{Prepared, prepared_of};
use super::super::{Act, fields::str_of};

/// Which handle act, and what only it carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CandidateAct {
    /// **Accept one candidate**, with the delivery subject's own text.
    Deliver { handle: String, summary: String },
    /// **Release one**, and whether its source ref went with it is the
    /// project's declared retention answering — which the receipt states.
    Retire { handle: String },
}

impl CandidateAct {
    /// The wire token this act posts — the label a control wears and the
    /// `act:` tag it carries, so the paint cannot show one word and post
    /// another.
    pub(crate) fn op(&self) -> &'static str {
        match self {
            Self::Deliver { .. } => "deliver",
            Self::Retire { .. } => "retire",
        }
    }

    /// **What text this act needs, and what to ask for it in.** `None` is an
    /// act that composes nothing; the sentence is what a disabled control says
    /// beside itself, because a greyed control says a thing is not live and
    /// nothing about what would make it live.
    #[must_use]
    pub fn wants(&self) -> Option<&'static str> {
        match self {
            Self::Deliver { .. } => Some("say what this delivery is"),
            Self::Retire { .. } => None,
        }
    }
}

impl CandidateAct {
    /// The same act carrying the picked row's handle and, where it takes one,
    /// the composer's text. One site knows which field the text is, which is
    /// what keeps the paint from having to.
    #[must_use]
    pub fn on(&self, handle: String, text: String) -> Self {
        match self {
            Self::Deliver { .. } => Self::Deliver {
                handle,
                summary: text,
            },
            Self::Retire { .. } => Self::Retire { handle },
        }
    }
}

/// Encode a handle act, with the obligation stated once.
pub(in super::super) fn encode(project: &str, ball: &str, act: &CandidateAct) -> Value {
    let mut map = obligation(act.op(), project, ball);
    match act {
        CandidateAct::Deliver { handle, summary } => {
            map.insert("handle".to_owned(), json!(handle));
            map.insert("summary".to_owned(), json!(summary));
        }
        CandidateAct::Retire { handle } => {
            map.insert("handle".to_owned(), json!(handle));
        }
    }
    Value::Object(map)
}

/// Encode a fan: the obligation, the staged body to spend, and n.
pub(in super::super) fn encode_fan(
    project: &str,
    ball: &str,
    prepared: &Prepared,
    n: usize,
) -> Value {
    let mut map = obligation("fan", project, ball);
    map.insert("prepared".to_owned(), super::super::start::body(prepared));
    map.insert("n".to_owned(), json!(n));
    Value::Object(map)
}

/// The half all three share: the op word and the obligation.
fn obligation(op: &str, project: &str, ball: &str) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("op".to_owned(), json!(op));
    map.insert("project".to_owned(), json!(project));
    map.insert("ball".to_owned(), json!(ball));
    map
}

/// Read one back — the inverse the conformance corpus is replayed through.
pub(in super::super) fn decode(op: &str, o: &Map<String, Value>) -> Result<Act, String> {
    let project = str_of(o, "project")?;
    let ball = named(o, op)?;
    let act = match op {
        "deliver" => CandidateAct::Deliver {
            handle: str_of(o, "handle")?,
            summary: str_of(o, "summary")?,
        },
        "retire" => CandidateAct::Retire {
            handle: str_of(o, "handle")?,
        },
        _ => {
            return Ok(Act::Fan {
                project,
                ball,
                prepared: prepared_of(o.get("prepared").ok_or("fan: missing field \"prepared\"")?)?,
                n: super::super::fields::usize_of(o, "n")?,
            });
        }
    };
    Ok(Act::Candidate { project, ball, act })
}

/// The ball this gesture is about — refused by name when the frame names
/// none, for the reason in this file's header.
fn named(o: &Map<String, Value>, op: &str) -> Result<String, String> {
    match o.get("ball") {
        None => Err(format!("{op}: unimplemented without a ball")),
        Some(_) => str_of(o, "ball"),
    }
}
