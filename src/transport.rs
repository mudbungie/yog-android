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
use crate::frame;
use crate::material::Material;
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
        let mut stream = Vec::new();
        loop {
            let frame = frame::read_frame(&mut tls).map_err(|e| format!("receive: {e}"))?;
            match frame {
                Some(body) => stream.push(parsed(&body)?),
                None => return Ok(stream),
            }
        }
    }

    /// Connect, handshake and send `request` — the handshake happens inside
    /// the first read, so what this hands back is a socket with an envelope
    /// on it and nothing yet read.
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
        frame::write_frame(&mut tls, request.to_string().as_bytes())
            .map_err(|e| format!("send: {e}"))?;
        Ok(tls)
    }
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
