//! **The ops trail** (yog DESIGN §4.2, §7.3; REMOTE §9.17) — the last actions
//! this engine took, and the two acts an operator has over them.
//!
//! The trail is the world's durable record of what ran: every subprocess the
//! engine forked, with the directory it ran in, what it said and what it
//! exited. It is the read every other recovery sentence in this client points
//! at — *"the world is the durable record"* (REMOTE §9.8) — and until this
//! surface it was the one read a phone could not make.
//!
//! **This decoder reads the row's own facts and classifies nothing.** yog
//! answers a failed action four ways it derives itself (the sentinel table,
//! the `128 + n` signal reading, the retirement key, the ack watermark), and
//! REMOTE §9.17 put all four on the wire as `failed`, `exit_label` and
//! `standing` so that no seat re-implements them — *"a seat that wanted the
//! banner had to re-implement … five derivations this document names one home
//! for apiece, and whose failure mode is a seat quietly disagreeing"*. The
//! corpus this build is vendored against predates that bump, so those three
//! fields are not here to read yet; the answer is to paint what the row says
//! and derive nothing, never to re-derive them from `exit`. Reading them is
//! bl-8e3c's, with the re-vendor that brings them.
//!
//! **`ack` and `clear-trail` carry nothing, and that is the whole shape.**
//! Neither names a row: the ack is a watermark over the trail as it stands and
//! the clear is a truncation of it, both world-wide, so there is nothing for a
//! client to select and nothing for it to get wrong.

use serde_json::Value;

use super::fields::{i64_of, str_of};

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
}

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
    })
}

#[cfg(test)]
mod tests;
