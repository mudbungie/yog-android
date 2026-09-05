//! **The worker's loop**: one pass, one wait, and the gestures that wake it.
//! Split from the handle in `model.rs` (bl-dfbb) on the seam the model has
//! always had — what the frame HOLDS, and what the thread DOES.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde_json::Value;

use super::model::Cmd;
use super::pass::Standing;
use super::{Focus, Snapshot};
use crate::transport::Seat;

pub(super) fn run(
    seat: &Seat,
    cadence: Duration,
    cache: &std::path::Path,
    kept: Option<(Focus, Snapshot, super::options::Options, Option<Value>)>,
    cmds: &mpsc::Receiver<Cmd>,
    lanes: &mpsc::Sender<Cmd>,
    out: &mpsc::Sender<Snapshot>,
) {
    let (mut focus, mut standing) = match kept {
        // The selectors' offerings come back with the rows (bl-0267): the
        // file holds them under the workspace they were read for, so a
        // resumed seat opens its selectors instantly and offline.
        Some((focus, snap, options, queue)) => (focus, Standing::resumed(snap, options, queue)),
        None => (Focus::default(), Standing::default()),
    };
    let mut note = None;
    loop {
        // An undeliverable snapshot is not a stop signal: `Model::drop` sends
        // `Stop` before the receiver can go away (join precedes field drop),
        // so shutdown always arrives as a command, never as a dead channel.
        let _ = out.send(standing.pass(seat, cache, &focus, note.take(), lanes));
        match wait(cmds, cadence, &mut standing, &focus, out) {
            Ok(Cmd::Workspace(workspace)) => {
                let was = focus.workspace.take();
                focus = Focus {
                    workspace,
                    agent: None,
                };
                // The assignments are a fact about the WORKSPACE, so they are
                // read when the workspace changes and not when the focus
                // merely deepens into a conversation inside it (bl-e9f9).
                if was != focus.workspace {
                    preload(seat, &focus, &mut standing);
                }
            }
            Ok(Cmd::Conversation(workspace, agent)) => {
                let was = focus.workspace.take();
                focus = Focus {
                    workspace: Some(workspace),
                    agent: Some(agent),
                };
                if was != focus.workspace {
                    preload(seat, &focus, &mut standing);
                }
            }
            Ok(Cmd::Deposit(content)) => {
                // The receipt is counted as well as reported: the composer's
                // echo has no other way to know its message landed (bl-66fb),
                // and since bl-07b1 there are three fates to count rather than
                // two — a lost reply is not a refusal, and an echo that read
                // it as one would hand the operator back a draft the engine
                // may already have taken.
                let posted = super::acts::deposit(seat, &focus, content);
                standing.posted(&posted);
                note = posted.note();
            }
            // The three selector gestures. A read's answer is learned as the
            // engine's own envelope (bl-0267); a failure is a sentence for
            // the banner exactly as an act's is.
            Ok(Cmd::Providers) => {
                note = learned(super::asks::providers(seat, &focus), None, &mut standing);
            }
            Ok(Cmd::Models(provider)) => {
                let listed = super::asks::models(seat, &focus, &provider);
                note = learned(listed, Some(provider), &mut standing);
            }
            // **A search needs no read after it either**, and no read before
            // it: it names no place, so nothing about the focus decides what
            // it means. The answer is held by `Standing` and painted onto
            // every snapshot after it, exactly as the deposit counters are.
            Ok(Cmd::Search(text)) => {
                note = searched(super::asks::search(seat, &text), &mut standing);
            }
            // **The one world read a gesture makes** (§13.8; the queue is the
            // lane's, §14.1), and the two acts over the trail. The read
            // replaces what it answers and a failure keeps what was there,
            // which is `searched`'s rule and is here for its reason: losing
            // an answer the engine gave over one it did not is the defect
            // §13.2's grace exists to prevent.
            Ok(Cmd::Ops) => {
                note = match super::asks::ops(seat) {
                    Ok(rows) => {
                        standing.trail = rows;
                        None
                    }
                    Err(why) => Some(why),
                };
            }
            // **The ball pane's read** (§13.9), on the trail's terms exactly:
            // opening the surface is the ask, the answer replaces what was
            // held, and a failure keeps what was there — losing an answer the
            // engine gave over one it did not is the defect §13.2's grace
            // exists to prevent.
            Ok(Cmd::Balls(view)) => {
                note = match super::asks::balls(seat, &focus, view) {
                    Ok(pane) => {
                        standing.pane = Some(pane);
                        None
                    }
                    Err(why) => Some(why),
                };
            }
            // **Both trail acts are followed by the read that says what they
            // did**, which is also the read that settles a lost one: the
            // watermark and the truncation are both invisible until the trail
            // is read again, so the screen would otherwise stand on what it
            // had before the act.
            Ok(Cmd::Ack) => {
                note = super::acts::ack(seat).note();
                reread(seat, &mut standing);
            }
            // **The queue act needs no read after it, and that is the lane's
            // doing** (§14.1). The two trail acts are followed by `reread`
            // because nothing stands over the trail; the queue has a held
            // lane whose whole contract is a frame whenever what needs the
            // operator changes, so the acknowledgement's effect arrives here
            // as the next frame — from the one writer, in the engine's own
            // words, without this seat asking twice.
            Ok(Cmd::Seen(workspace, agent)) => {
                note = super::acts::seen(seat, workspace, agent).note();
            }
            Ok(Cmd::ClearTrail) => {
                note = super::acts::clear_trail(seat).note();
                reread(seat, &mut standing);
            }
            Ok(Cmd::StopTurn(children)) => {
                note = super::acts::stop(seat, &focus, children).note();
            }
            Ok(Cmd::Nudge) => note = super::acts::nudge(seat, &focus).note(),
            // **A row act needs no read after it** (§13.5). The three write
            // nothing a control is showing optimistically — unlike the tuning
            // pair below, whose whole point is that the assignments read
            // overtakes the guess — and what each of them DID lands in the
            // standing set this loop is already re-asking for: the transcript,
            // the row's flight, the row's attention mark. So the gesture wakes
            // the pass and the pass is the recovery.
            // **An answer needs no read after it either**: what it did lands
            // in the standing set this loop already re-asks — the transcript,
            // the row's flight, and the queue row that stops carrying the
            // parked call the moment it is answered.
            Ok(Cmd::Answer(verdict)) => {
                note = super::acts::answer(seat, &focus, verdict).note();
            }
            Ok(Cmd::Row(agent, act)) => {
                note = super::acts::row(seat, &focus, agent, act).note();
            }
            // A tuning act is followed by the read that makes it true: the
            // control showed its pick optimistically, and this is what
            // overtakes it (bl-e9f9).
            Ok(Cmd::Effort(level)) => {
                note = super::acts::effort(seat, &focus, level).note();
                preload(seat, &focus, &mut standing);
            }
            Ok(Cmd::Priority(on)) => {
                note = super::acts::priority(seat, &focus, on).note();
                preload(seat, &focus, &mut standing);
            }
            Ok(Cmd::Pick(provider, model)) => {
                note = super::acts::pick(seat, &focus, &provider, &model).note();
                preload(seat, &focus, &mut standing);
            }
            Ok(Cmd::Start(goal)) => note = super::acts::started(seat, &focus, goal).note(),
            Ok(Cmd::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => return,
            // A lane's frame never comes back from `wait` — it is adopted
            // there, inside the same deadline — so it is the tick's arm.
            Ok(Cmd::Lane(_)) | Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
    }
}

/// **Read what the workspace is set to, and say nothing if it cannot be
/// read** (bl-e9f9). This is a preload, not an answer to a gesture the
/// operator made: its absence means the controls seed from nothing, which is
/// exactly where they stood before this read existed. So every way it can
/// fail is swallowed — including the one that will be common for a while, an
/// engine that predates the read and refuses the op in band by name. A
/// banner for that would be this app telling an operator off for running the
/// engine they have.
fn preload(seat: &Seat, focus: &Focus, standing: &mut Standing) {
    if let Ok((workspace, envelope)) = super::asks::roles(seat, focus) {
        standing.options.assigned(&workspace, envelope);
        standing.reads += 1;
    }
}

/// **Re-read the trail after an act on it**, swallowing the failure: this is
/// not the operator's gesture, it is what makes the gesture's effect visible,
/// and its absence leaves the rows exactly as they were — which is where they
/// stood before the act was fired. `preload`'s rule, on the other pair.
fn reread(seat: &Seat, standing: &mut Standing) {
    if let Ok(rows) = super::asks::ops(seat) {
        standing.trail = rows;
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

/// **One search, held** (bl-4c2b) — `learned`'s shape for the other read a
/// gesture makes, and here for the same reason: what the fold does with an
/// answer is the worker's business, and what a PASS means is `seat::pass`'s.
///
/// The answer replaces whatever was held. A failure does NOT: the operator is
/// still looking at the hits they have, and dropping an answer the engine gave
/// because of one it did not is the defect §13.2's grace exists to prevent.
fn searched(
    read: Result<Option<crate::codec::Found>, String>,
    standing: &mut Standing,
) -> Option<String> {
    match read {
        Ok(found) => {
            standing.found = found;
            None
        }
        Err(why) => Some(why),
    }
}

/// **How long between passes — and what arrives inside that wait** (§14.1).
///
/// The world is re-read at the model's own cadence. What arrives between
/// passes is the held lanes' frames — the queue as it changes, the tail as it
/// is written — each adopted onto the standing and published at once, so
/// arriving text appears at the engine's write cadence rather than once a
/// pass. A frame is not a gesture: it wakes no pass, and the wait goes on to
/// the same deadline. (The tail used to be re-asked here one shot at a time
/// on a 500 ms rest, bl-4822; the intake holds that read, so a one-shot ask
/// of it waited a hold. The lane is what replaced it.)
///
/// It hands back exactly what `recv_timeout` hands back, so the loop above
/// reads the same three outcomes it always did.
fn wait(
    cmds: &mpsc::Receiver<Cmd>,
    cadence: Duration,
    standing: &mut Standing,
    focus: &Focus,
    out: &mpsc::Sender<Snapshot>,
) -> Result<Cmd, mpsc::RecvTimeoutError> {
    let deadline = Instant::now() + cadence;
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        match cmds.recv_timeout(left) {
            Ok(Cmd::Lane(framed)) => {
                standing.adopted(framed);
                let _ = out.send(standing.publish(focus));
            }
            other => return other,
        }
    }
}
