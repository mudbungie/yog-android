//! **The handles that SAY** — every command the frame sends that changes the
//! world, split from the handle itself (bl-5a56) on the seam its neighbour
//! `model/reads.rs` states: the boundary's grammar is asks and acts, and this
//! crate reads it that way at four other sites.
//!
//! **Not one of these is ever sent twice.** An act is not idempotent (REMOTE
//! §9.8: two taps of Nudge are two nudges), no idempotency token rides the
//! envelope, and a reply this end never received leaves the act IN DOUBT — so
//! what a lost one earns is the banner's sentence and the read that settles
//! it, never a resend (`seat::posted`).

use super::Model;
use crate::seat::cmd::Cmd;

impl Model {
    /// Post the composer's text to the focused conversation. The receipt —
    /// or the refusal — arrives with the next snapshot.
    pub fn deposit(&self, content: String) {
        let _ = self.cmds.send(Cmd::Deposit(content));
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

    /// **Answer the parked tool call** in the focused conversation (§13.7,
    /// bl-b39d). The subject is the focus and not a row: answering is what an
    /// operator does after reading what the call is about to do, which is a
    /// thing only the transcript screen shows.
    ///
    /// Not idempotent and never re-sent: the queue read that no longer carries
    /// the call is what settles a lost one (`seat::acts::held`).
    pub fn answer(&self, verdict: crate::codec::Verdict) {
        let _ = self.cmds.send(Cmd::Answer(verdict));
    }

    /// **Mint the next device's material** (REMOTE §8.4, §13.18) in the
    /// focused workspace, under `name` and at `grade`.
    ///
    /// Not idempotent and never re-sent: a second mint under one name is
    /// refused by the certificate the engine kept, so a repeat on a lost reply
    /// would report the name as taken rather than minting it
    /// (`seat::acts::enroll`).
    pub fn enroll(&self, name: String, grade: crate::leaf::Grade) {
        let _ = self.cmds.send(Cmd::Enroll(name, grade));
    }

    /// **Forget what a mint answered with** — the one handle here whose whole
    /// product is that something is gone. It crosses no wire.
    pub fn forget(&self) {
        let _ = self.cmds.send(Cmd::Forget);
    }

    /// **Fire one act of the admin surface** (§13.17) — a config write, a
    /// task-branch mark, an inbox flush, or one of the two deletions.
    ///
    /// Not idempotent, any of the five, so nothing here is ever sent twice: a
    /// repeated config write re-applies bytes the operator may have edited
    /// since, and a lost reply becomes the banner's sentence and the read that
    /// settles it (`seat::acts::admin`).
    pub fn admin(&self, act: crate::codec::AdminAct) {
        let _ = self.cmds.send(Cmd::Admin(act));
    }

    /// **Fork the focused conversation at a picked point** (§13.16), with
    /// `goal` as the child's first instruction.
    ///
    /// Not idempotent and never re-sent: a fork materializes a worktree and
    /// starts a driver, so a repeat is a second child doing the same work —
    /// the read that settles a lost one is the spine the gesture was fired
    /// from (`seat::acts::fork`).
    pub fn fork(&self, from: String, goal: String) {
        let _ = self.cmds.send(Cmd::Fork { from, goal });
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

    /// **Fire one of the candidates screen's acts** (§13.12) at the obligation
    /// the row named. Not idempotent, any of the three — a repeated fan is n
    /// more worktrees — so nothing here is ever sent twice: a lost reply
    /// becomes the banner's sentence and the listing is what settles it.
    pub fn candidate_act(&self, project: String, ball: String, act: crate::codec::CandidateAct) {
        let _ = self.cmds.send(Cmd::Candidate(project, ball, act));
    }

    /// **Spread one obligation over `n` candidates and fire each with
    /// `goal`** (§13.12). One handle rather than three: a fan is a chain, and
    /// what the frame knows about it is the count and the instruction.
    pub fn fan(&self, project: String, ball: String, n: usize, goal: String) {
        let _ = self.cmds.send(Cmd::Fan {
            project,
            ball,
            n,
            goal,
        });
    }

    /// **Raise or lower one of the workspace's two armings** (§13.13) — the
    /// drone loop, or the alignment monitor. Not idempotent in any sense worth
    /// relying on and never re-sent: a repeated `fleet` re-arms a loop that
    /// may already have claimed balls, and the read that settles a lost one is
    /// the board.
    pub fn fleet_act(&self, act: crate::codec::FleetAct) {
        let _ = self.cmds.send(Cmd::Fleet(act));
    }

    /// **Acknowledge the trail's alarms** (yog §4.2, §7.3). Not idempotent in
    /// any sense worth relying on and never re-sent: the watermark lands on
    /// the trail as it stood, and the trail read after it is what says so.
    pub fn ack_trail(&self) {
        let _ = self.cmds.send(Cmd::Ack);
    }

    /// **Answer the attention queue at the conversation this row names**
    /// (yog §8.5, DESIGN §13.8). Not idempotent in any sense worth relying on
    /// and never re-sent: what says the mark is down is the attention lane's
    /// next frame, which arrives on its own the moment the write lands.
    pub fn seen(&self, workspace: String, agent: String) {
        let _ = self.cmds.send(Cmd::Seen(workspace, agent));
    }

    /// **Fire one of the ball pane's acts** (§13.9) at the ball the control
    /// hangs on — the project is the row's, because a project is a fact only
    /// a row carries here, and the `--as` stamp is the focused workspace's and
    /// is read where the focus lives.
    ///
    /// Not idempotent, any of the five, so nothing here is ever sent twice: a
    /// lost reply becomes the banner's sentence and the pane's own read is
    /// what settles it (`seat::acts::ball`).
    pub fn ball_act(&self, project: String, act: crate::codec::BallAct) {
        let _ = self.cmds.send(Cmd::Ball(project, act));
    }

    /// **Truncate the trail.** The arming is the control's, not the model's:
    /// a handle that armed itself would be a second authority for what is on
    /// the glass, and this seat's rule is that the tap IS the act (§13.2).
    pub fn clear_trail(&self) {
        let _ = self.cmds.send(Cmd::ClearTrail);
    }

    /// Start a new conversation in the focused workspace with `goal` as its
    /// first instruction. The staging and the firing are one gesture from
    /// here because they are one act to the operator; the engine's two-step
    /// is the wire's business, not the composer's.
    pub fn start_conversation(&self, goal: String) {
        let _ = self.cmds.send(Cmd::Start(goal));
    }
}
