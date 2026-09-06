//! **The handle the frame holds**: the commands it can send and the snapshot
//! it reads back. A gesture is a command down one channel; the answer is the
//! next snapshot up the other, and every command wakes the worker
//! immediately — so the cadence bounds staleness, never responsiveness.
//!
//! **The handles themselves are two files beside this one** (bl-5a56), on the
//! grammar's own seam: `model/reads.rs` is what the frame ASKS for and
//! `model/acts.rs` is what it SAYS — the split `codec::encode`,
//! `codec::request`, `seat::asks` and `seat::acts` already make. What is left
//! here is the handle: what it holds, how it starts, and what it hands back.
//!
//! The loop that spends them is `seat::worker`, split out when the tuning
//! pair's two commands took this file to the 300 wall (bl-dfbb); the
//! vocabulary they are sent in is `seat::cmd`, split out when the ball pane's
//! act took it there again (bl-f36e); what one PASS is — the standing
//! questions and what survives a failed one — is `seat::pass` (bl-3202), and
//! the acts it posts are `seat::acts`.

use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use super::Snapshot;
use super::cmd::Cmd;
use crate::transport::Seat;

mod acts;
mod reads;

/// The frame's handle. Dropping it stops the worker and joins it.
pub struct Model {
    cmds: mpsc::Sender<Cmd>,
    snaps: mpsc::Receiver<Snapshot>,
    last: Snapshot,
    worker: Option<std::thread::JoinHandle<()>>,
}

impl Model {
    /// Start the worker over an opened seat. `cadence` is how long the model
    /// rests between unprompted refreshes — a gesture refreshes immediately.
    ///
    /// **`cache` is where the last answered pass is kept** (bl-de96), and it
    /// is read HERE, synchronously, before the first frame: the handle starts
    /// holding what the engine last said instead of an empty snapshot, so the
    /// app paints its world immediately and the first cadence read replaces
    /// it. The worker is seeded with the same value — both its rows, so a
    /// first pass that fails republishes them rather than blanking the
    /// screen (§13.2's grace), and its FOCUS, because rows are only paintable
    /// under the focus they were asked at and the operator is put back where
    /// they were by the same fact.
    pub fn start(seat: Seat, cadence: Duration, cache: PathBuf) -> Self {
        // The cached pass, with its selector offerings painted into it: the
        // handle's first `snapshot()` is read before any pass has run, so a
        // resumed seat's selectors are open on the way to the first frame
        // (bl-0267) and not one round trip later.
        let kept = crate::cache::read(&cache).map(|(focus, mut snap, stored)| {
            let queue = stored.attention.clone();
            let options = super::options::from_cache(stored);
            options.paint(&focus, &mut snap);
            (focus, snap, options, queue)
        });
        let last = kept.clone().map(|(_, snap, _, _)| snap).unwrap_or_default();
        let (cmds, cmd_rx) = mpsc::channel();
        let (snap_tx, snaps) = mpsc::channel();
        // The worker holds a sender of its own for the lanes to hand frames
        // down (§14.1), so the channel never reads as disconnected while it
        // runs: `Stop` is the one way out, and `drop` sends it.
        let lanes = cmds.clone();
        let worker = std::thread::spawn(move || {
            super::worker::run(&seat, cadence, &cache, kept, &cmd_rx, &lanes, &snap_tx);
        });
        Self {
            cmds,
            snaps,
            last,
            worker: Some(worker),
        }
    }

    /// The latest published snapshot. Non-blocking: drains whatever the
    /// worker has produced and hands back the newest, or the previous answer
    /// when nothing new arrived — the frame paints at its own cadence.
    pub fn snapshot(&mut self) -> Snapshot {
        while let Ok(snap) = self.snaps.try_recv() {
            self.last = snap;
        }
        self.last.clone()
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        let _ = self.cmds.send(Cmd::Stop);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
