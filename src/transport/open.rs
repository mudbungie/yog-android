//! **A connection with its request on it**: the frames left to read, the
//! edition the server stated, and the handle that ends the read from another
//! thread.

use super::Wire;
use crate::frame;
use rustls::{ClientConnection, StreamOwned};
use serde_json::Value;
use std::net::TcpStream;

/// A connection with its request written and the preface confirmed: the
/// answer's frames are what is left on it.
pub struct Open {
    tls: StreamOwned<ClientConnection, TcpStream>,
    /// **What the server on the other end can spell** (REMOTE §3.2): the
    /// edition its preface stated, or `ledger::FLOOR` where it stated none.
    /// Kept on the connection rather than in a process-wide slot, because a
    /// capability question must not depend on what else has been dialled
    /// since — see `crate::ledger`.
    edition: u32,
}

/// **The way to end a held read from another thread.** A reader parked on
/// the socket wakes when the socket is shut down under it, so a lane is
/// stopped by hanging up rather than by a flag it would only read between
/// frames — up to a hold away.
pub struct Hangup {
    tcp: TcpStream,
}

impl Open {
    pub(super) fn new(tls: StreamOwned<ClientConnection, TcpStream>, edition: u32) -> Open {
        Open { tls, edition }
    }

    /// **The edition the server stated**, for whoever asks
    /// `crate::ledger::spells` — the one question a control has that no
    /// protocol bump ever answered: *could this server have said that field
    /// at all* (REMOTE §3.2).
    #[must_use]
    pub fn edition(&self) -> u32 {
        self.edition
    }

    /// Read every frame up to the terminator, handing each to `adopt` as it
    /// lands. `adopt` answering `false` ends the read here — the connection
    /// is dropped, which is how the server learns its answer has no reader.
    /// A socket that ends without the terminator is a lost stream (REMOTE
    /// §10: a stream that ended and a dial that failed are one case).
    pub fn each(mut self, adopt: &mut dyn FnMut(Value) -> bool) -> Result<(), Wire> {
        loop {
            // **Lost and not `Transport`** (bl-07b1): the gesture is on the
            // wire by the time this reads, so a channel that dies here is yog
            // REMOTE §3's lost reply — the server may have completed the act.
            // The channel question is unchanged (`Wire::transport` answers yes
            // to both), so the tool host's ladder reads exactly what it read.
            let frame = frame::read_frame(&mut self.tls)
                .map_err(|e| Wire::Lost(format!("receive: {e}")))?;
            let Some(body) = frame else {
                return Ok(());
            };
            if !adopt(parsed(&body)?) {
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
    /// error: the reader it was for has nothing left to be woken from.
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
