//! **The two armings a workspace carries** (yog §8.5, DESIGN §13.13): the
//! drone loop, and the alignment monitor watching what it commits.
//!
//! **The naming trap is the first fact about this family, and it costs an
//! afternoon to rediscover** (lernie DESIGN §4.33, whose ruling transfers
//! whole). `fleet` and `disband` are the **loop** — claim this project's top
//! ready ball and start a drone on it, up to a cap. `arm` and `disarm` are the
//! **alignment monitor** — a cheap model reads each commit against its goal
//! and records a verdict on the trail. Two families, two settings, two
//! carriers, and **one shared reply kind**: all four answer
//! `{"kind": "armed", "armed": BOOL}`, so no reader can tell which family an
//! answer belongs to by looking at it.
//!
//! **So every reader here reads the OP back instead.** The gesture states
//! which family it is; the reply states only a boolean; and the sentence the
//! seat paints is composed from the two together
//! (`seat::acts::fleet`). A seat that classified off the reply would be
//! guessing between two settings.
//!
//! **There is no `fleet` READ, and that is not an omission.** Whether a loop
//! is running, how full it is and when it last acted are on the `board`
//! answer, which the ball pane already paints (§13.9). One fact, one home.

use serde_json::{Map, Value, json};

use super::fields::{str_of, usize_of};

/// Which of the four, and what only it carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FleetAct {
    /// **Run this workspace's project up to `cap` drones at once.**
    Fleet { project: String, cap: usize },
    /// **Stop the loop.** It stops nothing that is running: everything
    /// already started is untouched and keeps its ball.
    Disband,
    /// **Raise the alignment monitor**, with the cheap model that reads each
    /// commit against its goal.
    Arm { model: String },
    /// **Lower it.** Every verdict already recorded on the trail stays there.
    Disarm,
}

impl FleetAct {
    /// The wire token this act posts — the label a control wears and the
    /// `act:` tag it carries, so the paint cannot show one word and post
    /// another.
    pub(crate) fn op(&self) -> &'static str {
        match self {
            Self::Fleet { .. } => "fleet",
            Self::Disband => "disband",
            Self::Arm { .. } => "arm",
            Self::Disarm => "disarm",
        }
    }

    /// **What name this act needs, and what to ask for it in.** `None` is an
    /// act that composes nothing. The sentence is what a disabled control says
    /// beside itself — and here it is load-bearing twice over, because two
    /// controls on one screen want DIFFERENT words out of one field, so the
    /// label is what says which (DESIGN §13.13).
    #[must_use]
    pub fn wants(&self) -> Option<&'static str> {
        match self {
            Self::Fleet { .. } => Some("name the project to run"),
            Self::Arm { .. } => Some("name the monitor's model"),
            Self::Disband | Self::Disarm => None,
        }
    }

    /// The same act carrying the composer's word and, for the loop, the cap
    /// the stepper stands at. One site knows which field the text is, which is
    /// what keeps the paint from having to.
    #[must_use]
    pub fn with(&self, text: String, cap: usize) -> Self {
        match self {
            Self::Fleet { .. } => Self::Fleet { project: text, cap },
            Self::Arm { .. } => Self::Arm { model: text },
            Self::Disband => Self::Disband,
            Self::Disarm => Self::Disarm,
        }
    }

    /// **What the engine's boolean MEANS, read against the op that was
    /// sent.** The reply says only whether something is armed; which of the
    /// two settings it is talking about is this gesture's own name, and
    /// nothing but the sender knows it.
    #[must_use]
    pub fn said(&self, armed: bool) -> String {
        let subject = match self {
            Self::Fleet { .. } | Self::Disband => "the loop",
            Self::Arm { .. } | Self::Disarm => "the monitor",
        };
        let standing = if armed { "armed" } else { "not armed" };
        format!("{}: {subject} is {standing}", self.op())
    }
}

/// Encode one, with the workspace stated once.
pub(super) fn encode(workspace: &str, act: &FleetAct) -> Value {
    let mut map = Map::new();
    map.insert("op".to_owned(), json!(act.op()));
    map.insert("workspace".to_owned(), json!(workspace));
    match act {
        FleetAct::Fleet { project, cap } => {
            map.insert("project".to_owned(), json!(project));
            map.insert("cap".to_owned(), json!(cap));
        }
        FleetAct::Arm { model } => {
            map.insert("model".to_owned(), json!(model));
        }
        FleetAct::Disband | FleetAct::Disarm => {}
    }
    Value::Object(map)
}

/// Read one back — the inverse the conformance corpus is replayed through.
pub(super) fn decode(op: &str, o: &Map<String, Value>) -> Result<super::Act, String> {
    let act = match op {
        "fleet" => FleetAct::Fleet {
            project: str_of(o, "project")?,
            cap: usize_of(o, "cap")?,
        },
        "arm" => FleetAct::Arm {
            model: str_of(o, "model")?,
        },
        "disband" => FleetAct::Disband,
        _ => FleetAct::Disarm,
    };
    Ok(super::Act::Fleet {
        workspace: str_of(o, "workspace")?,
        act,
    })
}

#[cfg(test)]
mod tests;
