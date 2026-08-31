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
    /// and a socket that never opened alike: all three are the same fact to a
    /// caller — this cannot be painted, and here is the sentence.
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
    pub fn answered(&self, request: &Value) -> Result<Reply, String> {
        let stream = self.ask(request)?;
        let last = stream
            .last()
            .ok_or_else(|| "the engine ended the stream without answering".to_owned())?;
        reply::decode(last).unwrap_or_else(Err)
    }

    /// Send one request envelope and read its whole reply stream — every
    /// frame up to the terminator. A stream of one is the ordinary answer.
    pub fn ask(&self, request: &Value) -> Result<Vec<Value>, String> {
        let mut tls = self.dial(request)?;
        // The engine's half of the §3 preface, read on the way to the answer:
        // a skew refuses here, before a frame of another protocol is decoded.
        hello::confirm(&mut tls)?;
        let mut stream = Vec::new();
        loop {
            let frame = frame::read_frame(&mut tls).map_err(|e| format!("receive: {e}"))?;
            match frame {
                Some(body) => stream.push(parsed(&body)?),
                None => return Ok(stream),
            }
        }
    }

    /// Connect, handshake and send this end's whole half of the exchange — the
    /// handshake happens inside the first write, so what this hands back is a
    /// socket with a preface and an envelope on it and nothing yet read.
    fn dial(&self, request: &Value) -> Result<StreamOwned<ClientConnection, TcpStream>, String> {
        let tcp = TcpStream::connect(&self.address)
            .map_err(|e| format!("connect {}: {e}", self.address))?;
        // A timeout that failed to arm costs a slow failure, never a wrong
        // one — and Some(nonzero) cannot be refused, so an error arm here
        // would be an untestable branch.
        let _ = tcp.set_read_timeout(Some(ASK_TIMEOUT));
        let conn = ClientConnection::new(Arc::clone(&self.config), self.name.clone())
            .map_err(|e| format!("tls {}: {e}", self.address))?;
        let mut tls = StreamOwned::new(conn, tcp);
        send(&mut tls, request).map_err(|e| format!("send: {e}"))?;
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
/// discipline at the framing, said in the frame's own terms.
fn parsed(body: &[u8]) -> Result<Value, String> {
    serde_json::from_slice(body).map_err(|e| format!("receive: frame is not JSON: {e}"))
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
