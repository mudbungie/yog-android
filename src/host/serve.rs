//! **The loop the worker runs**: present, wait, run, answer, present again —
//! and what it does when that stops (bl-8641). Split from the handle in
//! `host.rs` because the two answer different questions: what the frame holds,
//! and what the thread does. The reconnect ruling and the class boundary it
//! stands on are stated there and in [`crate::transport::Wire`]; this file is
//! the ladder.
//!
//! **The set is re-asserted at the end of every hand-off** (REMOTE §5.1,
//! bl-cc54, following thrall's bl-2d78). Both of the engine's guards over the
//! advertised set stand on this client holding a *parked read* — the second
//! follow-class read is refused, and an advertisement that would change the
//! set in force is refused while the read is parked — and this loop is serial,
//! so for the whole runtime of a tool this device holds no read at all and
//! neither guard covers it. In that window another connection bearing this
//! device's certificate may replace the set. Re-presenting bounds the damage
//! to one tool's runtime instead of forever, and it costs an idle host
//! nothing: no hand-off, no gesture.
//!
//! **And since PROTOCOL 8 it also buys knowing.** The receipt carries `wrote`
//! (yog bl-66d4), so a re-assertion that WROTE is this device being told the
//! set it presents was not the set in force — a disarming, healed silently
//! until the field existed. It is counted onto the standing and painted, which
//! is the whole remedy ([`super::RESTORED`]). **A `true` on a channel's FIRST
//! presentation says nothing** and is discarded: every fresh channel presents
//! into whatever the engine happens to hold, and an ordinary first one writes.
//! Only a presentation this device made after work it just did can tell a
//! rival from a beginning.

use std::sync::mpsc;
use std::time::Duration;

use super::{Dispatch, Health, Nap, Standing};
use crate::codec::{Capture, Invocation, Tool};
use crate::foot::Foot;
use crate::transport::Wire;

/// The first rest after a channel breaks, and the longest one. A phone that
/// walks out of the house should be back within half a minute of the wifi
/// coming back, and a phone with no network at all should not be dialling in
/// a spin — so the ladder doubles from a second to thirty and stays there,
/// forever. There is no attempt count: a device that changes networks hourly
/// has no number of failures after which giving up is the right answer.
const FIRST: Duration = Duration::from_secs(1);
const LONGEST: Duration = Duration::from_secs(30);

/// The next rest up the ladder.
pub(super) fn climb(wait: Duration) -> Duration {
    (wait * 2).min(LONGEST)
}

/// Why one channel ended.
enum Stop {
    /// The frame stopped reading — nobody is left to publish to, so there is
    /// nothing a new channel could be for.
    Gone,
    /// The wire, in its own two classes: redial a broken channel, stop on a
    /// refusal.
    Wire(Wire),
}

/// Present, then wait, run and answer — and when the channel breaks, climb
/// the ladder and do it again. The only ways out are a refusal that redialling
/// cannot mend and a frame that stopped reading.
pub(super) fn serve(
    foot: &Foot,
    tools: Vec<Tool>,
    run: &Dispatch,
    nap: &Nap,
    out: &mpsc::Sender<Standing>,
) {
    let mut standing = Standing {
        tools: tools.iter().map(|t| t.name.clone()).collect(),
        ..Standing::default()
    };
    // The first publish's failure is not a stop: a frame that has already
    // gone away is caught by the next one, and the host that presents itself
    // once into a void is the shape the suite pins (a worker that returned
    // here would never dial at all).
    let _ = out.send(standing.clone());
    let mut wait = FIRST;
    loop {
        let broke = match hold(foot, tools.clone(), run, out, &mut standing) {
            Stop::Gone => return,
            Stop::Wire(wire) if wire.transport() => wire.sentence(),
            Stop::Wire(wire) => {
                standing.health = Health::Stopped(wire.sentence());
                let _ = out.send(standing);
                return;
            }
        };
        // A channel that got as far as being accepted starts the ladder over:
        // the last dial worked, so the next one has no history to answer for.
        // The presentation goes with the connection that carried it.
        if standing.advertised {
            wait = FIRST;
        }
        standing.advertised = false;
        standing.health = Health::Redialling(broke);
        if out.send(standing.clone()).is_err() {
            return;
        }
        nap(wait);
        wait = climb(wait);
        standing.health = Health::Serving;
    }
}

/// One channel, served. The return is why it ended: there is no success exit,
/// so none is spelled — a host's only way out of this loop is a gesture that
/// failed, and an `Ok` arm here would be one no state of the world can reach.
///
/// **Every wire crossing in this loop is a [`Foot`] method**, which is the
/// bl-2040 narrowing: the three gestures REMOTE §4.2 allows a foot are the
/// three this function can reach, and the general encode-any-gesture door is
/// not in scope here at all.
fn hold(
    foot: &Foot,
    tools: Vec<Tool>,
    run: &Dispatch,
    out: &mpsc::Sender<Standing>,
    standing: &mut Standing,
) -> Stop {
    // The first presentation's reading is discarded on purpose: a fresh
    // channel writes whenever the engine held something else, and there is no
    // rival in that. Only the re-assertion below can mean one.
    if let Err(why) = foot.advertise(tools.clone()) {
        return Stop::Wire(why);
    }
    standing.advertised = true;
    if out.send(standing.clone()).is_err() {
        return Stop::Gone;
    }
    loop {
        let work = match foot.invocations() {
            Ok(work) => work,
            Err(why) => return Stop::Wire(why),
        };
        for invocation in work {
            let capture = match &invocation.cwd {
                None => run(&invocation.tool, &invocation.input),
                Some(cwd) => unconsented(&invocation.tool, cwd),
            };
            standing.served += 1;
            standing.last = Some(format!("{} → {}", invocation.tool, capture.exit_code));
            if let Err(why) = answer(foot, &invocation, capture) {
                return Stop::Wire(why);
            }
            match foot.advertise(tools.clone()) {
                Err(why) => return Stop::Wire(why),
                Ok(true) => standing.restored += 1,
                Ok(false) => {}
            }
            if out.send(standing.clone()).is_err() {
                return Stop::Gone;
            }
        }
    }
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

/// Post one capture back, quoting the handle it answers.
fn answer(foot: &Foot, invocation: &Invocation, capture: Capture) -> Result<(), Wire> {
    foot.complete(invocation.id.clone(), capture)
}
