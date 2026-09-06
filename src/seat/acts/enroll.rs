//! **The mint, posted** (REMOTE §8.4, DESIGN §13.18): enroll the next device
//! in the focused workspace, and hand back the material it answered.
//!
//! **It is the one act in this crate whose ANSWER is the product.** Every
//! other act here earns a receipt and the world is what changed; this one
//! answers a private key for a box that does not exist yet, and the whole
//! gesture is *put it on the glass, let a camera read it, and forget it*. So
//! the material rides back with the outcome rather than being read again —
//! there is no read that could fetch it, because the engine shredded the key
//! the moment it answered.
//!
//! **Never sent twice, and here that is more than an idiom.** A second mint
//! under one name is refused by the engine — the certificate it kept is what
//! refuses it — so a resend on a lost reply would turn a mint whose material
//! was lost into a refusal that says the name is taken. The read that settles
//! it is the workspace's own client roster (§13.14).

use crate::codec::reply::Reply;
use crate::codec::{Act, Gesture, encode};
use crate::envelope::Envelope;
use crate::leaf::Grade;
use crate::seat::pass::kind_err;
use crate::transport::Seat;

use super::super::Focus;
use super::super::posted::{Posted, faulted};

/// **The read that settles a mint in doubt**: the workspace's machines, which
/// list a client the moment its registration exists.
const SETTLES: &str = "The workspace's machines list a client as soon as it is registered — open \
     that screen to see whether the name was taken. Nothing was sent again: a second mint \
     under one name is refused by the certificate the engine kept, so a repeat would \
     report the name as taken rather than minting it.";

/// Post one mint at the focused workspace, and hand back what it answered.
pub(crate) fn enroll(
    seat: &Seat,
    focus: &Focus,
    name: String,
    grade: Grade,
) -> (Posted, Option<Envelope>) {
    let workspace = match super::focused(focus) {
        Ok(workspace) => workspace,
        Err(why) => return (Posted::Refused(format!("enroll: {why}")), None),
    };
    let gesture = Gesture::Act(Act::Enroll {
        workspace,
        name,
        grade,
    });
    match seat.answered(&encode(&gesture)) {
        Ok(Reply::Enrolled(envelope)) => (Posted::Took, Some(envelope)),
        Ok(other) => (Posted::Refused(kind_err("enroll", &other)), None),
        Err(why) => (faulted(&why, "enroll", SETTLES), None),
    }
}
