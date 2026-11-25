//! **The loop the worker runs**: present, read, run, answer — and what it
//! does when that stops (bl-8641). Split from the handle in `host.rs` because
//! the two answer different questions: what the frame holds, and what the
//! thread does. The reconnect ruling and the class boundary it stands on are
//! stated there and in [`crate::transport::Wire`]; this file is the ladder.
//!
//! **One loop, one predicate** (REMOTE §5.8, bl-1782): whether the server's
//! hello, read off this channel's own advertisement, states edition 32. Above
//! it the read states `holding` and is held open; each frame's job starts at
//! once on its own thread ([`super::job`]); a heartbeat is a frame with no
//! rows and starts nothing; a read that ends is opened again with no
//! advertisement between, because an accepted advertisement would end the
//! read as superseded. Below it the read is serial and every job runs here,
//! in order, before the next read.
//!
//! **A serial host re-asserts the set at the end of every hand-off** (REMOTE
//! §5.1, bl-cc54, following thrall's bl-2d78). Both of the server's guards
//! over the advertised set stand on this client holding a *parked read*, and
//! a serial loop holds none for the whole runtime of a tool, so another
//! connection bearing this device's certificate may replace the set in that
//! window. Re-presenting bounds the damage to one tool's runtime. **A pushing
//! host does not, and must not**: its read is never closed while it works, so
//! the window does not exist, and presenting while the read is open would end
//! it as superseded (REMOTE §5.8.1).
//!
//! **And since PROTOCOL 8 the re-assertion buys knowing.** The receipt carries
//! `wrote` (yog bl-66d4), so a re-assertion that WROTE is this device being
//! told the set it presents was not the set in force. It is counted onto the
//! standing and painted ([`super::RESTORED`]). **A `true` on a channel's FIRST
//! presentation says nothing** and is discarded: every fresh channel presents
//! into whatever the server happens to hold.

use std::sync::{Arc, mpsc};
use std::time::Duration;

use super::job::{Held, execute};
use super::{Dispatch, Health, Nap, Note, Standing};
use crate::codec::Tool;
use crate::foot::Foot;
use crate::transport::Wire;

/// **The edition at which a server pushes** (REMOTE §5.8.1:
/// `request/invocations/holding` is stamped 32). Below it the server would
/// read `holding` as an unknown key and the read as serial.
const PUSHES: u32 = 32;

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
/// (REMOTE §5.1, adopted from thrall's redial in bl-8bd0) — on a server older
/// than edition 29; a newer one ends the predecessor's read at this host's own
/// advertisement, so the refusal this waits out is never met. A read parked when
/// its connection died does not leave until the server tries to answer it, so
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
    /// **The read was answered `superseded`** (REMOTE §5.1, edition 29,
    /// bl-83d3): a newer activation of this device's certificate advertised
    /// and took the queue. No gesture failed, so it carries no class and
    /// meets no leg — it is drawn from the answer's own key, never from prose.
    Superseded,
    /// A gesture did not land.
    Wire {
        /// Which class failed ([`crate::transport::Wire`]).
        why: Wire,
        /// Whether the gesture in flight was the follow-class read. It is the
        /// LEG and never the server's prose, because a device that decided its
        /// own lifetime by reading sentences would be one the far end could
        /// rewrite by rewording.
        read: bool,
        /// Whether the server answered a read on this channel before it ended.
        /// One answered read — even an empty one — is the server having parked
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
/// is what made a phone's first network flap permanent — the server still held
/// the dead connection's parked read, refused the redial naming this very
/// device, and the foot stopped for good three seconds after a wifi handover.
///
/// **A fourth row is no failure at all** (bl-83d3, thrall's bl-1f02): a read
/// answered `superseded` ends the host with [`super::SUPERSEDED`] and no dial,
/// because a dial opens with an advertisement and that would take the queue
/// back from the activation that replaced this one.
pub(super) fn serve(
    foot: &Arc<Foot>,
    tools: Vec<Tool>,
    run: &Arc<Dispatch>,
    nap: &Nap,
    out: &mpsc::Sender<Note>,
) {
    let mut standing = Standing {
        tools: tools.iter().map(|t| t.name.clone()).collect(),
        ..Standing::default()
    };
    // The first publish's failure is not a stop: a frame that has already
    // gone away is caught by the next one, and the host that presents itself
    // once into a void is the shape the suite pins (a worker that returned
    // here would never dial at all).
    let _ = out.send(Note::Stood(standing.clone()));
    let mut held = Held::new(foot, run, out);
    let mut wait = FIRST;
    loop {
        let (broke, floor, served) = match hold(foot, &tools, run, out, &mut held, &mut standing) {
            Stop::Gone => return,
            Stop::Superseded => {
                standing.health = Health::Stopped(super::SUPERSEDED.to_owned());
                let _ = out.send(Note::Stood(standing));
                return;
            }
            Stop::Wire { why, served, .. } if why.transport() => (why.sentence(), FIRST, served),
            Stop::Wire {
                why: why @ Wire::Refused(_),
                read: true,
                served,
            } => (why.sentence(), PREDECESSOR, served),
            Stop::Wire { why, .. } => {
                standing.health = Health::Stopped(why.sentence());
                let _ = out.send(Note::Stood(standing));
                return;
            }
        };
        // **A channel that ANSWERED A READ starts the ladder over**, which is
        // thrall's rule and not "a channel that was accepted": an answered
        // read is the server having parked this device for its own hold, and
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
        if out.send(Note::Stood(standing.clone())).is_err() {
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
/// three this function can reach.
///
/// **A pushing read is answered `served` by any frame**, a heartbeat
/// included: it is the server holding this device's read, which is the
/// evidence the ladder resets on. A frame that dies quiet for
/// [`Foot`]'s silence bound is a dead connection like any other, and the
/// ladder's redial is the reconnect REMOTE §5.8.1 asks for, `holding` and all.
fn hold(
    foot: &Foot,
    tools: &[Tool],
    run: &Dispatch,
    out: &mpsc::Sender<Note>,
    held: &mut Held,
    standing: &mut Standing,
) -> Stop {
    let mut served = false;
    // The first presentation's reading is discarded on purpose: a fresh
    // channel writes whenever the server held something else, and there is no
    // rival in that. Only the re-assertion below can mean one.
    let edition = match foot.advertise(tools.to_vec()) {
        Ok((_, edition)) => edition,
        Err(why) => {
            return Stop::Wire {
                why,
                read: false,
                served,
            };
        }
    };
    let pushing = edition >= PUSHES;
    standing.advertised = true;
    if out.send(Note::Stood(standing.clone())).is_err() {
        return Stop::Gone;
    }
    loop {
        let mut handed = Vec::new();
        let read = foot.invocations(pushing.then(|| held.list()), &mut |rows| {
            served = true;
            if pushing {
                held.start(rows);
            } else {
                handed.extend(rows);
            }
        });
        match read {
            Ok(false) => {}
            Ok(true) => return Stop::Superseded,
            Err(why) => {
                return Stop::Wire {
                    why,
                    read: true,
                    served,
                };
            }
        }
        for invocation in handed {
            let capture = execute(run, &invocation, out);
            if let Err(why) = foot.complete(invocation.id, capture) {
                return Stop::Wire {
                    why,
                    read: false,
                    served,
                };
            }
            match foot.advertise(tools.to_vec()) {
                Err(why) => {
                    return Stop::Wire {
                        why,
                        read: false,
                        served,
                    };
                }
                Ok((true, _)) => standing.restored += 1,
                Ok((false, _)) => {}
            }
            if out.send(Note::Stood(standing.clone())).is_err() {
                return Stop::Gone;
            }
        }
    }
}
