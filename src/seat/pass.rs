//! **One pass of the model's loop**: the standing questions asked as deep as
//! the focus goes, and what survives a pass the engine did not answer. Split
//! from `model.rs` when the grace gave a pass state to carry between calls
//! (bl-3202) — the handle and the loop are there, what a pass MEANS is here.
//! The two acts the seat POSTS are `seat::acts`: a pass is what the seat
//! asks, and an act is what it says.

use std::path::Path;
use std::sync::mpsc;

use serde_json::Value;

use super::lane::Lanes;
use super::model::Cmd;
use super::options::Options;
use super::{Focus, Snapshot};
use crate::cache::Envelopes;
use crate::codec::reply::Reply;
use crate::codec::{Ask, Gesture, encode};
use crate::transport::Seat;
use fill::fill;

mod adopt;
mod fill;
mod publish;

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
    /// **The answer in flight** (bl-4822), the follow lane's fold (§14.1):
    /// every frame the lane hands over is absorbed onto it, and the stream's
    /// end empties it. It is painted into every published snapshot and never
    /// into `last`: a tail that grows several times a second must not make
    /// the §14 cache rewrite itself as often, and what the cache is for is
    /// the world the engine has written down.
    live: Option<crate::codec::Stream>,
    /// **The two held reads** (§14.1), tended by every pass: the attention
    /// lane always, the follow lane while the focused row states a flight.
    lanes: Lanes,
    /// The attention lane's last frame, verbatim — what the §14 cache stores
    /// for the queue, the way `fill` keeps the pass's own envelopes.
    queue_envelope: Option<Value>,
    /// The last pass's failure sentence, held for the grace below.
    failure: Option<String>,
    /// **The sentence a gesture earned, carried for one pass.** A note is
    /// set by the pass a gesture wakes (or by a lane frame this build could
    /// not read) and stands on every snapshot until the next pass replaces
    /// it — never only on the one snapshot it arrived with, because a lane's
    /// frame can publish the next snapshot a moment later and a sentence
    /// nobody saw is a message that vanished.
    note: Option<String>,
    /// How many times the assignments have been read (bl-e9f9) — the
    /// controls' watermark for "your optimistic value has been overtaken".
    pub(super) reads: usize,
    /// The deposits the engine took, refused, and **left in doubt** (bl-66fb,
    /// widened in bl-07b1). Carried between passes because a deposit is a
    /// gesture, not a pass — and painted into every published snapshot for the
    /// same reason the options are.
    posted: (usize, usize, usize),
    /// **The decision queue, and the trail** (§13.7, §13.8) — the two reads
    /// whose subject is the WORLD rather than a depth of the focus, so a pass
    /// that narrows the focus loses neither. The queue is written by the
    /// attention lane's frames and by nothing else (§14.1); the trail only
    /// ever by the trail screen's gesture, because nothing paints it
    /// standing.
    pub(super) queue: Vec<crate::codec::QueueRow>,
    pub(super) trail: Vec<crate::codec::OpRow>,
    /// **The last needle's answer** (bl-4c2b), carried between passes for the
    /// reason the counters above are: it is a gesture's answer, not a pass's,
    /// and a pass that re-reads the world must not drop the search the
    /// operator is looking at. Folded in by `worker::searched`, where the
    /// other gesture-made read is folded too.
    pub(super) found: Option<crate::codec::Found>,
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
    pub(super) fn resumed(snap: Snapshot, options: Options, queue_envelope: Option<Value>) -> Self {
        Self {
            options,
            live: None,
            lanes: Lanes::default(),
            queue_envelope,
            failure: None,
            note: None,
            reads: 0,
            posted: (0, 0, 0),
            // The cached queue is an answer the engine gave, so it is held as
            // one: the band paints from the file on the way to the first pass
            // exactly as the rows beside it do (§14).
            queue: snap.queue.clone(),
            trail: Vec::new(),
            found: None,
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
    /// - **The sentence waits, and `note` never does** — `Self::publish`,
    ///   which every snapshot goes out through.
    pub(super) fn pass(
        &mut self,
        seat: &Seat,
        cache: &Path,
        focus: &Focus,
        note: Option<String>,
        tx: &mpsc::Sender<Cmd>,
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
        let (providers, models, roles) = self.options.envelopes();
        kept.providers = providers;
        kept.models = models;
        kept.roles = roles;
        kept.options_workspace = self.options.workspace();
        kept.attention.clone_from(&self.queue_envelope);
        // The queue rides the comparison below because it is an answer the
        // engine gave and the file holds it (§14); it is written by the lane
        // rather than by this pass, which changes nothing about that.
        fresh.queue.clone_from(&self.queue);
        if failed.is_none() {
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
        } else if self.last.focus != *focus {
            self.last = fresh;
        }
        // **The lanes are tended after the rows** (§14.1): what should stand
        // is read off the pass's own answer — the focused row's flight — and
        // a lane is dialled only by a pass the engine answered, so an engine
        // that is down costs one failed dial a pass and not two. The drop
        // half runs regardless: a lane on a subject the focus has left must
        // not go on writing its frames under the new one.
        let wanted = self.wanted(focus);
        self.lanes.tend(seat, &wanted, tx, failed.is_none());
        if failed.is_none() {
            self.failed = 0;
        } else {
            self.failed += 1;
        }
        self.failure = failed;
        // A turn that has finished has no tail: the answer arrives as a
        // transcript row, and a fold left standing under it would be the
        // same words twice (bl-4822).
        if !self.streaming(focus) {
            self.live = None;
        }
        self.note = note;
        self.publish(focus)
    }
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
