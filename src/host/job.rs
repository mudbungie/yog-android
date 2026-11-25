//! **One job: run it, count it, post it** — and, on a server that pushes, the
//! set of jobs this host holds (REMOTE §5.8.1, bl-1782). Split from the loop
//! because the loop is a ladder of channels and this is what outlives them: a
//! job started on one channel finishes on whatever channel is up when it
//! does, and the ids it holds are named on every read in between.
//!
//! **What a pushing host owes, and where each clause lives here.** *Start
//! every pushed job at once, on its own thread, with no limit* — [`Held::start`]
//! spawns one thread per row and keeps nothing waiting. *Post its results on
//! its own connection* — [`Crew::post`] is a `complete` of its own, never the
//! read's. *Keep an id in `holding` until its final `complete` is answered* —
//! an id leaves [`Held`] only when a post landed or was refused. *A pushed id
//! already in `holding` is not started twice* — [`Held::start`]'s insert is
//! the test. *Hold no backlog* — there is no queue here to hold one in.
//!
//! **A post the wire swallowed is posted again, not run again** (REMOTE §5.6
//! Ruling 1, and §5.8's note that a pushing host needs no ordering for it).
//! The capture goes back to the worker, which hands it to a fresh post thread
//! the next time it wakes — at the next frame, a heartbeat at most thirty
//! seconds off, or at the next channel the ladder opens. So the re-post is
//! paced by the read it rides beside and needs no clock of its own, and while
//! it waits the id is still listed, so the server neither hands it out again
//! nor answers it in doubt.
//!
//! **Nothing here is shared state.** The jobs speak to the worker over one
//! channel ([`Back`]) and to the frame over the other ([`Note`]); the worker
//! reads its channel whenever it is about to name what it holds.

use std::collections::BTreeSet;
use std::sync::{Arc, mpsc};

use super::{Dispatch, Note};
use crate::codec::{Capture, Invocation};
use crate::foot::Foot;

/// What a job thread tells the worker when it is done with the wire.
enum Back {
    /// The final post was answered — landed or refused — so the id is no
    /// longer this host's to name.
    Answered(String),
    /// The post did not come back; the capture is still this host's, and so
    /// is the id.
    Unposted(String, Capture),
}

/// What every job thread carries: the wire, the tools, and the two channels.
#[derive(Clone)]
pub(super) struct Crew {
    foot: Arc<Foot>,
    run: Arc<Dispatch>,
    out: mpsc::Sender<Note>,
    back: mpsc::Sender<Back>,
}

/// **Every job this host has taken whose final `complete` is unanswered** —
/// the `holding` of every pushing read. It lives as long as the host, across
/// every channel, because a job does.
pub(super) struct Held {
    ids: BTreeSet<String>,
    back: mpsc::Receiver<Back>,
    crew: Crew,
}

impl Held {
    pub(super) fn new(foot: &Arc<Foot>, run: &Arc<Dispatch>, out: &mpsc::Sender<Note>) -> Self {
        let (back, from_jobs) = mpsc::channel();
        Self {
            ids: BTreeSet::new(),
            back: from_jobs,
            crew: Crew {
                foot: Arc::clone(foot),
                run: Arc::clone(run),
                out: out.clone(),
                back,
            },
        }
    }

    /// The `holding` list, read the moment it is sent.
    pub(super) fn list(&mut self) -> Vec<String> {
        self.settle();
        self.ids.iter().cloned().collect()
    }

    /// Start every row this host does not already hold, each on its own
    /// thread, now.
    pub(super) fn start(&mut self, rows: Vec<Invocation>) {
        self.settle();
        for job in rows {
            if self.ids.insert(job.id.clone()) {
                let crew = self.crew.clone();
                std::thread::spawn(move || {
                    let capture = execute(&crew.run, &job, &crew.out);
                    crew.post(job.id, capture);
                });
            }
        }
    }

    /// Read what the jobs said: drop what was answered, post again what was
    /// not.
    fn settle(&mut self) {
        while let Ok(said) = self.back.try_recv() {
            match said {
                Back::Answered(id) => {
                    self.ids.remove(&id);
                }
                Back::Unposted(id, capture) => {
                    let crew = self.crew.clone();
                    std::thread::spawn(move || crew.post(id, capture));
                }
            }
        }
    }
}

impl Crew {
    /// One post, on its own connection. Whatever the server said is an
    /// answer; only a channel that failed keeps the capture.
    fn post(self, id: String, capture: Capture) {
        let said = match self.foot.complete(id.clone(), capture.clone()) {
            Err(why) if why.transport() => Back::Unposted(id, capture),
            Ok(()) | Err(_) => Back::Answered(id),
        };
        // A worker that has gone has nobody left to name the id to.
        let _ = self.back.send(said);
    }
}

/// Run one invocation — or refuse the `cwd` it carries — and say it ran. The
/// one place a tool runs, for the serial host and the pushing one alike.
pub(super) fn execute(
    run: &Dispatch,
    invocation: &Invocation,
    out: &mpsc::Sender<Note>,
) -> Capture {
    let capture = match &invocation.cwd {
        None => run(&invocation.tool, &invocation.input),
        Some(cwd) => unconsented(&invocation.tool, cwd),
    };
    // A frame that has gone is the worker's to notice, at its next publish.
    let _ = out.send(Note::Ran(format!(
        "{} → {}",
        invocation.tool, capture.exit_code
    )));
    capture
}

/// The refusal a carried `cwd` earns: the capture's three facts, the sentence
/// on stderr where a tool's diagnostics go, and the key named — because the
/// reader is a model and the fixer is an operator with another machine. It
/// says what would run it instead rather than only what will not, which is
/// REMOTE §5.4's own posture for every miss in this lane.
fn unconsented(tool: &str, cwd: &str) -> Capture {
    crate::tools::refused(
        crate::tools::UNCONSENTED,
        &format!(
            "this machine does not run {tool} at a directory an invocation names: \
             it advertises no \"subject_cwd\" consent, because it dispatches to a \
             function of its own rather than spawning a program somewhere, and a \
             phone holds no worktree of this conversation. {cwd} was not entered \
             and nothing ran. Route this to the box that holds the server's \
             worktrees, or load this machine's tool by name to run it where this \
             machine runs things."
        ),
    )
}
