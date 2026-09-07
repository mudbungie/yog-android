//! **The ops trail** (yog DESIGN §4.2, §7.3; REMOTE §9.17) — the last actions
//! this engine took, and the two acts an operator has over them.
//!
//! The trail is the world's durable record of what ran: every subprocess the
//! engine forked, with the directory it ran in, what it said and what it
//! exited. It is the read every other recovery sentence in this client points
//! at — *"the world is the durable record"* (REMOTE §9.8) — and until this
//! surface it was the one read a phone could not make.
//!
//! **This decoder reads the row's own words and classifies nothing.** yog
//! answers a failed action four ways it derives itself (the sentinel table,
//! the `128 + n` signal reading, the retirement key, the ack watermark), and
//! REMOTE §9.17 put all four on the wire as `failed`, `exit_label` and
//! `standing` so that no seat re-implements them — *"a seat that wanted the
//! banner had to re-implement … five derivations this document names one home
//! for apiece, and whose failure mode is a seat quietly disagreeing"*. Since
//! the protocol-13 re-vendor (bl-8e3c) all three are read here, and `exit`
//! rides beside them as the number the engine logged: a seat that read a
//! verdict out of it would be the disagreement §9.17 exists to prevent.
//!
//! **`standing` is typed, and the queue's `signals` are not — deliberately
//! both.** §9.17 states the five words as *"total, never absent"*, a closed
//! vocabulary whose every member the trail paints differently; a sixth would
//! reach this seat as a red corpus fixture at the re-vendor that brought it,
//! which is the right moment. A signal is an open list painted as its tokens.
//!
//! **The row says who said it** (REMOTE §9.20, PROTOCOL 16). Two seats
//! depositing into one conversation within the same second left two rows
//! spelling `argv`, `cwd`, `origin` and `exit` and nothing that told them
//! apart, and on a shared workspace that is the question a trail is read to
//! answer. `client` is required for the reason `standing` is: a row that could
//! omit it would make "an older engine" and "yog itself" one shape, and yog
//! itself has a true answer — `local`.
//!
//! **`ack` and `clear-trail` carry nothing, and that is the whole shape.**
//! Neither names a row: the ack is a watermark over the trail as it stands and
//! the clear is a truncation of it, both world-wide, so there is nothing for a
//! client to select and nothing for it to get wrong.

use serde_json::Value;

use super::fields::{bool_of, i64_of, pick, str_of};

/// One action the engine took.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpRow {
    /// When it ran, in the engine's own stamp — carried as the string it
    /// wrote rather than a parsed instant, because a phone's job is to show
    /// what the record says and no clock here is the record's.
    pub ts: String,
    /// **Which surface owes this row a reading** (yog bl-48f8): `balls`,
    /// `world`, `conversation`. Stored on the line because it cannot be
    /// derived, and the key a banner would group by.
    pub origin: String,
    /// The command line, as the engine logged it.
    pub argv: String,
    /// Where it ran.
    pub cwd: String,
    /// What it exited with. Negative values are the engine's own sentinels
    /// (a handoff, a drift observation) — which is exactly why nothing here
    /// reads meaning into the number.
    pub exit: i64,
    pub stdout: String,
    pub stderr: String,
    /// **Whether this line failed** — the row's own question, answerable of
    /// it held alone (REMOTE §9.17), and never re-derived from `exit`.
    pub failed: bool,
    /// The engine's own reading of `exit`: `exit 1`, or the sentinel's
    /// sentence (*detached — handed off, no exit to observe*).
    pub exit_label: String,
    /// Where this row stands in the tail — the alarm's whole state.
    pub standing: Standing,
    /// **Who made this act** (REMOTE §9.20, PROTOCOL 16): a connection
    /// certificate's common name, or `local` for the acts yog took itself —
    /// the window's, the `gestures/` inbox's, its own loops'. Read strictly,
    /// like `standing`: it is the one fact on this row that cannot be
    /// recovered later, because presence is a point-in-time observation by
    /// design and nothing after the fact can say who a row belonged to.
    ///
    /// It is carried as the engine's word and classified no further. `local`
    /// is §3's reserved in-world identity and every other value is a leaf's
    /// common name, which is the same type the roster spells — a trail row and
    /// a client row say one identity one way.
    pub client: String,
}

/// **Where a trail row stands** (REMOTE §9.17): DESIGN §6's outcome folded
/// with §4.2's ack watermark, so a banner is *the rows standing `live`*, and
/// a seat can say why an alarm is down — retirement and the ack are told
/// apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// Ran clean, or was never an attempted action.
    Clean,
    /// Handed off; no exit to observe.
    Detached,
    /// Failed, and nothing has answered it: the alarm.
    Live,
    /// Failed, and a newer clean run of the same verb retired it.
    Retired,
    /// Failed, and the operator's watermark covers it.
    Acked,
}

impl Standing {
    /// The engine's word, which is also the label the trail paints. `pub`
    /// for `queue::held_at`'s reason: a reading the ANDROID paint spends,
    /// which a `pub(crate)` would leave dead on a host build.
    #[must_use]
    pub fn word(self) -> String {
        STANDINGS
            .iter()
            .find(|(_, standing)| *standing == self)
            .map_or_else(String::new, |(word, _)| (*word).to_owned())
    }
}

const STANDINGS: [(&str, Standing); 5] = [
    ("clean", Standing::Clean),
    ("detached", Standing::Detached),
    ("live", Standing::Live),
    ("retired", Standing::Retired),
    ("acked", Standing::Acked),
];

/// Read one trail row, strictly.
pub(crate) fn row(v: &Value) -> Result<OpRow, String> {
    let o = v.as_object().ok_or("ops row: not an object")?;
    Ok(OpRow {
        ts: str_of(o, "ts")?,
        origin: str_of(o, "origin")?,
        argv: str_of(o, "argv")?,
        cwd: str_of(o, "cwd")?,
        exit: i64_of(o, "exit")?,
        stdout: str_of(o, "stdout")?,
        stderr: str_of(o, "stderr")?,
        failed: bool_of(o, "failed")?,
        exit_label: str_of(o, "exit_label")?,
        standing: pick(o, "standing", &STANDINGS)?,
        client: str_of(o, "client")?,
    })
}

#[cfg(test)]
mod tests;
