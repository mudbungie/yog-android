//! **The worker's loop**: one pass, one wait, and the gestures that wake it.
//! Split from the handle in `model.rs` (bl-dfbb) on the seam the model has
//! always had — what the frame HOLDS, and what the thread DOES.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use super::model::Cmd;
use super::pass::Standing;
use super::{Focus, Snapshot};
use crate::transport::Seat;

pub(super) fn run(
    seat: &Seat,
    cadence: Duration,
    cache: &std::path::Path,
    kept: Option<(Focus, Snapshot, super::options::Options)>,
    cmds: &mpsc::Receiver<Cmd>,
    out: &mpsc::Sender<Snapshot>,
) {
    let (mut focus, mut standing) = match kept {
        // The selectors' offerings come back with the rows (bl-0267): the
        // file holds them under the workspace they were read for, so a
        // resumed seat opens its selectors instantly and offline.
        Some((focus, snap, options)) => (focus, Standing::resumed(snap, options)),
        None => (Focus::default(), Standing::default()),
    };
    let mut note = None;
    loop {
        // An undeliverable snapshot is not a stop signal: `Model::drop` sends
        // `Stop` before the receiver can go away (join precedes field drop),
        // so shutdown always arrives as a command, never as a dead channel.
        let _ = out.send(standing.pass(seat, cache, &focus, note.take()));
        match wait(seat, cmds, cadence, &mut standing, &focus, out) {
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
            Ok(Cmd::Deposit(content)) => {
                // The receipt is counted as well as reported: the composer's
                // echo has no other way to know its message landed (bl-66fb).
                let posted = super::acts::deposit(seat, &focus, content);
                standing.posted(posted.is_ok());
                note = posted.err();
            }
            // The three selector gestures. A read's answer is learned as the
            // engine's own envelope (bl-0267); a failure is a sentence for
            // the banner exactly as an act's is.
            Ok(Cmd::Providers) => {
                note = learned(super::acts::providers(seat, &focus), None, &mut standing);
            }
            Ok(Cmd::Models(provider)) => {
                let listed = super::acts::models(seat, &focus, &provider);
                note = learned(listed, Some(provider), &mut standing);
            }
            Ok(Cmd::StopTurn(children)) => {
                note = super::acts::stop(seat, &focus, children).err();
            }
            Ok(Cmd::Nudge) => note = super::acts::nudge(seat, &focus).err(),
            Ok(Cmd::Effort(level)) => {
                note = super::acts::effort(seat, &focus, level).err();
            }
            Ok(Cmd::Priority(on)) => note = super::acts::priority(seat, &focus, on).err(),
            Ok(Cmd::Pick(provider, model)) => {
                note = super::acts::pick(seat, &focus, &provider, &model).err();
            }
            Ok(Cmd::Start(goal)) => note = super::acts::started(seat, &focus, goal).err(),
            Ok(Cmd::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => return,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
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

/// **How long between passes — and what happens inside that wait** (bl-4822).
///
/// The world is re-read at the model's own cadence and nothing about that
/// changed. What changed is that a conversation which is WRITING is asked for
/// its tail on a quicker rest while the wait runs: `follow` one shot at a
/// time (REMOTE §5.5), published as it lands, so arriving text appears four
/// times a rest instead of once a cadence. Each tick costs one small read of
/// the answer so far rather than a whole transcript, which is the difference
/// between smoothing the arrival and paying the amplification the lane was
/// built to remove (upstream measured 20x, quadratic in the answer's length).
///
/// It hands back exactly what `recv_timeout` hands back, so the loop above
/// reads the same three outcomes it always did.
fn wait(
    seat: &Seat,
    cmds: &mpsc::Receiver<Cmd>,
    cadence: Duration,
    standing: &mut Standing,
    focus: &Focus,
    out: &mpsc::Sender<Snapshot>,
) -> Result<Cmd, mpsc::RecvTimeoutError> {
    let deadline = Instant::now() + cadence;
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() || !standing.streaming(focus) {
            // Not writing, or the cadence is up: the caller's own pass is the
            // next thing that should happen.
            return cmds.recv_timeout(left);
        }
        match cmds.recv_timeout(LIVE_REST.min(left)) {
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let _ = out.send(standing.living(seat, focus));
            }
            other => return other,
        }
    }
}

/// How long between reads of an answer being written. Short enough that text
/// arrives in pieces an eye can follow, long enough that a phone's radio is
/// not held awake for a paragraph.
const LIVE_REST: Duration = Duration::from_millis(500);
