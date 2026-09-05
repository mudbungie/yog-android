//! **The capability boundary's two gestures, as this seat spends them** (yog
//! DESIGN §8.6, VISION §4.11; DESIGN §13.7, bl-b39d): answer the invocation
//! parked at a conversation, and take that conversation's tool auto-approval
//! away or give it back.
//!
//! **The answer names a conversation, never a call.** The wire carries no
//! `tool_use` on the way out and that is the engine's design: it reads the
//! held mark off the branch at fire time, so *"nothing is typed and nothing
//! can be spent by a different call"*. This seat holds the id anyway — the
//! queue read carries it (`codec::queue`) — and paints it nowhere and sends it
//! nowhere: what it is for is telling one parked call from the next one, so a
//! band that is still on the glass after an answer is not read as the same
//! call twice.
//!
//! **Three verdicts and no fourth.** `pass` lets this one call through,
//! `refuse` declines it in band — the model reads why and carries on — and
//! `hold` keeps it parked even where the policy would now let it by. Nothing
//! here stops an agent: yog's own note is that `litany stop` mid-tool-window
//! wedges the branch permanently, so declining is in-band and parking is a
//! park.
//!
//! **The floor pair rides `RowAct`** (`codec::row`) rather than this file's
//! own act, because it is a conversation-level gesture that needs nothing
//! typed and nothing read — the §13.5 menu's exact class. What lives here is
//! what the two of them ANSWER, which is a receipt shape of their own.

use serde_json::{Map, Value, json};

use super::fields::str_of;

/// What an answer says about the parked call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Let this one call through, and drive the conversation on.
    Pass,
    /// Decline it in band: the model is told, and carries on.
    Refuse,
    /// Keep it parked. The one verdict that drives nothing — the operator is
    /// saying *stay where you are*, and a driver launched to re-park would
    /// spend a process reaching the state it is already in.
    Hold,
}

impl Verdict {
    /// Every verdict, in the order a thumb meets them: the release first,
    /// because it is the answer most calls earn.
    pub(crate) const ALL: [Self; 3] = [Self::Pass, Self::Refuse, Self::Hold];

    /// The engine's own word for this verdict — the wire token, the control's
    /// label and its `act:` reading all at once, so a control cannot say one
    /// thing and send another.
    pub(crate) fn word(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Refuse => "refuse",
            Self::Hold => "hold",
        }
    }

    /// **Whether this verdict releases the branch.** `pass` and `refuse` both
    /// move it on (one executes, one declines in band); `hold` moves nothing.
    /// Read here rather than at the paint because it is what decides whether
    /// an unadvanced receipt is worth a sentence.
    pub(crate) fn releases(self) -> bool {
        !matches!(self, Self::Hold)
    }
}

/// Encode the answer gesture: the conversation, and the verdict.
pub(crate) fn encode(workspace: &str, agent: &str, verdict: Verdict) -> Value {
    json!({ "op": "answer", "workspace": workspace, "agent": agent,
            "verdict": verdict.word() })
}

/// Read one back. The verdict is found among the three that own the words, so
/// the tokens have one home and an unknown one refuses by name.
pub(crate) fn decode(o: &Map<String, Value>) -> Result<(String, String, Verdict), String> {
    Ok((
        str_of(o, "workspace")?,
        str_of(o, "agent")?,
        verdict_of(o, "answer")?,
    ))
}

/// The verdict token, read for whoever is asking — the gesture on the way out
/// and the receipt on the way back, which carries a verdict and no address and
/// so cannot go through the gesture's reader.
fn verdict_of(o: &Map<String, Value>, whose: &str) -> Result<Verdict, String> {
    let word = str_of(o, "verdict")?;
    Verdict::ALL
        .into_iter()
        .find(|v| v.word() == word)
        .ok_or_else(|| format!("{whose}: unknown verdict {word:?}"))
}

/// **What an answer earns**: the call it landed on, and whether the branch was
/// driven on afterwards.
///
/// `advanced` is not decoration. A releasing verdict whose launch did not
/// happen leaves the answer recorded and the conversation exactly where it
/// was — nothing will move until something advances it — and that is the one
/// outcome an operator cannot see by looking at the screen they are on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answered {
    pub tool_use: String,
    pub tool: String,
    pub verdict: Verdict,
    pub advanced: bool,
}

/// Read the answer's receipt.
pub(crate) fn answered_of(o: &Map<String, Value>) -> Result<Answered, String> {
    Ok(Answered {
        tool_use: str_of(o, "tool_use")?,
        tool: str_of(o, "tool")?,
        verdict: verdict_of(o, "answered")?,
        advanced: super::fields::bool_of(o, "advanced")?,
    })
}

#[cfg(test)]
mod tests;
