//! **What each control on the row SAYS** (DESIGN §13.2, bl-809d): the words
//! a selector wears when nobody has just tapped it.
//!
//! Every control in the block reads as a STATE and not as an instruction,
//! because that is what an operator is looking for when they glance at it —
//! *which model is this conversation on* is the question, and `model` is not
//! an answer to it. The wire already carries the answer: REMOTE §9.4's
//! assignments read says what the worker role is set to (bl-e9f9), and this
//! module is the one place that turns it into words.
//!
//! **Three sources in a fixed precedence, and the third is a real absence.**
//! An optimistic pick wins, because the operator just made it and the round
//! trip is seconds; the workspace's own assignment is next, because it is the
//! truth that overtakes the guess; and where NEITHER exists the control wears
//! its own name — never a value. That last arm is §8's rule and it is the
//! reason this is not a `unwrap_or_default`: an engine that predates the read
//! answers nothing, and a seat that painted `effort: off` there would be
//! inventing an assignment to keep its face tidy.

use super::{Effort, RoleRow};

/// The model face. The workspace's model is shown only under the workspace's
/// OWN provider: a model belongs to the provider that serves it, so painting
/// one beside a provider the operator has just switched to would name a pair
/// that does not exist.
pub fn model(picked: Option<String>, set: Option<&RoleRow>, provider: Option<&str>) -> String {
    picked
        .or_else(|| {
            set.filter(|row| Some(row.provider.as_str()) == provider)
                .map(|row| row.model.clone())
        })
        .unwrap_or_else(|| "model".to_owned())
}

/// The effort face, which carries the control's name as well as its value
/// (bl-b191): a magnitude names nothing, and `medium` alone was read as a
/// context size once already.
///
/// An assignment with no level is `off` — the absence carried as a real null
/// — and that is a value, so it is said. No assignment at all is not.
pub fn effort(picked: Option<String>, set: Option<&RoleRow>) -> String {
    match picked
        .or_else(|| set.map(|row| row.effort.clone().unwrap_or_else(|| Effort::label(None))))
    {
        Some(level) => format!("effort: {level}"),
        None => "effort".to_owned(),
    }
}

/// The priority face and the state the toggle stands in, which are one
/// reading: the word says which way it is set and the toggle's own fill says
/// it again, so an operator gets the answer from a glance or from a read.
pub fn priority(picked: Option<bool>, set: Option<&RoleRow>) -> (bool, String) {
    match picked.or_else(|| set.map(|row| row.priority)) {
        Some(on) => (on, format!("priority: {}", if on { "on" } else { "off" })),
        None => (false, "priority".to_owned()),
    }
}

#[cfg(test)]
mod tests;
