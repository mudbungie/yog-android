//! **What one gesture does** — the command table the loop next door spends,
//! split from it (bl-2f17) when the candidates screen's two commands took
//! `worker.rs` past clippy's line bound. The seam is the one that file already
//! reads as: `run` is the loop and `wait` is its clock, and this is *a command
//! arrived, now what does the seat do about it*.
//!
//! Every arm answers the same thing — the sentence the gesture earned, or none
//! — and the two that END the loop stay in `run`, because ending it is the
//! loop's own business and a table cannot say it.

use super::super::cmd::Cmd;
use super::super::pass::Standing;
use super::super::{Focus, acts, asks};
use super::after::{self, preload, reread};
use super::fold::{self, learned, searched};
use crate::transport::Seat;

/// Spend one command. `focus` is `&mut` because two of these MOVE it, which is
/// the one piece of loop state a gesture is allowed to write.
pub(super) fn spend(
    seat: &Seat,
    cmd: Cmd,
    focus: &mut Focus,
    standing: &mut Standing,
) -> Option<String> {
    match cmd {
        Cmd::Workspace(workspace) => aimed(
            seat,
            focus,
            standing,
            Focus {
                workspace,
                agent: None,
            },
        ),
        Cmd::Conversation(workspace, agent) => aimed(
            seat,
            focus,
            standing,
            Focus {
                workspace: Some(workspace),
                agent: Some(agent),
            },
        ),
        // The receipt is counted as well as reported: the composer's echo has
        // no other way to know its message landed (bl-66fb), and since bl-07b1
        // there are three fates to count rather than two — a lost reply is not
        // a refusal, and an echo that read it as one would hand the operator
        // back a draft the engine may already have taken.
        Cmd::Deposit(content) => {
            let posted = acts::deposit(seat, focus, content);
            standing.posted(&posted);
            posted.note()
        }
        // The three selector gestures. A read's answer is learned as the
        // engine's own envelope (bl-0267); a failure is a sentence for the
        // banner exactly as an act's is.
        Cmd::Providers => learned(asks::providers(seat, focus), None, standing),
        Cmd::Models(provider) => {
            let listed = asks::models(seat, focus, &provider);
            learned(listed, Some(provider), standing)
        }
        // **A search needs no read after it either**, and no read before it:
        // it names no place, so nothing about the focus decides what it means.
        // The answer is held by `Standing` and painted onto every snapshot
        // after it, exactly as the deposit counters are.
        Cmd::Search(text) => searched(asks::search(seat, &text), standing),
        // **The world reads a gesture makes** (§13.8, §13.9, §13.11, §13.12;
        // the queue is the lane's, §14.1). Each replaces what it answers and a
        // failure keeps what was there, which is `searched`'s rule and is here
        // for its reason: losing an answer the engine gave over one it did not
        // is the defect §13.2's grace exists to prevent.
        Cmd::Ops => fold::opened(asks::ops(seat), standing),
        Cmd::Balls(view) => fold::paned(asks::balls(seat, focus, view), standing),
        Cmd::Records => fold::recorded(asks::opened(seat, focus), standing),
        Cmd::Step(seq) => fold::drilled(asks::drill(seat, focus, seq), standing),
        // **The anchored read and the act it informs** (§13.16). The read
        // folds into the records like the drill-in beside it; the act needs no
        // read after it — what a fork DID is a child card on the spine, which
        // the next opening of this screen reads.
        Cmd::Anchor(at) => {
            let read = asks::anchored(seat, focus, at.clone());
            fold::anchored(read, at, standing)
        }
        Cmd::Fork { from, goal } => acts::fork(seat, focus, from, goal).note(),
        Cmd::Science => fold::spread(asks::science(seat, focus), standing),
        Cmd::Clients => fold::machined(asks::clients(seat, focus), standing),
        Cmd::Config(at) => fold::configured(asks::config(seat, at), standing),
        Cmd::Marks => fold::marked(asks::marks(seat, focus), standing),
        Cmd::Admin(act) => after::administered(seat, standing, act),
        Cmd::Files(path) => fold::filed(asks::files(seat, focus, path), standing),
        Cmd::Work(file) => fold::worked(asks::work(seat, focus, file), standing),
        // **Both trail acts are followed by the read that says what they
        // did**, which is also the read that settles a lost one: the watermark
        // and the truncation are both invisible until the trail is read again,
        // so the screen would otherwise stand on what it had before the act.
        Cmd::Ack => {
            let note = acts::ack(seat).note();
            reread(seat, standing);
            note
        }
        Cmd::ClearTrail => {
            let note = acts::clear_trail(seat).note();
            reread(seat, standing);
            note
        }
        // **The queue act needs no read after it, and that is the lane's
        // doing** (§14.1): the acknowledgement's effect arrives as the next
        // frame, from the one writer, without this seat asking twice.
        Cmd::Seen(workspace, agent) => acts::seen(seat, workspace, agent).note(),
        // **An arming needs no read after it** (§13.13): what it did is on the
        // board, which is a different screen, and its receipt is a sentence
        // rather than a silence for exactly that reason.
        Cmd::Fleet(act) => acts::fleet(seat, focus, act).note(),
        Cmd::StopTurn(children) => acts::stop(seat, focus, children).note(),
        Cmd::Nudge => acts::nudge(seat, focus).note(),
        // **An answer, a row act and a start need no read after them**: what
        // each of them DID lands in the standing set this loop is already
        // re-asking for — the transcript, the row's flight, the row's
        // attention mark, the queue row that stops carrying the parked call.
        // So the gesture wakes the pass and the pass is the recovery.
        Cmd::Answer(verdict) => acts::answer(seat, focus, verdict).note(),
        Cmd::Row(agent, act) => acts::row(seat, focus, agent, act).note(),
        Cmd::Start(goal) => acts::started(seat, focus, goal).note(),
        // The two aimed screens whose acts ARE invisible until re-read.
        Cmd::Ball(project, act) => after::balled(seat, focus, standing, project, act),
        Cmd::Candidate(project, ball, act) => {
            let posted = acts::candidate(seat, project, ball, act);
            after::listed(seat, focus, standing, posted)
        }
        Cmd::Fan {
            project,
            ball,
            n,
            goal,
        } => {
            let posted = acts::spread(seat, focus, project, ball, n, goal);
            after::listed(seat, focus, standing, posted)
        }
        // A tuning act is followed by the read that makes it true: the control
        // showed its pick optimistically, and this is what overtakes it
        // (bl-e9f9).
        Cmd::Effort(level) => tuned(seat, focus, standing, acts::effort(seat, focus, level)),
        Cmd::Priority(on) => tuned(seat, focus, standing, acts::priority(seat, focus, on)),
        Cmd::Pick(provider, model) => {
            let posted = acts::pick(seat, focus, &provider, &model);
            tuned(seat, focus, standing, posted)
        }
        // The two the loop keeps: `Stop` ends it and a lane's frame is adopted
        // inside `wait`, so neither reaches here — and neither has a sentence.
        Cmd::Stop | Cmd::Lane(_) => None,
    }
}

/// **Move the focus, and read what the new workspace is set to when it
/// changed** (bl-e9f9). The assignments are a fact about the WORKSPACE, so
/// they are read when the workspace moves and not when the focus merely
/// deepens into a conversation inside it. One body for both moves, because
/// that is the only thing the two arms ever differed in.
fn aimed(seat: &Seat, focus: &mut Focus, standing: &mut Standing, moved: Focus) -> Option<String> {
    let was = focus.workspace.take();
    *focus = moved;
    if was != focus.workspace {
        preload(seat, focus, standing);
    }
    None
}

/// A tuning act and the read that overtakes its optimistic control.
fn tuned(
    seat: &Seat,
    focus: &Focus,
    standing: &mut Standing,
    posted: crate::seat::posted::Posted,
) -> Option<String> {
    let note = posted.note();
    preload(seat, focus, standing);
    note
}

#[cfg(test)]
mod tests {
    use super::{Cmd, Focus, Standing, spend};
    use crate::test_support::{material, mint_ca, mint_leaf, scratch};
    use crate::transport::Seat;

    /// **The two the loop keeps have nothing to say here.** `run` ends on
    /// `Stop` and adopts a lane's frame inside `wait`, so neither ever
    /// reaches this table — but a `match` must be total, and the honest arm
    /// is the one that says there is no sentence rather than a panic this
    /// crate may not write.
    #[test]
    fn the_two_the_loop_keeps_earn_no_sentence() {
        let dir = scratch();
        mint_ca(&dir, "ca");
        mint_leaf(&dir, "ca", "client", false);
        // `Seat::open` dials nothing, so a leaf is the whole of what this
        // needs: the arm under test never touches the wire.
        let seat = Seat::open(&material(&dir, "ca", "client", "127.0.0.1:9")).unwrap();
        let said = spend(
            &seat,
            Cmd::Stop,
            &mut Focus::default(),
            &mut Standing::default(),
        );
        assert!(said.is_none());
    }
}
