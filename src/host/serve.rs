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

/// The first rest after a channel breaks, and the floor the series returns to.
/// It is short because the ordinary case is a blip that is over by the time it
/// is noticed — a phone walking back into the house — and because a channel
/// that SERVED starts the ladder again, so this is the rest a healthy phone
/// almost always pays.
const FIRST: Duration = Duration::from_secs(1);

/// The longest rest between dials. A phone with no network at all must settle
/// to a slow cadence rather than burn a core, and there is no attempt count: a
/// device that changes networks hourly has no number of failures after which
/// giving up is the right answer.
///
/// **It is above [`PREDECESSOR`] on purpose** (bl-8bd0, thrall's own constant).
/// The floor below is a fixed wait, so a cap under it would make the ladder
/// inert for the one ending that repeats — a rival permanently holding this
/// device's read would then be dialled every 32 seconds forever, which on a
/// pocketed phone is a handshake and a radio wake a minute for as long as the
/// battery lasts. Above it, the series climbs past the floor and that case
/// settles at a minute like any other.
const LONGEST: Duration = Duration::from_secs(64);

/// **How long a vanished predecessor of THIS device can still hold its claim**
/// (REMOTE §5.1, adopted from thrall's redial in bl-8bd0). A read parked when
/// its connection died does not leave until the engine tries to answer it, so
/// a redial inside that window is refused naming this very device — the stale
/// predecessor, not a rival. REMOTE states the bound as a contract rather than
/// an accident: *"Its life is the hold and not the connection's ...
/// `Mailbox::take` drops the claim on the way out, before the caller writes
/// the answer, so a peer that vanished without a FIN frees the slot within one
/// hold's width — thirty seconds."* Two seconds over the stated width, because
/// this end's window began before it noticed the drop.
const PREDECESSOR: Duration = Duration::from_secs(32);

/// The next rest up the ladder.
pub(super) fn climb(wait: Duration) -> Duration {
    (wait * 2).min(LONGEST)
}

/// Why one channel ended, and the two facts the ladder reads off it.
enum Stop {
    /// The frame stopped reading — nobody is left to publish to, so there is
    /// nothing a new channel could be for.
    Gone,
    /// A gesture did not land.
    Wire {
        /// Which class failed ([`crate::transport::Wire`]).
        why: Wire,
        /// Whether the gesture in flight was the follow-class read. It is the
        /// LEG and never the engine's prose, because a device that decided its
        /// own lifetime by reading sentences would be one the far end could
        /// rewrite by rewording.
        read: bool,
        /// Whether the engine answered a read on this channel before it ended.
        /// One answered read — even an empty one — is the engine having parked
        /// this device for its own hold, which is the evidence that the
        /// channel was real, and a hammering loop cannot manufacture it.
        served: bool,
    },
}

/// Present, then wait, run and answer — and when the channel breaks, climb
/// the ladder and do it again. The only ways out are an ending no redial can
/// mend and a frame that stopped reading.
///
/// **The matrix is three rows and it is thrall's** (bl-8bd0, its bl-916d):
/// the wire is always worth another dial; a refusal of this device's READ is
/// REMOTE §5.1's one-reader guard naming a predecessor whose claim is already
/// expiring, so it waits one hold's width and asks again; every other refusal
/// and every unusable answer ends the host. Taking the read's refusal as final
/// is what made a phone's first network flap permanent — the engine still held
/// the dead connection's parked read, refused the redial naming this very
/// device, and the foot stopped for good three seconds after a wifi handover.
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
        let (broke, floor, served) = match hold(foot, tools.clone(), run, out, &mut standing) {
            Stop::Gone => return,
            Stop::Wire { why, served, .. } if why.transport() => (why.sentence(), FIRST, served),
            Stop::Wire {
                why: why @ Wire::Refused(_),
                read: true,
                served,
            } => (why.sentence(), PREDECESSOR, served),
            Stop::Wire { why, .. } => {
                standing.health = Health::Stopped(why.sentence());
                let _ = out.send(standing);
                return;
            }
        };
        // **A channel that ANSWERED A READ starts the ladder over**, which is
        // thrall's rule and not "a channel that was accepted": an answered
        // read is the engine having parked this device for its own hold, and
        // an accepted advertisement is not — a rival holding this device's
        // read refuses every read while accepting every advertisement, so the
        // weaker predicate would reset the ladder forever on exactly the
        // ending that must back off. A phone that walks back into the house
        // is still back within a second of the wifi, because the channel it
        // lost had been served.
        if served {
            wait = FIRST;
        }
        standing.advertised = false;
        standing.health = Health::Redialling(broke);
        if out.send(standing.clone()).is_err() {
            return;
        }
        nap(wait.max(floor));
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
    let mut served = false;
    // The first presentation's reading is discarded on purpose: a fresh
    // channel writes whenever the engine held something else, and there is no
    // rival in that. Only the re-assertion below can mean one.
    if let Err(why) = foot.advertise(tools.clone()) {
        return Stop::Wire {
            why,
            read: false,
            served,
        };
    }
    standing.advertised = true;
    if out.send(standing.clone()).is_err() {
        return Stop::Gone;
    }
    loop {
        let work = match foot.invocations() {
            Ok(work) => work,
            Err(why) => {
                return Stop::Wire {
                    why,
                    read: true,
                    served,
                };
            }
        };
        served = true;
        for invocation in work {
            let capture = match &invocation.cwd {
                None => run(&invocation.tool, &invocation.input),
                Some(cwd) => unconsented(&invocation.tool, cwd),
            };
            standing.served += 1;
            standing.last = Some(format!("{} → {}", invocation.tool, capture.exit_code));
            if let Err(why) = answer(foot, &invocation, capture) {
                return Stop::Wire {
                    why,
                    read: false,
                    served,
                };
            }
            match foot.advertise(tools.clone()) {
                Err(why) => {
                    return Stop::Wire {
                        why,
                        read: false,
                        served,
                    };
                }
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

/// Post one capture back, quoting the handle it answers. **A lost one is not
/// re-posted and the next channel does not carry it** — the contract and its
/// recovery are on [`Foot::complete`] (yog REMOTE §3, bl-07b1). What the
/// redial re-asserts is the presentation, which is idempotent by design and
/// says so in its own receipt: two of the three gestures may be repeated, and
/// the one that may not is this one.
fn answer(foot: &Foot, invocation: &Invocation, capture: Capture) -> Result<(), Wire> {
    foot.complete(invocation.id.clone(), capture)
}
