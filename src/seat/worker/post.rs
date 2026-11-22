//! **What one act does** — the table the poster spends (`seat::poster`,
//! bl-2cad), and the half of each act the worker finishes. Split from the
//! command table beside it when the acts were given a thread of their own: an
//! act used to be an arm of `spend`, so it left only when the worker had
//! finished whatever read it was inside.
//!
//! **Every arm is two halves, and the seam between them is the thread.** The
//! first is the wire crossing, run where this is called — on the poster, on a
//! connection of its own, at once. The second is the [`Fold`] it hands back:
//! what the answer does to the standing, and the read that makes the act
//! visible (`worker::after`). The worker owns the standing and every read, so
//! the fold is run there, as a command like any other. An act therefore never
//! waits behind a read, and a read never waits behind an act.
//!
//! **The fold carries the focus the act was posted at**, not the focus the
//! worker stands at when it runs: the read that shows what an act did is
//! about the place the act was fired from.

use super::super::pass::Standing;
use super::super::posted::Posted;
use super::super::say::Say;
use super::super::{Focus, acts};
use super::after::{self, preload, reread};
use crate::transport::Seat;

/// What the worker does with an act's answer: fold it onto the standing, make
/// the read that shows it, and hand back the sentence it earned, or none.
pub(in crate::seat) type Fold = Box<dyn FnOnce(&Seat, &mut Standing) -> Option<String> + Send>;

/// Post one act at `focus`, and hand back what is left to do about it.
pub(in crate::seat) fn post(seat: &Seat, focus: &Focus, say: Say) -> Fold {
    let focus = focus.clone();
    match say {
        // The receipt is counted as well as reported: the composer's echo has
        // no other way to know its message landed (bl-66fb), and since bl-07b1
        // there are three fates to count rather than two — a lost reply is not
        // a refusal, and an echo that read it as one would hand the operator
        // back a draft the server may already have taken.
        Say::Deposit(content) => {
            let posted = acts::deposit(seat, &focus, content);
            Box::new(move |_: &Seat, standing: &mut Standing| {
                standing.posted(&posted);
                posted.note()
            })
        }
        // **The sign-in** (§13.19). The act's answer IS the tail's first
        // frame, so it is adopted rather than reported (`after::signed`).
        Say::Login(provider, code) => {
            let finishing = code.is_some();
            let (posted, view) = acts::login(seat, &focus, provider.clone(), code);
            Box::new(move |_: &Seat, standing: &mut Standing| {
                after::signed(standing, provider, finishing, posted, view)
            })
        }
        // **The mint, and the two that settle one** (§13.18): each is followed
        // by the pending rows it changed.
        Say::Offer(name, grade) => {
            let (posted, offered) = acts::offer(seat, &focus, name.clone(), grade);
            Box::new(move |seat: &Seat, standing: &mut Standing| {
                after::offered(seat, &focus, standing, name, posted, offered)
            })
        }
        Say::Settle(settle, name) => {
            let posted = acts::settle(seat, &focus, settle, name);
            Box::new(move |seat: &Seat, standing: &mut Standing| {
                after::pending(seat, &focus, standing, posted.note())
            })
        }
        Say::Admin(act) => {
            let workspace = match &act {
                crate::codec::AdminAct::Marks { workspace, .. } => Some(workspace.clone()),
                _ => None,
            };
            let settled = matches!(act, crate::codec::AdminAct::Proposal { .. });
            let (posted, landed) = acts::admin(seat, act);
            let marked = workspace.zip(landed);
            Box::new(move |seat: &Seat, standing: &mut Standing| {
                after::administered(seat, &focus, standing, marked, settled, posted)
            })
        }
        // **Both trail acts are followed by the read that says what they
        // did**, which is also the read that settles a lost one: the watermark
        // and the truncation are both invisible until the trail is read again,
        // so the screen would otherwise stand on what it had before the act.
        Say::Ack => trailed(acts::ack(seat)),
        Say::ClearTrail => trailed(acts::clear_trail(seat)),
        // **A start is followed into focus** (§25, bl-82d7): its receipt names
        // the conversation, and the worker's loop focuses the row that answers
        // to the name.
        Say::Start(goal) => {
            let (posted, named) = acts::started(seat, &focus, goal);
            Box::new(move |_: &Seat, standing: &mut Standing| {
                standing.claimed = named;
                posted.note()
            })
        }
        // The two aimed screens whose acts ARE invisible until re-read. **The
        // ball act's `--as` stamp is the focused workspace's own name**, or
        // the empty string where nothing is focused — which the server refuses
        // in its own words, and is the right end for that sentence to come
        // from (§13.9).
        Say::Ball(project, act) => {
            let name = focus.workspace.clone().unwrap_or_default();
            let posted = acts::ball(seat, project, name, act);
            Box::new(move |seat: &Seat, standing: &mut Standing| {
                after::repane(seat, &focus, standing);
                posted.note()
            })
        }
        Say::Candidate(project, ball, act) => {
            listed(focus, acts::candidate(seat, project, ball, act))
        }
        Say::Fan {
            project,
            ball,
            n,
            goal,
        } => {
            let posted = acts::spread(seat, &focus, project, ball, n, goal);
            listed(focus, posted)
        }
        // A tuning act is followed by the read that makes it true: the control
        // showed its pick optimistically, and this is what overtakes it
        // (bl-e9f9).
        Say::Effort(level) => {
            let posted = acts::effort(seat, &focus, level);
            tuned(focus, posted)
        }
        Say::Priority(on) => {
            let posted = acts::priority(seat, &focus, on);
            tuned(focus, posted)
        }
        Say::Pick(provider, model) => {
            let posted = acts::pick(seat, &focus, &provider, &model);
            tuned(focus, posted)
        }
        // **The rest need no read after them**: what each of them DID lands in
        // the standing set the worker is already re-asking for, or — the queue
        // act (§14.1) — arrives as the lane's next frame, or is on another
        // screen and says so in its own receipt (§13.13, §13.16). So the
        // receipt wakes the pass and the pass is the recovery.
        Say::Fork { from, goal } => said(acts::fork(seat, &focus, from, goal)),
        Say::Seen(workspace, agent) => said(acts::seen(seat, workspace, agent)),
        Say::Fleet(act) => said(acts::fleet(seat, &focus, act)),
        Say::StopTurn(children) => said(acts::stop(seat, &focus, children)),
        Say::Nudge => said(acts::nudge(seat, &focus)),
        Say::Answer(verdict, scope) => said(acts::answer(seat, &focus, verdict, scope)),
        Say::Row(agent, act) => said(acts::row(seat, &focus, agent, act)),
    }
}

/// An act whose whole fold is its sentence.
fn said(posted: Posted) -> Fold {
    Box::new(move |_: &Seat, _: &mut Standing| posted.note())
}

/// A trail act and the trail read that shows what it did.
fn trailed(posted: Posted) -> Fold {
    Box::new(move |seat: &Seat, standing: &mut Standing| {
        reread(seat, standing);
        posted.note()
    })
}

/// A candidate act and the listing that shows what it did.
fn listed(focus: Focus, posted: Posted) -> Fold {
    Box::new(move |seat: &Seat, standing: &mut Standing| {
        after::listed(seat, &focus, standing, posted)
    })
}

/// A tuning act and the read that overtakes its optimistic control.
fn tuned(focus: Focus, posted: Posted) -> Fold {
    Box::new(move |seat: &Seat, standing: &mut Standing| {
        let note = posted.note();
        preload(seat, &focus, standing);
        note
    })
}
