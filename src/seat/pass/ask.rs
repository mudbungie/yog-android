//! **One standing question, asked** — the read every gesture-shaped caller in
//! this seat makes, and the sentence a reply of the wrong kind earns.
//!
//! Split from [`super`] at the cap, on the seam that module's own doc already
//! draws: what a PASS means is up there — the grace, the cache write, the
//! published snapshot — and this is the single ask underneath it, which every
//! `seat::asks` and `seat::acts` file reaches for directly. They change for
//! unrelated reasons: a new read is a caller of this, and the grace is not.

use serde_json::Value;

use crate::codec::reply::Reply;
use crate::codec::{Ask, Gesture, encode};
use crate::transport::{Seat, Wire};

/// One standing question, and **the engine's own envelope beside the rows it
/// decoded to** (bl-de96). The raw value is what the cache stores, so the
/// file holds the wire's spelling rather than a second one this client would
/// have to keep in step — see `crate::cache`.
///
/// **The class rides out** (bl-eec1). This model opens a connection per ask,
/// so a broken channel is already re-dialled by the next pass and there is
/// nothing here to decide about retrying (bl-8641) — but there is something
/// to decide about SAYING: a channel that failed is a redial the banner can
/// wait out, and an answer this end cannot use is a fault it must not. So the
/// class crosses and [`answer`] is the same read with it dropped, for every
/// caller that only ever wanted the sentence.
pub(in crate::seat) fn wired(seat: &Seat, ask: &Ask) -> Result<(Reply, Value), Wire> {
    let stream = seat.ask(&encode(&Gesture::Ask(ask.clone())))?;
    let last = stream
        .last()
        .ok_or_else(|| Wire::Unusable(crate::transport::NO_ANSWER.to_owned()))?;
    // **The decoder's two errors are two different facts about who failed**
    // (bl-8bd0, `Seat::answered`'s own note): the OUTER one is a reply this
    // end cannot read, the INNER one is the engine's own `ok: false`
    // sentence. Neither is the channel, so neither waits out the grace — but
    // saying which is free here and a caller that wants only the sentence
    // gets it from [`answer`].
    match crate::codec::reply::decode(last) {
        Err(unreadable) => Err(Wire::Unusable(unreadable)),
        Ok(Err(refusal)) => Err(Wire::Refused(refusal)),
        Ok(Ok(reply)) => Ok((reply, last.clone())),
    }
}

/// [`wired`] with the class dropped — what every gesture-shaped read wants,
/// because a gesture's answer is a sentence for the banner either way.
pub(in crate::seat) fn answer(seat: &Seat, ask: &Ask) -> Result<(Reply, Value), String> {
    wired(seat, ask).map_err(Into::into)
}

/// The wrong-kind sentence names the kind, never the rows it carried. Shared
/// with `seat::acts`, which asks the same question of a receipt.
pub(in crate::seat) fn kind_err(asked: &str, got: &Reply) -> String {
    format!("{asked}: the engine answered {} instead", got.kind())
}
