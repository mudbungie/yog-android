//! The seat's transport — the mirror of the server-side client
//! (`wire/client.rs`; yog REMOTE §8, §9.5): what a client of the engine
//! holds, and the only thing it holds.
//!
//! A client owns its key material and RAM, nothing else (REMOTE §6) — so
//! this is a configuration and an address, and **every ask is its own TCP
//! connection and its own handshake**: "the seat polls", at human cadence.
//! A follow-class read is the same connection kept open for as long as the
//! engine holds it (REMOTE §3: the streaming form is not a second form):
//! [`Seat::hold`] is the one door every ask goes through, and [`Seat::ask`]
//! is that door with every frame collected — so a held lane and a one-shot
//! answer differ only in who reads the frames and when.
//!
//! **The server's name comes from the address, never from a second knob.** A
//! dotted quad or a bracketed v6 literal is verified as an IP address — the
//! server leaf must carry the matching `IP:` SAN — and anything else is a
//! DNS name. Nothing to configure, nothing that can disagree with what was
//! dialled.
//!
//! **Since the punched wire (DESIGN §21) the socket comes from the
//! [`Ladder`]**, not from `TcpStream::connect` — the same four-rung climb
//! REMOTE §13.4 gives every client — and "every ask is its own connection"
//! holds for a dialled socket exactly as before while a PUNCHED one is held:
//! its preface is spent once, a finished ask hands it back
//! ([`Open::each`](open::Open::each)), and the next ask writes only its
//! request. The engine serves a held stream request after request and pings
//! it through its silence; the ping is discarded ahead of every answer.

mod open;
mod wire;

pub use open::{Hangup, Open};
pub use wire::Wire;

use crate::codec::reply::{self, Reply};
use crate::ladder::{Clock, Conn, Ladder, Rove, SystemClock};
use crate::material::Material;
use crate::{frame, hello};
use rustls::pki_types::ServerName;
use rustls::{ClientConnection, StreamOwned};
use serde_json::Value;
use std::net::{IpAddr, TcpStream};
use std::sync::Arc;
use std::time::Duration;

/// How long a seat waits on one answer before giving up on the connection.
const ASK_TIMEOUT: Duration = Duration::from_mins(2);

/// **What a stream that carried no frame at all says.** One home, because two
/// readers ask the same question of one shape: `Seat::answered` here, and
/// `seat::pass::ask::wired`, which wants the envelope beside the reply and so
/// cannot go through it (bl-eec1).
pub(crate) const NO_ANSWER: &str = "the engine ended the stream without answering";

/// A seat's end of the wire.
pub struct Seat {
    config: Arc<rustls::ClientConfig>,
    address: String,
    name: ServerName<'static>,
    ladder: Arc<Ladder>,
}

impl Seat {
    /// Build the seat from provisioned material. Nothing is dialled here: a
    /// seat is a fact about what this device may say, not about whether an
    /// engine happens to be up. Material that roves climbs the mainline.
    pub fn open(m: &Material) -> Result<Self, String> {
        let rove = m.pairing.clone().map(Rove::mainline);
        Self::open_with(m, rove, Arc::new(SystemClock))
    }

    /// [`open`](Self::open) with the commons and the clock named — the
    /// suite's door, pointing the ladder at a fake DHT and a clock it turns.
    pub fn open_with(
        m: &Material,
        rove: Option<Rove>,
        clock: Arc<dyn Clock>,
    ) -> Result<Self, String> {
        Ok(Self {
            config: crate::tls::client_config(m)?,
            address: m.address.clone(),
            name: server_name(&m.address)?,
            ladder: Arc::new(Ladder::new(m.address.clone(), rove, clock)),
        })
    }

    /// How many punched streams this seat is holding for its next asks.
    pub fn held(&self) -> usize {
        self.ladder.held()
    }

    /// The address this seat dials.
    pub fn address(&self) -> String {
        self.address.clone()
    }

    /// Ask once and decode the answer — the last frame of the stream, which
    /// today is the only frame. One `Err` for a refusal, an unreadable answer
    /// and a socket that never opened alike: the same fact to a caller that
    /// only paints it, carrying [`Wire`]'s class for the one that redials.
    ///
    /// **`last()` is not the door to the follow lane** (REMOTE §5.5, bl-2842).
    /// Since yog bl-3655 a `Query::Follow` frame carries what landed *since
    /// the previous frame*, not the whole answer, and the rule is one line
    /// with no flag: *"Absorb every frame of a read, in order, onto an empty
    /// fold. What you hold after the last frame you have received is what you
    /// paint."* Taking the last frame of such a stream paints the final delta
    /// alone and calls it the answer. Nothing here follows — this seat
    /// re-reads the transcript at the model's cadence — so no test can catch
    /// it, which is exactly why the trap is named at the `last()` rather than
    /// in a document the author will not be reading. The lane's door is
    /// [`ask`](Self::ask), which hands back every frame; the fold goes on top
    /// of it.
    pub fn answered(&self, request: &Value) -> Result<Reply, Wire> {
        let stream = self.ask(request)?;
        let last = stream
            .last()
            .ok_or_else(|| Wire::Unusable(NO_ANSWER.to_owned()))?;
        // **The decoder already draws this line and it was being collapsed**
        // (bl-8bd0): its OUTER error is a reply this end cannot read, its
        // INNER one is the engine's own `ok: false` sentence. They are two
        // different facts about who failed, and the redial matrix needs them
        // apart — a refusal on the follow read is worth another dial and an
        // unreadable answer never is.
        match reply::decode(last) {
            Err(unreadable) => Err(Wire::Unusable(unreadable)),
            Ok(Err(refusal)) => Err(Wire::Refused(refusal)),
            Ok(Ok(reply)) => Ok(reply),
        }
    }

    /// Send one request envelope and read its whole reply stream — every
    /// frame up to the terminator. A stream of one is the ordinary answer.
    pub fn ask(&self, request: &Value) -> Result<Vec<Value>, Wire> {
        let mut stream = Vec::new();
        let (open, _hangup) = self.hold(request)?;
        open.each(&mut |frame| {
            stream.push(frame);
            true
        })?;
        Ok(stream)
    }

    /// **Open one held read** (DESIGN §14.1): the request written and the
    /// engine's preface confirmed, with no frame read yet — and the handle
    /// that ends it from another thread. The caller reads the frames on
    /// whatever thread it likes and hangs up from any other, which is the
    /// whole of what a lane needs and one-shot asks do not.
    pub fn hold(&self, request: &Value) -> Result<(Open, Hangup), Wire> {
        let (mut tls, hangup, spent, punched) = self.dial(request)?;
        // The engine's half of the §3 preface, read on the way to the answer:
        // a skew refuses here, before a frame of another protocol is decoded.
        // A held stream spent its preface on its first ask and carries the
        // edition it was told then.
        let edition = match spent {
            Some(edition) => edition,
            None => hello::confirm(&mut tls).map_err(Wire::Unusable)?,
        };
        let keep = punched.then(|| Arc::clone(&self.ladder));
        Ok((Open::new(tls, edition, keep), hangup))
    }

    /// The connection with the request written, and the hang-up handle on it
    /// — a second descriptor on the same socket, taken here so that a clone
    /// that cannot be had is the same sentence as a socket that would not
    /// open: both are the channel failing before a byte of the act left. The
    /// third answer is the edition a HELD stream already carries, `None` for
    /// a fresh socket whose preface is still to be read; the fourth is
    /// whether the socket was punched, which is what a finished ask keeps.
    fn dial(&self, request: &Value) -> Result<Dialled, Wire> {
        let conn = self.ladder.connect().map_err(Wire::Transport)?;
        let (tcp, punched) = match conn {
            Conn::Held(held) => {
                let mut tls = held.tls;
                let hangup = tls
                    .sock
                    .try_clone()
                    .map_err(|e| Wire::Transport(format!("connect {}: {e}", self.address)))?;
                let _ = tls.sock.set_read_timeout(Some(ASK_TIMEOUT));
                frame::write_frame(&mut tls, request.to_string().as_bytes())
                    .map_err(|e| Wire::Transport(format!("send: {e}")))?;
                return Ok((tls, Hangup::new(hangup), Some(held.edition), true));
            }
            Conn::Fresh { tcp, punched } => (tcp, punched),
        };
        let (tcp, hangup) = tcp
            .try_clone()
            .map(|clone| (clone, tcp))
            .map_err(|e| Wire::Transport(format!("connect {}: {e}", self.address)))?;
        // A timeout that failed to arm costs a slow failure, never a wrong
        // one — and Some(nonzero) cannot be refused, so an error arm here
        // would be an untestable branch.
        let _ = tcp.set_read_timeout(Some(ASK_TIMEOUT));
        let conn = ClientConnection::new(Arc::clone(&self.config), self.name.clone())
            .map_err(|e| Wire::Transport(format!("tls {}: {e}", self.address)))?;
        let mut tls = StreamOwned::new(conn, tcp);
        // **A write that failed is not in doubt, and the framing is why**
        // (bl-07b1). An `io::Error` out of a write is that write's bytes not
        // being accepted, so a frame that failed mid-write is a frame the
        // engine never received whole — and `frame`'s reader takes a length
        // header and then exactly that many bytes, never scanning, so an
        // incomplete frame is not decoded, never mind executed. The doubt
        // therefore begins where the write ENDS, which is why the class here
        // is the same one a socket that would not open earns.
        send(&mut tls, request).map_err(|e| Wire::Transport(format!("send: {e}")))?;
        Ok((tls, Hangup::new(hangup), None, punched))
    }
}

/// What [`Seat::dial`] hands up: the stream with the request on it, the
/// hang-up handle, the edition if the preface is already spent, and whether
/// the socket was punched.
type Dialled = (
    StreamOwned<ClientConnection, TcpStream>,
    Hangup,
    Option<u32>,
    bool,
);

/// This end's two frames, written in one breath (REMOTE §3): the version
/// preface, then the gesture envelope. One fallible unit because they are one
/// act to a caller — a connection that could not carry the preface could not
/// have carried the request either, and two sentences for that would be two
/// spellings of "the socket went away".
fn send(w: &mut dyn std::io::Write, request: &Value) -> std::io::Result<()> {
    hello::state(w)?;
    frame::write_frame(w, request.to_string().as_bytes())
}

/// The name to verify the server certificate against, read off the address.
fn server_name(address: &str) -> Result<ServerName<'static>, String> {
    let host = address.rsplit_once(':').map_or(address, |(head, _)| head);
    let host = host.trim_start_matches('[').trim_end_matches(']');
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(ServerName::IpAddress(ip.into()));
    }
    ServerName::try_from(host.to_owned()).map_err(|e| format!("{address}: not a server name: {e}"))
}

#[cfg(test)]
mod tests;
