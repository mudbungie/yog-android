//! **What a sign-in's tail line carries, and which door it is** (DESIGN §24
//! item 5, amending §13.19's painting). `bz --login` runs on the SERVER and
//! streams its human-facing flow back as lines (REMOTE §8.3): *open this
//! address*, *enter this code*, a failure's reason. A desktop can follow the
//! address and select the code; a phone can do neither (§13.2: no selection
//! on the glass), so each line is read here for the two things a thumb needs
//! made into doors, and `shell/screens/world/signin.rs` paints what this
//! file decides.
//!
//! **Two doors, one reading.** A `https://` token is a door that opens the
//! phone's browser — the operator finishes the flow there, bz polls the token
//! endpoint from the server, and nothing credential-shaped ever reaches this
//! device. A token of the device-code shape ([`CODE`]) earns a `copy <code>`
//! door at the line's end, because the browser page asks for it and a code
//! that can only be read must be typed. A line with neither stays weak prose.
//!
//! **Only `https://`.** A sign-in that hands out a plaintext address is not
//! one this seat will open — `crate::update` refuses a release offered over
//! plaintext for the same reason — so an `http://` token stays prose and
//! reads as what it is.
//!
//! **Which box is signed in is the row's name** (REMOTE §5.7): a row on
//! another machine runs its sign-in there, and the three sentences that say
//! where — [`fact`], [`starts`], [`browser`] — take the machine the name was
//! read as (`codec::pick::machine`).
//!
//! Pure and host-tested: every word on the painted door is spelled here.

use crate::codec::pick::machine::Machine;

#[cfg(test)]
mod tests;

/// **The device-code shape, as a template**: four capitals or digits, a
/// hyphen, four more — the shape every device flow prints (GitHub, Google,
/// the claude-code preset's). The template is also the matcher
/// ([`is_code`]), so the shape has one home. Capitals and not letters: a
/// lowercase `open-this` in prose is not a code, and no device flow prints
/// one in lowercase.
pub const CODE: &str = "AAAA-0000";

/// The scheme a door opens; anything else is prose (module doc).
const SCHEME: &str = "https://";

/// The help a URL door tells on a long press (DESIGN §23 item 10).
pub const OPENS: &str = "opens this address in the phone's browser; a long press copies it instead";

/// The help a code door tells.
pub const COPIES: &str = "copies this code, to paste where the browser asks for it";

/// Punctuation that leans on a token without being part of it: the full stop
/// after an address at a sentence's end, the bracket around one.
const LEANING_IN: &[char] = &['(', '[', '`', '"', '\''];
const LEANING_OUT: &[char] = &['.', ',', ';', ':', '!', '?', ')', ']', '`', '"', '\''];

/// One run of a line: words as they stand, or an address that is a door.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Part {
    Prose(String),
    Url(String),
}

/// **One tail line, read.** Its runs in order, and the device code it
/// carries — which stays in the prose where it was said, so the line still
/// reads as a sentence, and earns its copy door at the line's end.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Read {
    pub parts: Vec<Part>,
    pub code: Option<String>,
}

impl Read {
    /// Whether this line has no door in it at all, and paints as the weak
    /// prose it always did.
    #[must_use]
    pub fn plain(&self) -> bool {
        self.code.is_none() && !self.parts.iter().any(|part| matches!(part, Part::Url(_)))
    }
}

/// Read one line into its doors. Tokens are whitespace-separated; the
/// punctuation leaning on a token is peeled before it is judged and stays in
/// the prose either side, so `visit https://example.com.` opens the address
/// and keeps the full stop.
#[must_use]
pub fn doors(line: &str) -> Read {
    let mut read = Read::default();
    let mut prose = String::new();
    for token in line.split_whitespace() {
        let (lead, rest) = token.split_at(token.len() - token.trim_start_matches(LEANING_IN).len());
        let (core, trail) = rest.split_at(rest.trim_end_matches(LEANING_OUT).len());
        if core.len() > SCHEME.len() && core.starts_with(SCHEME) {
            say(&mut prose, lead);
            flush(&mut read, &mut prose);
            read.parts.push(Part::Url(core.to_owned()));
            prose.push_str(trail);
            continue;
        }
        if is_code(core) && read.code.is_none() {
            read.code = Some(core.to_owned());
        }
        say(&mut prose, token);
    }
    flush(&mut read, &mut prose);
    read
}

/// Whether `token` has the device-code shape ([`CODE`]).
#[must_use]
pub fn is_code(token: &str) -> bool {
    token.len() == CODE.len()
        && token
            .chars()
            .zip(CODE.chars())
            .all(|(have, want)| match want {
                '-' => have == '-',
                _ => have.is_ascii_uppercase() || have.is_ascii_digit(),
            })
}

/// The words on a code's door.
#[must_use]
pub fn copy_word(code: &str) -> String {
    format!("copy {code}")
}

/// The box a sign-in runs on, as a sentence names it.
fn place(on: &Machine) -> String {
    match on {
        Machine::Server => "the server".to_owned(),
        Machine::Client(name) => name.clone(),
    }
}

/// **What a credential landing here means**, said under a row while its tail
/// is open (§24 item 5; REMOTE §5.7): a bare row signs in the server, inside
/// the workspace, for every seat and every agent of it; a row on another
/// machine signs in THAT machine, whose credential is spent only by the
/// model calls the workspace routes to it.
#[must_use]
pub fn fact(workspace: &str, on: &Machine) -> String {
    match on {
        Machine::Server => format!("signs in the server, for every seat of {workspace}"),
        Machine::Client(name) => {
            format!("signs in {name}, for the model calls {workspace} routes to it")
        }
    }
}

/// The help the `sign in` button tells: which box the run starts on.
#[must_use]
pub fn starts(on: &Machine) -> String {
    format!(
        "starts this provider's sign-in on {}, and follows what it says",
        place(on)
    )
}

/// **Where a browser flow lands, for a row on another machine** (REMOTE
/// §5.7: *a browser-only row opens its browser on the far machine, which is
/// the one box where that redirect can land*). Said under every such row's
/// tail and not only a browser-only one's, because the listing does not
/// state which flow a row has: a sentence that is sometimes unneeded, over a
/// silence that is sometimes wrong. A device flow is untouched by it — its
/// address and code are doors on this glass. The server's own row has no
/// such sentence: §13.19's reading stands for it.
#[must_use]
pub fn browser(on: &Machine) -> Option<String> {
    match on {
        Machine::Server => None,
        Machine::Client(name) => Some(format!(
            "a sign-in that opens a web page opens its browser on {name}, and is finished \
             there; one that gives an address and a code is finished here"
        )),
    }
}

/// Add one word to the prose being gathered, a space before it where there
/// are words already. Leaning punctuation is a word with no space of its own
/// — it arrives attached to the token it leaned on.
fn say(prose: &mut String, word: &str) {
    if word.is_empty() {
        return;
    }
    if !prose.is_empty() && !prose.ends_with(LEANING_IN) {
        prose.push(' ');
    }
    prose.push_str(word);
}

/// Hand the gathered prose to the reading as one run, when there is any.
fn flush(read: &mut Read, prose: &mut String) {
    if !prose.is_empty() {
        read.parts.push(Part::Prose(std::mem::take(prose)));
    }
}
