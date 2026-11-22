//! **The handle the frame holds**: the commands it can send and the snapshot
//! it reads back. A gesture is a command down one channel; the answer is the
//! next snapshot up the other, and every command wakes the worker
//! immediately — so the cadence bounds staleness, never responsiveness.
//!
//! **Two channels down, since bl-2cad**: what the frame ASKS goes to the
//! worker, and what it SAYS goes to the poster (`seat::poster`), a second
//! thread with its own connections — so a send leaves at once, whatever read
//! the worker is inside. A focus gesture goes down both.
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
use std::sync::{Arc, mpsc};
use std::time::Duration;

use super::Snapshot;
use super::cmd::Cmd;
use super::poster::Post;
use crate::transport::Seat;

mod acts;
mod reads;

/// The frame's handle. Dropping it stops both threads and joins them.
pub struct Model {
    cmds: mpsc::Sender<Cmd>,
    /// **The poster's channel** (bl-2cad): every act goes down this one, and
    /// every focus gesture goes down both.
    posts: mpsc::Sender<Post>,
    snaps: mpsc::Receiver<Snapshot>,
    last: Snapshot,
    worker: Option<std::thread::JoinHandle<()>>,
    poster: Option<std::thread::JoinHandle<()>>,
}

impl Model {
    /// Start the worker over an opened seat. `cadence` is how long the model
    /// rests between unprompted refreshes — a gesture refreshes immediately.
    ///
    /// **`cache` is where the last answered pass is kept** (bl-de96), and it
    /// is read HERE, synchronously, before the first frame: the handle starts
    /// holding what the server last said instead of an empty snapshot, so the
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
        let (posts, post_rx) = mpsc::channel();
        let (snap_tx, snaps) = mpsc::channel();
        // **Two threads over one seat** (bl-2cad): the worker makes every
        // read, the poster posts every act, and each gesture is a connection
        // of its own (REMOTE §3) — so neither waits for the other. The poster
        // starts where the worker does: at the focus the cache resumed.
        let seat = Arc::new(seat);
        let aim = kept
            .as_ref()
            .map(|(focus, ..)| focus.clone())
            .unwrap_or_default();
        let (hand, back) = (Arc::clone(&seat), cmds.clone());
        let poster = std::thread::spawn(move || {
            super::poster::run(&hand, aim, &post_rx, &back);
        });
        // The worker holds a sender of its own for the lanes to hand frames
        // down (§14.1), so the channel never reads as disconnected while it
        // runs: `Stop` is the one way out, and `drop` sends it.
        let down = super::worker::Down {
            lanes: cmds.clone(),
            poster: posts.clone(),
        };
        let worker = std::thread::spawn(move || {
            super::worker::run(&seat, cadence, &cache, kept, &cmd_rx, &down, &snap_tx);
        });
        Self {
            cmds,
            posts,
            snaps,
            last,
            worker: Some(worker),
            poster: Some(poster),
        }
    }

    /// **Aim the focus, at both threads** — the poster first, so no act sent
    /// after this can reach it ahead of the move (`seat::poster`).
    fn aim(&self, focus: super::Focus) {
        let _ = self.posts.send(Post::Aim(focus.clone()));
        let _ = self.cmds.send(Cmd::Aim(focus));
    }

    /// Say one act: straight to the poster, which posts it at once.
    fn say(&self, say: super::say::Say) {
        let _ = self.posts.send(Post::Say(say));
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
    /// **The poster first, then the worker.** Every act already said is
    /// posted before the poster returns, and its receipt is in the worker's
    /// queue ahead of the `Stop` sent after — so nothing the operator said is
    /// dropped on the way out, and nothing it earned goes unfolded.
    fn drop(&mut self) {
        let _ = self.posts.send(Post::Stop);
        if let Some(poster) = self.poster.take() {
            let _ = poster.join();
        }
        let _ = self.cmds.send(Cmd::Stop);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
