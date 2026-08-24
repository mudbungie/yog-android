//! The model's two halves: the handle the frame holds, and the worker loop
//! that owns the wire. A gesture is a command down one channel; the answer
//! is the next snapshot up the other. Every command wakes the worker
//! immediately and every pass through the loop publishes exactly one
//! snapshot, so the cadence bounds staleness, never responsiveness.

use std::sync::mpsc;
use std::time::Duration;

use super::{Focus, Snapshot};
use crate::codec::reply::Reply;
use crate::codec::{Act, Ask, Gesture, encode};
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
    loop {
        // An undeliverable snapshot is not a stop signal: `Model::drop` sends
        // `Stop` before the receiver can go away (join precedes field drop),
        // so shutdown always arrives as a command, never as a dead channel.
        let _ = out.send(refresh(seat, &focus, note.take()));
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
            Ok(Cmd::Deposit(content)) => note = deposit(seat, &focus, content).err(),
            Ok(Cmd::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => return,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
    }
}

/// One refresh pass. `note` is a failure carried in from a deposit; a
/// refresh failure joins it rather than replacing it — both belong to the
/// banner this snapshot earns.
fn refresh(seat: &Seat, focus: &Focus, note: Option<String>) -> Snapshot {
    let mut snap = Snapshot {
        focus: focus.clone(),
        ..Snapshot::default()
    };
    let failed = fill(seat, focus, &mut snap).err();
    snap.error = match (note, failed) {
        (Some(note), Some(failed)) => Some(format!("{note}; {failed}")),
        (note, failed) => note.or(failed),
    };
    snap
}

/// The standing questions, as deep as the focus goes. The first failure
/// stops the walk: an unreachable engine is one sentence, not three.
fn fill(seat: &Seat, focus: &Focus, snap: &mut Snapshot) -> Result<(), String> {
    snap.workspaces = match answer(seat, &Ask::Workspaces)? {
        Reply::Workspaces { rows, .. } => rows,
        other => return Err(kind_err("workspaces", &other)),
    };
    let Some(workspace) = focus.workspace.clone() else {
        return Ok(());
    };
    let ask = Ask::Conversations {
        workspace: workspace.clone(),
    };
    snap.conversations = match answer(seat, &ask)? {
        Reply::Conversations(rows) => rows,
        other => return Err(kind_err("conversations", &other)),
    };
    let Some(agent) = focus.agent.clone() else {
        return Ok(());
    };
    snap.transcript = match answer(seat, &Ask::Transcript { workspace, agent })? {
        Reply::Transcript(rows) => rows,
        other => return Err(kind_err("transcript", &other)),
    };
    Ok(())
}

fn answer(seat: &Seat, ask: &Ask) -> Result<Reply, String> {
    seat.answered(&encode(&Gesture::Ask(ask.clone())))
}

/// Post one message. The receipt is an `outcome` whose `ok` is the server's
/// own verdict; anything else is a sentence for the banner.
fn deposit(seat: &Seat, focus: &Focus, content: String) -> Result<(), String> {
    let Focus {
        workspace: Some(workspace),
        agent: Some(agent),
    } = focus.clone()
    else {
        return Err("deposit: no conversation is focused".to_owned());
    };
    let act = Act::Message {
        workspace,
        agent,
        content,
    };
    match seat.answered(&encode(&Gesture::Act(act)))? {
        Reply::Outcome { ok: true, .. } => Ok(()),
        Reply::Outcome { stderr, .. } => Err(format!("deposit refused: {stderr}")),
        other => Err(kind_err("deposit", &other)),
    }
}

/// The wrong-kind sentence names the kind, never the rows it carried.
fn kind_err(asked: &str, got: &Reply) -> String {
    let kind = match got {
        Reply::Outcome { .. } => "outcome",
        Reply::Workspaces { .. } => "workspaces",
        Reply::Conversations(_) => "conversations",
        Reply::Transcript(_) => "transcript",
    };
    format!("{asked}: the engine answered {kind} instead")
}
