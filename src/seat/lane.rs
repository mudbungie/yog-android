//! **The held lanes beside the pass** (DESIGN §14.1, bl-8e3c): the two
//! follow-class reads this seat keeps standing — the decision queue
//! (`attention`, REMOTE §14.1) and the focused turn's live tail (`follow`,
//! §5.5) — each a thread parked on one held connection, handing every frame
//! to the worker down the worker's own command channel.
//!
//! **Why held, and not asked.** The wire intake this seat dials HOLDS a
//! follow-class read: the first frame at connect, a frame per change, the
//! terminator when the hold ends — thirty seconds, the follow lane's own
//! bound. A one-shot read of that blocks for the whole hold, and the ask
//! carries no bound a seat could shorten it with (neither `attention` nor
//! `follow` has a field for one). Hanging up after the first frame would
//! park an engine thread per read for a hold's length — sixty of them a
//! minute at the tail's old 500 ms rest. So the lane is held, which is the
//! desktop seat's own shape (lernie DESIGN §4.12) with one difference: the
//! phone's worker owns the fold, and no lock is added — the frame is a
//! command like any gesture, and the worker adopts it where it adopts
//! everything else.
//!
//! **The pass is the lane's clock.** A lane is opened by a pass and only by
//! a pass; one that ended — the hold's bound, the step's commit, a socket
//! that went away — is reopened at the next pass, never by itself. That is
//! what bounds the redial rate to the cadence on a phone that changes
//! networks hourly, and what keeps the harness's request order a script.
//!
//! **A frame decides whether it is still wanted by its lane's id**, never by
//! its subject: the worker drops a lane whose subject moved *before* it opens
//! one on the new subject, so a frame from a dropped lane carries an id no
//! held lane has and is ignored. Hanging up is a socket shutdown from the
//! worker's thread, which wakes a reader parked up to a hold away — a flag
//! it read between frames could not.

use std::sync::mpsc;

use serde_json::Value;

use super::cmd::Cmd;
use crate::codec::{Ask, Gesture, encode};
use crate::transport::{Hangup, Seat};

/// What a lane is about — the ask it stands on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Subject {
    Attention,
    Follow { workspace: String, agent: String },
}

impl Subject {
    fn ask(&self) -> Ask {
        match self {
            Self::Attention => Ask::Attention,
            Self::Follow { workspace, agent } => Ask::Follow {
                workspace: workspace.clone(),
                agent: agent.clone(),
            },
        }
    }

    /// The word the kind-error names.
    pub(super) fn name(&self) -> &'static str {
        match self {
            Self::Attention => "attention",
            Self::Follow { .. } => "follow",
        }
    }
}

/// What a lane hands the worker: a frame, or its end. Stamped with the
/// lane's id so the worker can tell a held lane's frame from a dropped one's.
pub(super) enum Framed {
    Frame(u64, Value),
    Over(u64),
}

struct Lane {
    id: u64,
    subject: Subject,
    hangup: Hangup,
    reader: Option<std::thread::JoinHandle<()>>,
}

impl Drop for Lane {
    /// Hang up, then wait for the reader: the shutdown is what unparks it.
    fn drop(&mut self) {
        self.hangup.hang_up();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

/// The lanes the worker holds, and the id the next one takes.
#[derive(Default)]
pub(super) struct Lanes {
    held: Vec<Lane>,
    next: u64,
}

impl Lanes {
    /// **Make what stands match what is wanted.** Every lane whose subject is
    /// not wanted is hung up first; then, when `open` says the engine is
    /// answering, every wanted subject with no lane gets one.
    ///
    /// **A dial that fails is not a sentence.** The pass that called this has
    /// just been answered by the same engine at the same address, so a lane
    /// it cannot open is a moment's failure the next pass dials again — and
    /// if the engine is in fact gone, the next pass is what says so, through
    /// the one grace every refresh failure waits out (§13.2). A second
    /// sentence here would count one outage twice.
    pub(super) fn tend(
        &mut self,
        seat: &Seat,
        wanted: &[Subject],
        tx: &mpsc::Sender<Cmd>,
        open: bool,
    ) {
        self.held.retain(|lane| wanted.contains(&lane.subject));
        if !open {
            return;
        }
        for subject in wanted {
            if self.held.iter().any(|lane| lane.subject == *subject) {
                continue;
            }
            if let Ok(lane) = self.open(seat, subject.clone(), tx.clone()) {
                self.held.push(lane);
            }
        }
    }

    fn open(
        &mut self,
        seat: &Seat,
        subject: Subject,
        tx: mpsc::Sender<Cmd>,
    ) -> Result<Lane, String> {
        let (open, hangup) = seat.hold(&encode(&Gesture::Ask(subject.ask())))?;
        let id = self.next;
        self.next += 1;
        let reader = std::thread::spawn(move || {
            let _ = open.each(&mut |frame| tx.send(Cmd::Lane(Framed::Frame(id, frame))).is_ok());
            let _ = tx.send(Cmd::Lane(Framed::Over(id)));
        });
        Ok(Lane {
            id,
            subject,
            hangup,
            reader: Some(reader),
        })
    }

    /// The subject of a lane still held, or `None` for one already dropped.
    pub(super) fn subject(&self, id: u64) -> Option<Subject> {
        self.held
            .iter()
            .find(|lane| lane.id == id)
            .map(|lane| lane.subject.clone())
    }

    /// A lane's stream ended: forget it, and say what it was about. The next
    /// pass reopens it if it is still wanted.
    pub(super) fn ended(&mut self, id: u64) -> Option<Subject> {
        let at = self.held.iter().position(|lane| lane.id == id)?;
        Some(self.held.remove(at).subject.clone())
    }
}
