//! **The worker's loop**: one pass, one wait, and the gestures that wake it.
//! Split from the handle in `model.rs` (bl-dfbb) on the seam the model has
//! always had — what the frame HOLDS, and what the thread DOES. What one
//! ANSWER does to the standing is `worker::fold`, split out on the seam this
//! file already read as (bl-f36e): the loop and its clock here, the folds
//! there.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use serde_json::Value;

mod after;
mod fold;
mod spend;

use super::cmd::Cmd;
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
            // The two that end the loop and the two that are not commands at
            // all. Everything else is one gesture spent, and what one gesture
            // DOES is `worker::spend` — the seam this file already reads as:
            // the loop and its clock here, the folds and the spending there.
            Ok(Cmd::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => return,
            // A lane's frame never comes back from `wait` — it is adopted
            // there, inside the same deadline — so it is the tick's arm.
            Ok(Cmd::Lane(_)) | Err(mpsc::RecvTimeoutError::Timeout) => {}
            Ok(cmd) => note = spend::spend(seat, cmd, &mut focus, &mut standing),
        }
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
