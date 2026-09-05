//! **The handle the frame holds**: the commands it can send and the snapshot
//! it reads back. A gesture is a command down one channel; the answer is the
//! next snapshot up the other, and every command wakes the worker
//! immediately — so the cadence bounds staleness, never responsiveness.
//!
//! The loop that spends them is `seat::worker`, split out when the tuning
//! pair's two commands took this file to the 300 wall (bl-dfbb); what one
//! PASS is — the standing questions and what survives a failed one — is
//! `seat::pass` (bl-3202), and the acts it posts are `seat::acts`.

use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use super::Snapshot;
use crate::transport::Seat;

/// The frame's handle. Dropping it stops the worker and joins it.
pub struct Model {
    cmds: mpsc::Sender<Cmd>,
    snaps: mpsc::Receiver<Snapshot>,
    last: Snapshot,
    worker: Option<std::thread::JoinHandle<()>>,
}

pub(super) enum Cmd {
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
    /// Set the worker's reasoning level, or remove it (`None` is off).
    Effort(Option<crate::codec::Effort>),
    /// Ask the worker's provider for its priority lane, or stop asking.
    Priority(bool),
    /// **Search the world, or drop the answer being shown** (bl-4c2b). The
    /// empty needle is the second: it crosses no wire, so a search can be
    /// left with the engine unreachable.
    Search(String),
    /// Stop the focused conversation's turn, optionally its subtree with it.
    StopTurn(bool),
    /// Re-prompt the focused conversation from where it stands.
    Nudge,
    /// **One act on a NAMED conversation** (§13.5): the agent the row's menu
    /// was opened on, and which of the three it fired. One command for the
    /// group because they are one gesture — the roster has one home, and it
    /// is `codec::RowAct`.
    Row(String, crate::codec::RowAct),
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
        let worker = std::thread::spawn(move || {
            super::worker::run(&seat, cadence, &cache, kept, &cmd_rx, &snap_tx);
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

    /// **Set the worker's reasoning level** (REMOTE §9.4, bl-dfbb) — how
    /// much reasoning its model calls request. `None` is off, which is the
    /// absence of a level rather than a fourth one. It takes at the next
    /// step, so it is a mid-conversation act like the model pick.
    pub fn set_effort(&self, level: Option<crate::codec::Effort>) {
        let _ = self.cmds.send(Cmd::Effort(level));
    }

    /// **Ask the worker's provider for its priority lane**, or stop asking.
    pub fn set_priority(&self, on: bool) {
        let _ = self.cmds.send(Cmd::Priority(on));
    }

    /// **Stop the focused conversation's in-flight turn** (bl-48fa), and its
    /// subtree with it when `children`. It is the wire's `stop` op — this
    /// seat never deposits a slash line for it, because a deposit is content
    /// and content starts the driver it was meant to stop.
    pub fn stop_turn(&self, children: bool) {
        let _ = self.cmds.send(Cmd::StopTurn(children));
    }

    /// **Nudge the focused conversation** (§8.2, bl-d09e) — the act for a
    /// branch that stopped advancing. Idempotent it is not: two taps are two
    /// nudges, which is why the control is offered only while the
    /// conversation is at rest.
    pub fn nudge(&self) {
        let _ = self.cmds.send(Cmd::Nudge);
    }

    /// **Fire one of the conversation row's acts** (§13.5, bl-f97c) at the
    /// conversation the menu was opened on — never at the focus, which is why
    /// the agent is carried rather than read from it: a long-press names its
    /// own subject, and the operator need not have opened it first.
    ///
    /// Not idempotent, any of the three, so nothing here is ever sent twice:
    /// a lost reply becomes the banner's sentence and the read that settles
    /// it (`seat::acts::row`).
    pub fn row_act(&self, agent: String, act: crate::codec::RowAct) {
        let _ = self.cmds.send(Cmd::Row(agent, act));
    }

    /// **Search everything this seat can see** (yog DESIGN §8.5) for `text`,
    /// or — with an empty needle — drop the answer that is standing. The hits
    /// arrive in the next snapshot like every other read's rows.
    pub fn search(&self, text: String) {
        let _ = self.cmds.send(Cmd::Search(text));
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
