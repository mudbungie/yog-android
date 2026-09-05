//! **The conversation's machinery** (yog §8.5, DESIGN §13.11): the reads
//! behind the transcript — what the conversation IS and may be done to
//! (`agent`), what its steps were and what one step recorded (`steps`,
//! `step`), the operable spine and the config governing it (`rail`,
//! `governing`), and the mail nothing has delivered yet (`inbox`).
//!
//! **They are one value and not six fields** (lernie DESIGN §4.32, whose
//! ruling transfers whole). Six questions about ONE subject, retired together
//! the moment that subject moves — six fields would be six places to remember,
//! and forgetting one paints one conversation's records under another's name.
//! [`Records`] therefore carries the workspace and the agent it was asked at,
//! which is §14's pairing law over rows and their focus, one surface along.
//!
//! **What is decoded is what this screen paints**, the codec's standing
//! grow-per-consumer rule (`codec.rs`) spent inside these shapes rather than
//! across them. Four things ride through unread and each is a decision:
//!
//! - the `agent` answer's **`held`** object — the parked call. §13.7's band
//!   already answers one, off the attention queue, which is that fact's one
//!   home; a second reader here would be a second authority for it.
//! - the `agent` answer's **`nudgeable`, `stoppable`, `stop_children`** —
//!   the same gates the conversation ROW carries (`codec::conv`), which is
//!   where `shell::controls` reads them. One fact, one home, again.
//! - a `step` record's **`value`** — `raw` read again by a parser. This seat
//!   paints the bytes, so the tree beside them is a second spelling of one
//!   file.
//! - a step row's **`auth_row`** — the §8.3 sign-in affordance beside a
//!   refusal. `login` is unbuilt here and cited in `parity.toml`; the day it
//!   is built is the day this field earns a reader.
//!
//! And one narrowing beside them: the `agent` answer's **`state` and
//! `flight`** are carried as the engine's own tokens rather than picked
//! against `codec::conv`'s tables, because nothing on this screen branches on
//! either — it paints them. That is what a step's `framing`, its `wound` and a
//! child card's `state` already do, and the conversation ROW keeps the strict
//! reading where controls actually branch.

mod agent;
mod inbox;
mod spine;
mod step;
mod steps;

#[cfg(test)]
mod tests;

pub use agent::{Agent, Context, SeatRow};
pub use inbox::Mail;
pub use spine::{Card, Governing, Notch, Rail};
pub use step::{Log, Record, Step, ToolRecord};
pub use steps::{Orphan, StepRow, Steps};

pub(super) use agent::agent_of;
pub(in crate::codec) use agent::words;
pub(super) use inbox::mail;
pub(super) use spine::{governing_of, rail_of};
pub(super) use step::step_of;
pub(super) use steps::steps_of;

/// **Everything the records screen holds**, and the conversation it is about.
///
/// The pair is carried rather than assumed for [`super::Pane`]'s reason
/// exactly: a screen that painted one conversation's steps under another's
/// name would be the §14 pairing law broken at a new surface, and the answer
/// is the same one — the value states its own subject, and the paint refuses
/// what disagrees with the focus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Records {
    pub workspace: String,
    pub agent: String,
    pub head: Agent,
    pub steps: Steps,
    pub rail: Rail,
    pub governing: Governing,
    pub inbox: Vec<Mail>,
    /// **The config lineages this conversation's workspace holds** (DESIGN
    /// §13.14). The one read of the set that is about the WORKSPACE rather
    /// than the conversation, and it is here because what it names is what
    /// `governing`'s `follows` is one of — a list with no home of its own
    /// would be a screen about a word.
    pub lineages: Vec<super::Lineage>,
    /// **The one step an operator drilled into**, or none. It is not asked
    /// with the five above: `step` is about ONE step and the five are about
    /// the conversation, so a standing read of it would have to invent a
    /// selection and then hold it — a second authority for a row somebody
    /// tapped (lernie §4.32). The answer echoes back the `seq` it was asked
    /// by, so the paint asks THIS value which row it belongs to and nothing
    /// here remembers a second name for it.
    pub drilled: Option<Step>,
}

impl Records {
    /// Whether these records are about the conversation now focused. The one
    /// reading of the pair above, so no surface takes a second.
    #[must_use]
    pub fn about(&self, workspace: &str, agent: &str) -> bool {
        self.workspace == workspace && self.agent == agent
    }
}
