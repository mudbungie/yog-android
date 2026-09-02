//! The model's two halves: the handle the frame holds, and the worker loop
//! that owns the wire. A gesture is a command down one channel; the answer
//! is the next snapshot up the other. Every command wakes the worker
//! immediately and every pass through the loop publishes exactly one
//! snapshot, so the cadence bounds staleness, never responsiveness.
//!
//! What one pass IS — the standing questions, the acts, and what survives a
//! failed pass — is `seat::pass`, split out when the grace gave a pass state
//! of its own to carry (bl-3202).

use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use super::pass::Standing;
use super::{Focus, Snapshot};
use crate::transport::Seat;

/// The frame's handle. Dropping it stops the worker and joins it.
pub struct Model {
    cmds: mpsc::Sender<Cmd>,
    snaps: mpsc::Receiver<Snapshot>,
    last: Snapshot,
    worker: Option<std::thread::JoinHandle<()>>,
}

enum Cmd {
    Workspace(Option<String>),
    Conversation(String, String),
    Deposit(String),
    Start(String),
    /// **List the focused workspace's providers** (bl-0267) — a gesture of
    /// the selectors' own, asked when one is opened rather than on every
    /// pass: a pass is the standing set, and these are options.
    Providers,
    /// List one provider's models.
    Models(String),
    /// Assign the worker role's provider and model, stated whole.
    Pick(String, String),
    /// Stop the focused conversation's turn, optionally its subtree with it.
    StopTurn(bool),
    Stop,
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
            let options = super::options::from_cache(stored);
            options.paint(&focus, &mut snap);
            (focus, snap, options)
        });
        let last = kept.clone().map(|(_, snap, _)| snap).unwrap_or_default();
        let (cmds, cmd_rx) = mpsc::channel();
        let (snap_tx, snaps) = mpsc::channel();
        let worker =
            std::thread::spawn(move || run(&seat, cadence, &cache, kept, &cmd_rx, &snap_tx));
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

    /// Focus a workspace (its conversation list joins the standing set), or
    /// `None` to back out to the workspace roster.
    pub fn focus_workspace(&self, workspace: Option<String>) {
        let _ = self.cmds.send(Cmd::Workspace(workspace));
    }

    /// Focus one conversation: its transcript joins the standing set.
    pub fn focus_conversation(&self, workspace: String, agent: String) {
        let _ = self.cmds.send(Cmd::Conversation(workspace, agent));
    }

    /// Post the composer's text to the focused conversation. The receipt —
    /// or the refusal — arrives with the next snapshot.
    pub fn deposit(&self, content: String) {
        let _ = self.cmds.send(Cmd::Deposit(content));
    }

    /// **Ask for the focused workspace's providers** (bl-0267). The answer
    /// arrives in the next snapshot, and what was already known keeps
    /// painting meanwhile — the selectors open on the cache and correct
    /// themselves a round trip later.
    pub fn list_providers(&self) {
        let _ = self.cmds.send(Cmd::Providers);
    }

    /// Ask for one provider's models.
    pub fn list_models(&self, provider: String) {
        let _ = self.cmds.send(Cmd::Models(provider));
    }

    /// **Assign the worker role's model** in the focused workspace. One act,
    /// no apply step (§13.2): the tap is the gesture, and the engine's
    /// refusal — if it refuses — arrives in the banner.
    pub fn pick_model(&self, provider: String, model: String) {
        let _ = self.cmds.send(Cmd::Pick(provider, model));
    }

    /// **Stop the focused conversation's in-flight turn** (bl-48fa), and its
    /// subtree with it when `children`. It is the wire's `stop` op — this
    /// seat never deposits a slash line for it, because a deposit is content
    /// and content starts the driver it was meant to stop.
    pub fn stop_turn(&self, children: bool) {
        let _ = self.cmds.send(Cmd::StopTurn(children));
    }

    /// Start a new conversation in the focused workspace with `goal` as its
    /// first instruction. The staging and the firing are one gesture from
    /// here because they are one act to the operator; the engine's two-step
    /// is the wire's business, not the composer's.
    pub fn start_conversation(&self, goal: String) {
        let _ = self.cmds.send(Cmd::Start(goal));
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

fn run(
    seat: &Seat,
    cadence: Duration,
    cache: &std::path::Path,
    kept: Option<(Focus, Snapshot, super::options::Options)>,
    cmds: &mpsc::Receiver<Cmd>,
    out: &mpsc::Sender<Snapshot>,
) {
    let (mut focus, mut standing) = match kept {
        // The selectors' offerings come back with the rows (bl-0267): the
        // file holds them under the workspace they were read for, so a
        // resumed seat opens its selectors instantly and offline.
        Some((focus, snap, options)) => (focus, Standing::resumed(snap, options)),
        None => (Focus::default(), Standing::default()),
    };
    let mut note = None;
    loop {
        // An undeliverable snapshot is not a stop signal: `Model::drop` sends
        // `Stop` before the receiver can go away (join precedes field drop),
        // so shutdown always arrives as a command, never as a dead channel.
        let _ = out.send(standing.pass(seat, cache, &focus, note.take()));
        match cmds.recv_timeout(cadence) {
            Ok(Cmd::Workspace(workspace)) => {
                focus = Focus {
                    workspace,
                    agent: None,
                }
            }
            Ok(Cmd::Conversation(workspace, agent)) => {
                focus = Focus {
                    workspace: Some(workspace),
                    agent: Some(agent),
                };
            }
            Ok(Cmd::Deposit(content)) => note = super::acts::deposit(seat, &focus, content).err(),
            // The three selector gestures. A read's answer is learned as the
            // engine's own envelope (bl-0267); a failure is a sentence for
            // the banner exactly as an act's is.
            Ok(Cmd::Providers) => {
                note = learned(super::acts::providers(seat, &focus), None, &mut standing);
            }
            Ok(Cmd::Models(provider)) => {
                let listed = super::acts::models(seat, &focus, &provider);
                note = learned(listed, Some(provider), &mut standing);
            }
            Ok(Cmd::StopTurn(children)) => {
                note = super::acts::stop(seat, &focus, children).err();
            }
            Ok(Cmd::Pick(provider, model)) => {
                note = super::acts::pick(seat, &focus, &provider, &model).err();
            }
            Ok(Cmd::Start(goal)) => note = super::acts::started(seat, &focus, goal).err(),
            Ok(Cmd::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => return,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
    }
}

/// Fold one selector read into the standing options, or hand back the
/// sentence it failed with. One body for both reads, because the only
/// difference between them is which slot the envelope lands in.
fn learned(
    read: Result<(String, serde_json::Value), String>,
    provider: Option<String>,
    standing: &mut Standing,
) -> Option<String> {
    match read {
        Ok((workspace, envelope)) => {
            standing
                .options
                .learned(&workspace, provider.as_deref(), envelope);
            None
        }
        Err(why) => Some(why),
    }
}
