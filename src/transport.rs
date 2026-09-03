//! The seat's transport — the mirror of the server-side client
//! (`wire/client.rs`; yog REMOTE §8, §9.5): what a client of the engine
//! holds, and the only thing it holds.
//!
//! A client owns its key material and RAM, nothing else (REMOTE §6) — so
//! this is a configuration and an address, and **every ask is its own TCP
//! connection and its own handshake**: "the seat polls", at human cadence,
//! exactly the shape the upstream ruling keeps (REMOTE §10 holds the held
//! connection open as a question, not a plan).
//!
//! **The server's name comes from the address, never from a second knob.** A
//! dotted quad or a bracketed v6 literal is verified as an IP address — the
//! server leaf must carry the matching `IP:` SAN — and anything else is a
//! DNS name. Nothing to configure, nothing that can disagree with what was
//! dialled.

use crate::codec::reply::{self, Reply};
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

/// **Why a gesture did not come back, and which end of the wire is why**
/// (bl-8641). Two classes, because exactly one caller distinction rides on
/// them: a channel that broke is worth dialling again, and a far end that
/// said no is not — the tool host redials the first forever and stops dead on
/// the second. Every caller that does not care converts to the sentence and
/// is unchanged (`From<Wire> for String`), which is what the seat model does:
/// it opens a connection per ask anyway, so a broken channel is already
/// re-dialled by its next pass and the class buys it nothing.
///
/// The line is drawn where this file already knows it — at the socket, not by
/// reading sentences back. **The version preface is on the far side of it**:
/// REMOTE §3 collapses a peer that hung up mid-preface into "the peer speaks
/// no version" by ruling, so a channel that dies inside that one window
/// stops the host rather than redialling. Narrowing that would be amending
/// REMOTE from a client, which §1 forbids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Wire {
    /// The channel: a socket that would not open, a handshake that would not
    /// build, a write that did not land, a read that died. The class a phone
    /// meets every time it changes networks.
    Transport(String),
    /// **The engine spoke, and what it said is no** — its own refusal,
    /// written as a sentence for an operator. Whether that is worth asking
    /// again is the LEG's answer and not this one's (`host::serve`, following
    /// thrall's bl-916d): a refusal of this device's *read* is REMOTE §5.1's
    /// one-reader guard, which after a drop names this very device.
    Refused(String),
    /// **An answer this end cannot use**: a stream that ended without one, a
    /// frame that is not JSON, a reply of a kind the gesture does not earn, a
    /// version that cannot be spoken to. Asking again asks the same question
    /// and gets the same answer, on every leg, so this class always stops.
    Unusable(String),
}

impl Wire {
    /// The sentence, for the frame that paints it and the caller that wanted
    /// nothing else.
    pub fn sentence(&self) -> String {
        match self {
            Self::Transport(said) | Self::Refused(said) | Self::Unusable(said) => said.clone(),
        }
    }

    /// Whether the channel is what failed — the one question a caller that
    /// can dial again asks.
    pub fn transport(&self) -> bool {
        matches!(self, Self::Transport(_))
    }
}

impl From<Wire> for String {
    fn from(wire: Wire) -> Self {
        wire.sentence()
    }
}

/// A seat's end of the wire.
pub struct Seat {
    config: Arc<rustls::ClientConfig>,
    address: String,
    name: ServerName<'static>,
}

impl Seat {
    /// Build the seat from provisioned material. Nothing is dialled here: a
    /// seat is a fact about what this device may say, not about whether an
    /// engine happens to be up.
    pub fn open(m: &Material) -> Result<Self, String> {
        Ok(Self {
            config: crate::tls::client_config(m)?,
            address: m.address.clone(),
            name: server_name(&m.address)?,
        })
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
        let last = stream.last().ok_or_else(|| {
            Wire::Unusable("the engine ended the stream without answering".to_owned())
        })?;
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
        let mut tls = self.dial(request)?;
        // The engine's half of the §3 preface, read on the way to the answer:
        // a skew refuses here, before a frame of another protocol is decoded.
        hello::confirm(&mut tls).map_err(Wire::Unusable)?;
        let mut stream = Vec::new();
        loop {
            let frame = frame::read_frame(&mut tls)
                .map_err(|e| Wire::Transport(format!("receive: {e}")))?;
            match frame {
                Some(body) => stream.push(parsed(&body)?),
                None => return Ok(stream),
            }
        }
    }

    /// Connect, handshake and send this end's whole half of the exchange — the
    /// handshake happens inside the first write, so what this hands back is a
    /// socket with a preface and an envelope on it and nothing yet read.
    fn dial(&self, request: &Value) -> Result<StreamOwned<ClientConnection, TcpStream>, Wire> {
        let tcp = TcpStream::connect(&self.address)
            .map_err(|e| Wire::Transport(format!("connect {}: {e}", self.address)))?;
        // A timeout that failed to arm costs a slow failure, never a wrong
        // one — and Some(nonzero) cannot be refused, so an error arm here
        // would be an untestable branch.
        let _ = tcp.set_read_timeout(Some(ASK_TIMEOUT));
        let conn = ClientConnection::new(Arc::clone(&self.config), self.name.clone())
            .map_err(|e| Wire::Transport(format!("tls {}: {e}", self.address)))?;
        let mut tls = StreamOwned::new(conn, tcp);
        send(&mut tls, request).map_err(|e| Wire::Transport(format!("send: {e}")))?;
        Ok(tls)
    }
}

/// This end's two frames, written in one breath (REMOTE §3): the version
/// preface, then the gesture envelope. One fallible unit because they are one
/// act to a caller — a connection that could not carry the preface could not
/// have carried the request either, and two sentences for that would be two
/// spellings of "the socket went away".
fn send(w: &mut dyn std::io::Write, request: &Value) -> std::io::Result<()> {
    hello::state(w)?;
    frame::write_frame(w, request.to_string().as_bytes())
}

/// One frame's bytes as the JSON value the codec reads — the strict-decode
/// discipline at the framing, said in the frame's own terms. Unusable and not
/// a channel failure: the bytes arrived intact and said something this end
/// cannot read, which dialling again cannot mend.
fn parsed(body: &[u8]) -> Result<Value, Wire> {
    serde_json::from_slice(body)
        .map_err(|e| Wire::Unusable(format!("receive: frame is not JSON: {e}")))
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
