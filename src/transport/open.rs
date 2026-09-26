//! **A connection with its request on it** (split from `transport.rs` when
//! the punched wire gave a stream a life after its answer, DESIGN §21): the
//! frames left to read, the edition the engine stated, and the handle that
//! ends the read from another thread.

use super::Wire;
use crate::frame;
use crate::ladder::{Held, Ladder};
use rustls::{ClientConnection, StreamOwned};
use serde_json::Value;
use std::net::TcpStream;
use std::sync::Arc;

/// A connection with its request written and the preface confirmed: the
/// answer's frames are what is left on it.
pub struct Open {
    tls: StreamOwned<ClientConnection, TcpStream>,
    /// **What the engine on the other end can spell** (REMOTE §3.2): the
    /// edition its preface stated, or `ledger::FLOOR` where it stated none.
    /// Kept on the connection rather than in a process-wide slot, because a
    /// capability question must not depend on what else has been dialled
    /// since — see `crate::ledger`.
    edition: u32,
    /// The ladder a PUNCHED stream goes back to when its answer ends clean
    /// (REMOTE §13.4: held and reused); `None` for a dialled socket, which is
    /// dropped as it always was.
    keep: Option<Arc<Ladder>>,
}

/// **The way to end a held read from another thread.** A reader parked on
/// the socket wakes when the socket is shut down under it, so a lane is
/// stopped by hanging up rather than by a flag it would only read between
/// frames — up to a hold away.
pub struct Hangup {
    tcp: TcpStream,
}

impl Open {
    pub(super) fn new(
        tls: StreamOwned<ClientConnection, TcpStream>,
        edition: u32,
        keep: Option<Arc<Ladder>>,
    ) -> Open {
        Open { tls, edition, keep }
    }

    /// **The edition the engine stated**, for whoever asks
    /// `crate::ledger::spells` — the one question a control has that no
    /// protocol bump ever answered: *could this engine have said that field
    /// at all* (REMOTE §3.2).
    #[must_use]
    pub fn edition(&self) -> u32 {
        self.edition
    }

    /// Read every frame up to the terminator, handing each to `adopt` as it
    /// lands. `adopt` answering `false` ends the read here — the connection
    /// is dropped, which is how the engine learns its answer has no reader.
    /// A socket that ends without the terminator is a lost stream (REMOTE
    /// §10: a stream that ended and a dial that failed are one case).
    ///
    /// **A ping is never a frame of the answer** (REMOTE §13.4): one the
    /// engine wrote into the silence just before the request landed sits in
    /// the buffer ahead of the reply, and is dropped here. And a punched
    /// stream whose answer ended with the terminator goes back to the ladder
    /// for the next ask — only then, because a stream left mid-answer is not
    /// one the next request could be written into.
    pub fn each(mut self, adopt: &mut dyn FnMut(Value) -> bool) -> Result<(), Wire> {
        loop {
            // **Lost and not `Transport`** (bl-07b1): the gesture is on the
            // wire by the time this reads, so a channel that dies here is yog
            // REMOTE §3's lost reply — the engine may have completed the act.
            // The channel question is unchanged (`Wire::transport` answers yes
            // to both), so the tool host's ladder reads exactly what it read.
            let frame = frame::read_frame(&mut self.tls)
                .map_err(|e| Wire::Lost(format!("receive: {e}")))?;
            let Some(body) = frame else {
                if let Some(ladder) = self.keep.take() {
                    ladder.keep(Held {
                        tls: self.tls,
                        edition: self.edition,
                    });
                }
                return Ok(());
            };
            let value = parsed(&body)?;
            if crate::ladder::held::is_ping(&value) {
                continue;
            }
            if !adopt(value) {
                return Ok(());
            }
        }
    }
}

impl Hangup {
    pub(super) fn new(tcp: TcpStream) -> Hangup {
        Hangup { tcp }
    }

    /// End the held read. Idempotent, and a socket already gone is not an
    /// error — the reader it was for has nothing left to be woken from.
    pub fn hang_up(&self) {
        let _ = self.tcp.shutdown(std::net::Shutdown::Both);
    }
}

/// One frame's bytes as the JSON value the codec reads — the strict-decode
/// discipline at the framing, said in the frame's own terms. Unusable and not
/// a channel failure: the bytes arrived intact and said something this end
/// cannot read, which dialling again cannot mend.
fn parsed(body: &[u8]) -> Result<Value, Wire> {
    serde_json::from_slice(body)
        .map_err(|e| Wire::Unusable(format!("receive: frame is not JSON: {e}")))
}
