//! **What became of an act, and the sentence a lost reply earns** (yog
//! REMOTE §3, bl-d1f1, consumed in bl-07b1). Split from `seat::acts` for the
//! reason `transport::wire` is split from its dialler: the taxonomy of an
//! outcome is read by callers that post nothing, and the acts themselves are
//! a file about gestures.
//!
//! The contract in one line: **an act with no reply is IN DOUBT, and the
//! recovery is a read — never a resend.** An act is not idempotent (§9.8: two
//! clicks of Nudge are two nudges), no idempotency token rides the envelope
//! and no redelivery slot exists for acts, so the only honest terminal for a
//! reply this end never received is to say so and name the read that settles
//! it. Asks are the opposite case and re-ask freely (§9.7), which is why
//! `seat::asks` has nothing of this in it.

use crate::transport::Wire;

/// **What became of an act.** Three and not two, because a lost reply is
/// neither: the engine may have completed the act, and this end cannot learn
/// which. The caller's whole duty on [`Self::InDoubt`] is to say so and to
/// send nothing again.
pub(super) enum Posted {
    /// The engine answered, and its answer was yes.
    Took,
    /// The engine answered, and its answer was no — definite: nothing ran.
    Refused(String),
    /// **The act was written and nothing answered it** — REMOTE §3's in doubt.
    InDoubt(String),
}

impl Posted {
    /// The banner's sentence for this outcome, or `None` when there is
    /// nothing to say.
    pub(crate) fn note(self) -> Option<String> {
        match self {
            Self::Took => None,
            Self::Refused(why) | Self::InDoubt(why) => Some(why),
        }
    }
}

/// **An act's wire failure, classed by REMOTE §3.** A channel that never
/// carried the gesture is an ordinary failure and reads as one; a channel that
/// carried it and brought nothing back leaves the act in doubt, and then the
/// sentence is [`doubted`]'s rather than the socket's.
pub(super) fn faulted(why: &Wire, act: &str, read: &str) -> Posted {
    if why.in_doubt() {
        Posted::InDoubt(doubted(act, &why.sentence(), read))
    } else {
        Posted::Refused(why.sentence())
    }
}

/// **The sentence an act in doubt earns**, in one place because it is one
/// contract. Three things, and the operator needs all three: that this is
/// doubt and not failure, that nothing was sent again and what a repeat would
/// cost, and the read that settles it — the world is the durable record
/// (REMOTE §9.8), so every act here can name the read that shows what became
/// of it.
fn doubted(act: &str, said: &str, read: &str) -> String {
    format!(
        "{act} may have run: the reply was lost ({said}). Nothing was sent again — \
         a repeat would be a second {act}. {read}"
    )
}
