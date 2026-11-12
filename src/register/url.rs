//! **The registration URL**, read as the three facts a claim needs (yog REMOTE
//! §15.1): `https://<host>[:<port>]/yog/register/<code>`.
//!
//! It is the whole of what travels to a new device — by eye, by paste or as a QR
//! symbol — and it carries no key and no certificate. Read strictly: any
//! other scheme, path or shape is refused before a socket opens, because a
//! claim is single-use and a guess at what the operator meant could spend it.

/// The path every registration URL has ahead of its code.
pub const PATH: &str = "/yog/register/";
/// The port an `https` URL that states none is dialled at.
const HTTPS: u16 = 443;

/// One offer, as its URL names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
    /// The host, without brackets where it is a v6 literal.
    pub host: String,
    pub port: u16,
    /// The one-time code.
    pub code: String,
}

impl Offer {
    /// The host as an address spells it: a v6 literal in its brackets.
    pub fn host(&self) -> String {
        if self.host.contains(':') {
            format!("[{}]", self.host)
        } else {
            self.host.clone()
        }
    }

    /// The `Host` header and the name said back to the operator: the port
    /// only where it is not the scheme's own.
    pub fn authority(&self) -> String {
        if self.port == HTTPS {
            self.host()
        } else {
            format!("{}:{}", self.host(), self.port)
        }
    }

    /// The request path.
    pub fn path(&self) -> String {
        format!("{PATH}{}", self.code)
    }
}

/// Read `url`, or say what a registration URL looks like.
pub fn parse(url: &str) -> Result<Offer, String> {
    read(url.trim()).ok_or_else(|| {
        format!(
            "{url:?} is not a registration URL — one looks like \
             https://<host>{PATH}<code>, exactly as the offer showed it"
        )
    })
}

fn read(url: &str) -> Option<Offer> {
    let (authority, code) = url.strip_prefix("https://")?.split_once(PATH)?;
    let (host, port) = match authority.strip_prefix('[') {
        Some(bracketed) => {
            let (host, tail) = bracketed.split_once(']')?;
            if tail.is_empty() {
                (host, None)
            } else {
                (host, Some(tail.strip_prefix(':')?))
            }
        }
        None => match authority.split_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (authority, None),
        },
    };
    let port = match port {
        Some(stated) => stated.parse().ok()?,
        None => HTTPS,
    };
    let plain = |text: &str, also: &str| {
        !text.is_empty()
            && text
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || also.contains(c))
    };
    (plain(host, ".-:") && plain(code, "")).then(|| Offer {
        host: host.to_owned(),
        port,
        code: code.to_owned(),
    })
}

#[cfg(test)]
mod tests;
