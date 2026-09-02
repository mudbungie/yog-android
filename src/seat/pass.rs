//! **One pass of the model's loop**: the standing questions asked as deep as
//! the focus goes, and what survives a pass the engine did not answer. Split
//! from `model.rs` when the grace gave a pass state to carry between calls
//! (bl-3202) — the handle and the loop are there, what a pass MEANS is here.
//! The two acts the seat POSTS are `seat::acts`: a pass is what the seat
//! asks, and an act is what it says.

use std::path::Path;

use serde_json::Value;

use super::options::Options;
use super::{Focus, Snapshot};
use crate::cache::Envelopes;
use crate::codec::reply::Reply;
use crate::codec::{Ask, Gesture, encode};
use crate::transport::Seat;

/// **How many consecutive failed passes an error waits for.** The cadence is
/// the clock (bl-3202): passes are one rest apart, so a second consecutive
/// failure is exactly *"it did not clear within one rest"* — no timestamp to
/// keep, none to inject, and one clock rather than two.
const GRACE: u32 = 1;

/// What the worker carries between passes: the last answer the engine
/// actually gave, and how many passes have failed since one did.
#[derive(Default)]
pub(super) struct Standing {
    /// What the composer's selectors offer, carried between passes because a
    /// pass does not read it — the selectors are their own gestures
    /// (bl-0267).
    pub(super) options: Options,
    /// **The answer in flight** (bl-4822), read on its own quicker rest. It
    /// is painted into every published snapshot and never into `last`: a
    /// tail that changes five times a second must not make the §14 cache
    /// rewrite itself five times a second, and what the cache is for is the
    /// world the engine has written down.
    live: Option<crate::codec::Stream>,
    /// The deposits the engine took and refused (bl-66fb). Carried between
    /// passes because a deposit is a gesture, not a pass — and painted into
    /// every published snapshot for the same reason the options are.
    posted: (usize, usize),
    last: Snapshot,
    failed: u32,
    /// What was last WRITTEN to the cache, so a pass that changed nothing
    /// writes nothing (bl-de96). Comparing the snapshot rather than the
    /// envelopes because the snapshot is what the operator would see change.
    stored: Snapshot,
}

impl Standing {
    /// Standing seeded from the cache: the rows the engine last gave, held as
    /// last-good so a first pass that fails republishes them rather than
    /// blanking the screen. `stored` is seeded with them too — they ARE what
    /// the file holds, so an unchanged pass rewrites nothing.
    pub(super) fn resumed(snap: Snapshot, options: Options) -> Self {
        Self {
            options,
            live: None,
            posted: (0, 0),
            last: snap.clone(),
            failed: 0,
            stored: snap,
        }
    }
    /// One refresh pass, and the snapshot the frame should paint for it.
    ///
    /// **A failure is not an error until it persists** (bl-3202). Swapping
    /// back into the app raced the network coming back: the first pass after
    /// a resume failed on a name lookup, and the frame painted a red banner
    /// over three emptied lists for a second. Both halves of that were the
    /// pass throwing away what it already had, so both are fixed here rather
    /// than in the paint — one clock, and a frame that only ever renders what
    /// it is handed.
    ///
    /// - **The rows survive.** A failed pass republishes the last answer the
    ///   engine gave, *under the focus it was asked at*: pairing one focus's
    ///   rows with another's is the one thing [`Snapshot`] promises never to
    ///   do, so a focus that moved gets the empty lists it honestly has.
    /// - **The sentence waits.** A refresh failure paints once it has
    ///   persisted past [`GRACE`]; a pass that succeeds clears it instantly,
    ///   because a standing success is never in doubt.
    /// - **`note` never waits.** It is a gesture's own answer — a refused
    ///   deposit, a start the engine would not run — and the operator just
    ///   acted. Silence there is a message that vanished.
    pub(super) fn pass(
        &mut self,
        seat: &Seat,
        cache: &Path,
        focus: &Focus,
        note: Option<String>,
    ) -> Snapshot {
        let mut fresh = Snapshot {
            focus: focus.clone(),
            ..Snapshot::default()
        };
        let mut kept = Envelopes::default();
        let failed = fill(seat, focus, &mut fresh, &mut kept).err();
        // The selectors' offerings ride every snapshot, under the focus they
        // were read for and no other (bl-0267).
        self.options.paint(focus, &mut fresh);
        let (providers, models) = self.options.envelopes();
        kept.providers = providers;
        kept.models = models;
        kept.options_workspace = self.options.workspace();
        if failed.is_none() {
            self.failed = 0;
            self.last = fresh;
            // The cache is written from a pass the engine ANSWERED, and only
            // when what it says changed: a live conversation changes every
            // pass and a quiet one never does, and the write is proportionate
            // either way — this app already paid to receive the same bytes
            // over TLS on the pass that produced them (bl-de96).
            if self.last != self.stored {
                // A cache that cannot be written is a cache miss next boot
                // and nothing else — never the banner, which is for what the
                // engine said, and never a stop. (`log` is the shell's
                // dependency, not the core's, so there is nowhere here to
                // say it either.)
                let _ = crate::cache::write(cache, focus, &kept);
                self.stored = self.last.clone();
            }
        } else {
            self.failed += 1;
            if self.last.focus != *focus {
                self.last = fresh;
            }
        }
        // A turn that has finished has no tail: the answer arrives as a
        // transcript row, and a fold left standing under it would be the
        // same words twice (bl-4822).
        if !self.streaming(focus) {
            self.live = None;
        }
        let mut out = self.last.clone();
        out.live.clone_from(&self.live);
        (out.landed, out.refused) = self.posted;
        // Painted onto the published snapshot as well as onto `fresh`: a
        // pass that failed republishes last-good rows, and the selectors'
        // offerings are not the pass's to lose (bl-0267).
        self.options.paint(focus, &mut out);
        out.error = match (note, failed.filter(|_| self.failed > GRACE)) {
            (Some(note), Some(failed)) => Some(format!("{note}; {failed}")),
            (note, failed) => note.or(failed),
        };
        out
    }
}

/// The standing questions, as deep as the focus goes. The first failure
/// stops the walk: an unreachable engine is one sentence, not three.
fn fill(
    seat: &Seat,
    focus: &Focus,
    snap: &mut Snapshot,
    kept: &mut Envelopes,
) -> Result<(), String> {
    let (reply, envelope) = answer(seat, &Ask::Workspaces)?;
    snap.workspaces = match reply {
        Reply::Workspaces { rows, .. } => rows,
        other => return Err(kind_err("workspaces", &other)),
    };
    kept.workspaces = Some(envelope);
    let Some(workspace) = focus.workspace.clone() else {
        return Ok(());
    };
    let ask = Ask::Conversations {
        workspace: workspace.clone(),
    };
    let (reply, envelope) = answer(seat, &ask)?;
    snap.conversations = match reply {
        Reply::Conversations(rows) => rows,
        other => return Err(kind_err("conversations", &other)),
    };
    kept.conversations = Some(envelope);
    let Some(agent) = focus.agent.clone() else {
        return Ok(());
    };
    let (reply, envelope) = answer(seat, &Ask::Transcript { workspace, agent })?;
    snap.transcript = match reply {
        Reply::Transcript(rows) => rows,
        other => return Err(kind_err("transcript", &other)),
    };
    kept.transcript = Some(envelope);
    Ok(())
}

/// One standing question, and **the engine's own envelope beside the rows it
/// decoded to** (bl-de96). The raw value is what the cache stores, so the
/// file holds the wire's spelling rather than a second one this client would
/// have to keep in step — see `crate::cache`.
pub(super) fn answer(seat: &Seat, ask: &Ask) -> Result<(Reply, Value), String> {
    // The transport's two classes collapse to the sentence here, and rightly:
    // this model opens a connection per ask, so a broken channel is already
    // re-dialled by the next pass and there is nothing for it to decide
    // (bl-8641). The tool host, which holds one channel, is the caller that
    // reads the class.
    let stream = seat.ask(&encode(&Gesture::Ask(ask.clone())))?;
    let last = stream
        .last()
        .ok_or("the engine ended the stream without answering")?;
    let reply = crate::codec::reply::decode(last).unwrap_or_else(Err)?;
    Ok((reply, last.clone()))
}

/// The wrong-kind sentence names the kind, never the rows it carried. Shared
/// with `seat::acts`, which asks the same question of a receipt.
pub(super) fn kind_err(asked: &str, got: &Reply) -> String {
    format!("{asked}: the engine answered {} instead", got.kind())
}

impl Standing {
    /// **One deposit's fate, counted** (bl-66fb). The composer's echo cannot
    /// see the receipt — the worker holds the wire — so what it watches is
    /// this pair moving.
    pub(super) fn posted(&mut self, took: bool) {
        if took {
            self.posted.0 += 1;
        } else {
            self.posted.1 += 1;
        }
    }

    /// **Whether the focused conversation is writing right now** — read off
    /// the row's own `flight`, which is where every conversation-level gate
    /// rides (REMOTE §9.4). A conversation the list has not caught up with
    /// has no row and so is not streaming, which is the honest answer.
    pub(super) fn streaming(&self, focus: &Focus) -> bool {
        let Some(agent) = focus.agent.as_deref() else {
            return false;
        };
        self.last
            .conversations
            .iter()
            .find(|row| row.root_id == agent)
            .is_some_and(|row| row.flight.is_some())
    }

    /// One live read, and the snapshot to publish for it. The tail replaces
    /// whatever was held (§5.5: every read of this seat's is a first frame),
    /// and a failure is a sentence for the banner like any other — it does
    /// not stop the lane, because the next tick re-asks and is whole.
    pub(super) fn living(&mut self, seat: &Seat, focus: &Focus) -> Snapshot {
        let read = super::acts::follow(seat, focus);
        let mut out = self.last.clone();
        (out.landed, out.refused) = self.posted;
        match read {
            Ok(stream) => {
                self.live = Some(stream);
                out.live.clone_from(&self.live);
            }
            Err(why) => out.error = Some(why),
        }
        self.options.paint(focus, &mut out);
        out
    }
}
