//! The model's two halves: the handle the frame holds, and the worker loop
//! that owns the wire. A gesture is a command down one channel; the answer
//! is the next snapshot up the other. Every command wakes the worker
//! immediately and every pass through the loop publishes exactly one
//! snapshot, so the cadence bounds staleness, never responsiveness.
//!
//! What one pass IS — the standing questions, the acts, and what survives a
//! failed pass — is `seat::pass`, split out when the grace gave a pass state
//! of its own to carry (bl-3202).

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
    Stop,
}

impl Model {
    /// Start the worker over an opened seat. `cadence` is how long the model
    /// rests between unprompted refreshes — a gesture refreshes immediately.
    pub fn start(seat: Seat, cadence: Duration) -> Self {
        let (cmds, cmd_rx) = mpsc::channel();
        let (snap_tx, snaps) = mpsc::channel();
        let worker = std::thread::spawn(move || run(&seat, cadence, &cmd_rx, &snap_tx));
        Self {
            cmds,
            snaps,
            last: Snapshot::default(),
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

fn run(seat: &Seat, cadence: Duration, cmds: &mpsc::Receiver<Cmd>, out: &mpsc::Sender<Snapshot>) {
    let mut focus = Focus::default();
    let mut note = None;
    let mut standing = Standing::default();
    loop {
        // An undeliverable snapshot is not a stop signal: `Model::drop` sends
        // `Stop` before the receiver can go away (join precedes field drop),
        // so shutdown always arrives as a command, never as a dead channel.
        let _ = out.send(standing.pass(seat, &focus, note.take()));
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
            Ok(Cmd::Deposit(content)) => note = super::pass::deposit(seat, &focus, content).err(),
            Ok(Cmd::Start(goal)) => note = super::pass::started(seat, &focus, goal).err(),
            Ok(Cmd::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => return,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
    }
}
