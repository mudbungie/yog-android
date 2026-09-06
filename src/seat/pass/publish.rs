//! **What every published snapshot is made of** (DESIGN §14.1): the last
//! answered rows with the standing folded over them — counters, world reads,
//! the tail, the selectors, the sentence. Split from `pass.rs` when the held
//! lanes made a snapshot something a FRAME publishes as well as a pass: one
//! builder, so the two cannot drift into two readings of what stands.

use super::{Focus, GRACE, Snapshot, Standing};
use crate::seat::lane::Subject;
use crate::seat::posted::Posted;

impl Standing {
    /// The lanes a pass wants standing (§14.1): the queue's always, the tail's
    /// while the focused conversation is writing, and the sign-in's while a
    /// provider's run is being watched and has not settled (§13.19).
    pub(super) fn wanted(&self, focus: &Focus) -> Vec<Subject> {
        let mut wanted = vec![Subject::Attention];
        wanted.extend(self.signing.wanted(focus));
        if let (true, Some(workspace), Some(agent)) = (
            self.streaming(focus),
            focus.workspace.clone(),
            focus.agent.clone(),
        ) {
            wanted.push(Subject::Follow { workspace, agent });
        }
        wanted
    }

    /// **The snapshot the frame should paint**, built from what stands: the
    /// last answered rows, the counters, the world reads, the tail folded
    /// over the transcript, the selectors' offerings — and the sentence.
    ///
    /// **A failure is not an error until it persists** (bl-3202): a refresh
    /// failure paints once it has persisted past [`GRACE`], and a pass that
    /// succeeds clears it instantly. **The note never waits.** It is a
    /// gesture's own answer — a refused deposit, a start the engine would not
    /// run, a lane's frame this build could not read — and the operator just
    /// acted. Silence there is a message that vanished.
    pub(in crate::seat) fn publish(&self, focus: &Focus) -> Snapshot {
        let mut out = self.last.clone();
        (out.landed, out.refused, out.doubted) = self.posted;
        out.roles_read = self.reads;
        self.world(&mut out);
        // One tail on the glass, and none at rest (bl-e3d1). The gate is the
        // row's own flight, so the transcript's tail obeys exactly what the
        // lane obeys.
        let flying = self.streaming(focus);
        out.transcript = crate::live::settled(out.transcript, self.live.as_ref(), flying);
        // Painted onto the published snapshot as well as onto `fresh`: a
        // pass that failed republishes last-good rows, and the selectors'
        // offerings are not the pass's to lose (bl-0267).
        self.options.paint(focus, &mut out);
        let failed = self.failure.clone().filter(|_| self.failed > GRACE);
        out.error = match (self.note.clone(), failed) {
            (Some(note), Some(failed)) => Some(format!("{note}; {failed}")),
            (note, failed) => note.or(failed),
        };
        out
    }

    /// **The reads whose subject is the world**, painted onto a snapshot
    /// however it was built (§13.6, §13.7, §13.8, §13.9). Four fields, one
    /// place:
    /// each is a gesture's answer rather than a depth's, so a pass that failed
    /// or narrowed must not drop any of them — and three copies of that rule
    /// at three publishers is how one of them comes to be forgotten.
    fn world(&self, out: &mut Snapshot) {
        out.search.clone_from(&self.found);
        out.queue.clone_from(&self.queue);
        out.trail.clone_from(&self.trail);
        out.pane.clone_from(&self.pane);
        out.records.clone_from(&self.records);
        out.candidates.clone_from(&self.candidates);
        out.clients.clone_from(&self.clients);
        out.files.clone_from(&self.files);
        out.work.clone_from(&self.work);
        out.config.clone_from(&self.config);
        out.marks.clone_from(&self.marks);
        out.minted.clone_from(&self.minted);
        out.login = self.signing.painted();
    }

    /// **One deposit's fate, counted** (bl-66fb). The composer's echo cannot
    /// see the receipt — the worker holds the wire — so what it watches is
    /// these counters moving.
    ///
    /// **Three, since bl-07b1**: a lost reply is not a refusal (yog REMOTE
    /// §3), and counting it as one made the echo hand its text back to the
    /// composer — an invitation to send a message the engine may already have
    /// taken. The third counter is what lets the echo stand instead.
    pub(in crate::seat) fn posted(&mut self, fate: &Posted) {
        match fate {
            Posted::Took => self.posted.0 += 1,
            Posted::Refused(_) => self.posted.1 += 1,
            Posted::InDoubt(_) => self.posted.2 += 1,
        }
    }

    /// **Whether the focused conversation is writing right now** — read off
    /// the row's own `flight`, which is where every conversation-level gate
    /// rides (REMOTE §9.4). A conversation the list has not caught up with
    /// has no row and so is not streaming, which is the honest answer.
    pub(in crate::seat) fn streaming(&self, focus: &Focus) -> bool {
        let Some(agent) = focus.agent.as_deref() else {
            return false;
        };
        self.last
            .conversations
            .iter()
            .find(|row| row.root_id == agent)
            .is_some_and(|row| row.flight.is_some())
    }
}
