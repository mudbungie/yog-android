//! **The sign-in, posted** (REMOTE §8.3, DESIGN §13.19): start `bz --login`
//! for one provider row inside the focused workspace's wall, on the engine,
//! and hand back the run's standing it answered with.
//!
//! **Its receipt is the lane's first frame** — the same value at the same
//! moment (yog's own `boundary::login`) — so the answer is adopted rather
//! than reported: the act opens the tail as well as the run, and a receipt
//! discarded here would blank a screen for one cadence and then fill it with
//! the very lines it had thrown away.
//!
//! **A refusal is the receipt too.** An unsigned wall, a row with no login
//! flow, a workspace this seat may not reach: each crosses as the act's own
//! `ok: false` and lands in the banner where every other refusal does. There
//! is nothing else to paint — a run that never started has said nothing.
//!
//! **Never sent twice, and the restart is the cancel.** Firing it again on a
//! row already signing in TERMINATES that run and starts a fresh one
//! upstream, so a resend on a lost reply would kill a browser flow the
//! operator may be halfway through. The read that settles a doubted one is
//! this act's own lane: the tail says whether a run is going.

use crate::codec::reply::Reply;
use crate::codec::{Act, Gesture, LoginView, encode};
use crate::seat::pass::kind_err;
use crate::transport::Seat;

use super::super::Focus;
use super::super::posted::{Posted, faulted};

/// **The read that settles a sign-in in doubt**: the tail this act's own
/// screen holds open, which says whether a run is going.
const SETTLES: &str = "The sign-in's own tail says whether a run started — it is held open on \
     this screen. Nothing was sent again: firing it a second time terminates the run that may \
     already be going, which would end a browser flow midway.";

/// Post one sign-in at the focused workspace, and hand back what it answered.
pub(crate) fn login(seat: &Seat, focus: &Focus, provider: String) -> (Posted, Option<LoginView>) {
    let workspace = match super::focused(focus) {
        Ok(workspace) => workspace,
        Err(why) => return (Posted::Refused(format!("login: {why}")), None),
    };
    let gesture = Gesture::Act(Act::Login {
        workspace,
        provider,
    });
    match seat.answered(&encode(&gesture)) {
        Ok(Reply::Login(view)) => (Posted::Took, Some(view)),
        Ok(other) => (Posted::Refused(kind_err("login", &other)), None),
        Err(why) => (faulted(&why, "login", SETTLES), None),
    }
}
