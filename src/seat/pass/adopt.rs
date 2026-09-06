//! **What a lane's frame means to the standing** (DESIGN §14.1): the fold
//! each of the two held reads is adopted onto, and the end of a stream.
//! Split from `pass.rs` on the seam its doc draws — what a PASS means is
//! there; what a FRAME means is here — because the two change for unrelated
//! reasons and neither should read the other's.
//!
//! **Two folds, one per lane, and neither is a special case of the other.**
//! The attention frame REPLACES (REMOTE §14.1: *"frames replace; they never
//! append"* — a handful of rows that grows with nothing), and the follow
//! frame APPENDS (§5.5: *"absorb every frame of a read, in order, onto an
//! empty fold"*). The wire says which by the reply kind, and this seat says
//! it by the lane's subject; the two must agree, and a frame of the wrong
//! kind is the sentence every wrong-kind answer earns.

use serde_json::Value;

use super::{Standing, kind_err};
use crate::codec::reply::{self, Reply};
use crate::seat::lane::{Framed, Subject};

impl Standing {
    /// **Adopt what a lane handed over.** What went wrong, if anything, is
    /// the note the next snapshots carry (`Standing::note`). A frame from a
    /// lane no longer held — hung up because the focus moved, or because it
    /// already ended — is nothing: the id says so, and the subject is not
    /// consulted, because the new lane on the same subject must not absorb
    /// the old stream's deltas.
    pub(in crate::seat) fn adopted(&mut self, framed: Framed) {
        match framed {
            Framed::Frame(id, frame) => {
                if let Some(subject) = self.lanes.subject(id)
                    && let Some(why) = self.adopt(id, &subject, &frame)
                {
                    self.note = Some(why);
                }
            }
            Framed::Over(id) => {
                // The stream is over: what it folded is committed, and the
                // transcript is the authority from here (REMOTE §5.5 — the
                // seat swaps to the committed entry with nothing to
                // reconcile). The next pass reopens the lane if the row still
                // flies, and its first frame is whole.
                if let Some(Subject::Follow { .. }) = self.lanes.ended(id) {
                    self.live = None;
                }
            }
        }
    }

    fn adopt(&mut self, id: u64, subject: &Subject, frame: &Value) -> Option<String> {
        let reply = match reply::decode(frame) {
            Err(unreadable) | Ok(Err(unreadable)) => return Some(unreadable),
            Ok(Ok(reply)) => reply,
        };
        match (subject, reply) {
            (Subject::Attention, Reply::Attention(rows)) => {
                self.queue = rows;
                self.queue_envelope = Some(frame.clone());
                None
            }
            (Subject::Follow { .. }, Reply::Follow(later)) => {
                self.live.get_or_insert_default().absorb(later);
                None
            }
            // **The sign-in's frames fold by the LANE they came from**
            // (§13.19): a redialled lane replays from zero, so the id is what
            // tells a replay from a delta and the subject cannot.
            (Subject::Login { .. }, Reply::Login(view)) => {
                self.signing.framed(id, view);
                None
            }
            (subject, other) => Some(kind_err(subject.name(), &other)),
        }
    }
}
