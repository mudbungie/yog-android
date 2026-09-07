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
//! **How far an answer stands is a second word** (REMOTE §5.5 at PROTOCOL 18,
//! yog bl-94a5). `scope` rides beside the verdict in both directions over
//! `call | conversation | workspace`: the held call alone (what every answer
//! used to be, and still the default), the class of that call for this
//! conversation and its descent, or that class for every conversation in the
//! workspace. An operator holding a conversation otherwise answers the same
//! question for every call of a kind they have already decided about.
//!
//! **Required in both directions**, never optional-defaults-to-`call`: an
//! absent field would let the two ends disagree about how wide an instruction
//! was, and *wider* is the reading nobody may arrive at by accident. That is
//! `reply/advertised`'s precedent at 8 and the tool window's at 15, applied to
//! the one gesture in this codec that authorizes an action.
//!
//! **The class the wide scopes stand over is the engine's**, derived from the
//! sentence it wrote into the hold mark — so this seat spells a decision and
//! never a key, exactly as it spells a conversation and never a `tool_use`.
//! A destructive or credential-reaching call takes the bare scope only, and
//! that refusal is the engine's too: it arrives in band, which is where every
//! other capability decision this seat does not own arrives.
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

/// **How far one answer stands** (PROTOCOL 18). The module doc says why it is
/// required rather than defaulted; this is the vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// The held call, and nothing else. The safe reading, and the one a
    /// control gives without being asked.
    Call,
    /// The held call's class — the same tool at the same reach — for this
    /// conversation and everything below it.
    Conversation,
    /// That class for every conversation in the workspace.
    Workspace,
}

impl Scope {
    /// Every scope, narrowest first, which is also the order a chooser offers
    /// them in: the default is the first thing a thumb meets, and widening is
    /// a deliberate move rightward.
    pub(crate) const ALL: [Self; 3] = [Self::Call, Self::Conversation, Self::Workspace];

    /// The engine's own word — the wire token and the control's label at once.
    pub(crate) fn word(self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Conversation => "conversation",
            Self::Workspace => "workspace",
        }
    }
}

impl Default for Scope {
    /// **The narrow one, always.** A scope this seat did not ask about must be
    /// the one that settles the least; every widening is somebody's tap.
    fn default() -> Self {
        Self::Call
    }
}

/// Encode the answer gesture: the conversation, the verdict, and how far it
/// stands.
pub(crate) fn encode(workspace: &str, agent: &str, verdict: Verdict, scope: Scope) -> Value {
    json!({ "op": "answer", "workspace": workspace, "agent": agent,
            "verdict": verdict.word(), "scope": scope.word() })
}

/// Read one back. The verdict and the scope are each found among the words
/// their own type owns, so the tokens have one home and an unknown one refuses
/// by name.
pub(crate) fn decode(o: &Map<String, Value>) -> Result<(String, String, Verdict, Scope), String> {
    Ok((
        str_of(o, "workspace")?,
        str_of(o, "agent")?,
        verdict_of(o, "answer")?,
        scope_of(o, "answer")?,
    ))
}

/// The scope token, read for whoever is asking — the gesture on the way out
/// and the receipt on the way back, `verdict_of`'s shape one field along.
fn scope_of(o: &Map<String, Value>, whose: &str) -> Result<Scope, String> {
    let word = str_of(o, "scope")?;
    Scope::ALL
        .into_iter()
        .find(|s| s.word() == word)
        .ok_or_else(|| format!("{whose}: unknown scope {word:?}"))
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
    /// **How far the engine took the answer** (PROTOCOL 18) — read back rather
    /// than assumed from what was sent, because a receipt that echoed the ask
    /// would prove nothing about how wide the instruction actually landed.
    pub scope: Scope,
    pub advanced: bool,
}

/// Read the answer's receipt.
pub(crate) fn answered_of(o: &Map<String, Value>) -> Result<Answered, String> {
    Ok(Answered {
        tool_use: str_of(o, "tool_use")?,
        tool: str_of(o, "tool")?,
        verdict: verdict_of(o, "answered")?,
        scope: scope_of(o, "answered")?,
        advanced: super::fields::bool_of(o, "advanced")?,
    })
}

#[cfg(test)]
mod tests;
