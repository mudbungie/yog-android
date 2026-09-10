//! **The `http` tool** (DESIGN §16.1, the net rung): one request, through
//! Android's own networking stack.
//!
//! **Why a tool and not a command.** The shell tool has always been able to
//! run a command line, and until the packaged executables landed beside this
//! one there was nothing on an Android device to run: the platform's `toybox`
//! carries no `wget` and no `curl`, so a model on this phone could reach the
//! glass, the camera and the shade and could not fetch a URL. The two answers
//! are complementary and both are built — this one is the direct verb, and it
//! is the one that costs no packaged binary at all, because the stack it
//! spends is the platform's.
//!
//! **What that buys, stated where a model reads it.** TLS is
//! `HttpsURLConnection`'s, which trusts the device's own CA store — the
//! system roots plus whatever the operator installed — so nothing here ships
//! a certificate bundle that could go stale, and a device that trusts a
//! private CA reaches its own hosts with no argument. It also means this tool
//! cannot be more permissive than the device: a host the phone will not trust
//! is a refusal here, naming the platform's own sentence.
//!
//! **The answer is bounded and the file is not.** A capture becomes a model's
//! tool result, so the text is capped and elided in the middle — head and
//! tail, the shape `read_file` already uses, and the same reason: a tool that
//! quietly returned the first N bytes would be lying by omission. `save_to`
//! is the other half: the WHOLE body lands in a file under this app's own
//! storage and the capture says where, which is how a download reaches the
//! `open` tool and, through it, the operator's tap on an installer.
//!
//! **What is pure lives here and is tested** — the advertised element, the
//! argument reading, the containment rule on a saved path, and the elision.
//! The request itself is [`bridge`], android-only and excluded from coverage,
//! the seam every bridge in this crate draws.

use serde_json::{Map, Value, json};

use super::bridged::answer;
use super::{BAD_INPUT, arg, cap, object_schema, refused};
use crate::codec::{Capture, Tool};

#[cfg(target_os = "android")]
mod bridge;

#[cfg(test)]
mod tests;

pub(crate) const NAME: &str = "http";

/// How much of an answer a call hands back when the caller states no cap.
/// `read_file`'s number, for `read_file`'s reason: large enough for an
/// ordinary page, small enough that an accidental fetch of something enormous
/// does not fill a context window.
const CAPTURE_CAP: usize = 64 * 1024;

pub(crate) fn tool() -> Tool {
    super::tool(
        NAME,
        "Make one HTTP or HTTPS request from this Android device, through the device's own \
         networking stack. TLS is the platform's: it trusts this device's CA store, so a \
         host the phone does not trust is refused here too, naming the platform's own \
         reason. Redirects are followed. The answer is the status line, the response \
         headers, a blank line and the body, cut to 65536 characters unless a larger \
         `limit` is given — a cut answer keeps the head and the tail and says in the \
         middle how much went. `save_to` writes the WHOLE body to a file under this app's \
         own storage instead, and answers with the path: that path is what the `open` tool \
         hands to Android, which is how a downloaded APK reaches the installer for the \
         operator to tap. There is no proxy setting and no client certificate.",
        object_schema(
            json!({
                "url": { "type": "string",
                         "description": "the absolute URL, http:// or https://" },
                "method": { "type": "string",
                            "description": "the HTTP method; GET when unstated" },
                "headers": { "type": "object",
                             "description": "request headers, as name/value strings" },
                "body": { "type": "string",
                          "description": "the request body, for POST and its kin" },
                "save_to": { "type": "string",
                             "description": "write the body to this file under the app's \
                                             own storage instead of returning it; a bare \
                                             name lands in that directory" },
                "limit": { "type": "integer",
                           "description": "maximum characters of the answer to return" }
            }),
            &["url"],
        ),
    )
}

/// Dispatch the one tool. `data_dir` is this app's own storage — the
/// containment a saved path is judged against, and the directory a bare name
/// lands in.
pub(crate) fn run(o: &Map<String, Value>, data_dir: &str) -> Capture {
    let url = match arg(o, "url") {
        Ok(url) => url,
        Err(why) => return refused(BAD_INPUT, &why),
    };
    let read = method(o).and_then(|m| Ok((m, headers(o)?, saved(o, data_dir)?)));
    match read {
        Ok((method, headers, save_to)) => {
            let body = o.get("body").and_then(Value::as_str).unwrap_or_default();
            let reply = bridge_http(&method, &url, &headers, body, &save_to);
            bounded(answer(&reply), cap(o, "limit", CAPTURE_CAP))
        }
        Err(why) => refused(BAD_INPUT, &why),
    }
}

/// The capture with its stdout cut to the cap. Only stdout: a refusal's
/// sentence is one line by construction, and cutting a diagnostic would take
/// the half that names the fix.
fn bounded(capture: Capture, at: usize) -> Capture {
    Capture {
        stdout: elide(&capture.stdout, at),
        ..capture
    }
}

/// The method, upper-cased, or the sentence refusing it. Letters only: a
/// method is a token in the request line, and anything else is either a
/// mis-call or an attempt to write a second line into it.
fn method(o: &Map<String, Value>) -> Result<String, String> {
    let stated = o.get("method").map_or(Ok("GET"), |v| {
        v.as_str()
            .ok_or_else(|| "\"method\" is not a string".to_owned())
    })?;
    if stated.is_empty() || !stated.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(format!("{stated:?} is not an HTTP method: letters only"));
    }
    Ok(stated.to_ascii_uppercase())
}

/// The request headers as the `Name: value` lines the bridge speaks, or the
/// sentence refusing them. A newline in either half is refused rather than
/// escaped: it would write a header this call never stated, and there is no
/// legitimate reading of one.
fn headers(o: &Map<String, Value>) -> Result<String, String> {
    let Some(stated) = o.get("headers") else {
        return Ok(String::new());
    };
    let stated = stated
        .as_object()
        .ok_or_else(|| "\"headers\" is not an object of names and values".to_owned())?;
    let mut lines = String::new();
    for (name, value) in stated {
        let value = value
            .as_str()
            .ok_or_else(|| format!("the header {name:?} has a non-string value"))?;
        if name.contains(['\r', '\n']) || value.contains(['\r', '\n']) {
            return Err(format!("the header {name:?} carries a line break"));
        }
        lines.push_str(name);
        lines.push_str(": ");
        lines.push_str(value);
        lines.push('\n');
    }
    Ok(lines)
}

/// Where a saved body lands, or the sentence refusing the path. **The
/// containment is this function** and it is the whole of it: a bare name is
/// joined to the app's own storage, an absolute path must already be under
/// it, and no path may walk out with `..`. The rule is the one the tool
/// advertises, so it is stated once, here, where a test can reach it — the
/// platform would refuse most escapes anyway, and a refusal that arrives as
/// the OS's `EACCES` teaches the model nothing about what it may ask for.
fn saved(o: &Map<String, Value>, data_dir: &str) -> Result<String, String> {
    let Some(stated) = o.get("save_to") else {
        return Ok(String::new());
    };
    let stated = stated
        .as_str()
        .ok_or_else(|| "\"save_to\" is not a string".to_owned())?;
    if stated.is_empty() {
        return Err("\"save_to\" is empty: state a filename or leave it out".to_owned());
    }
    if stated.split('/').any(|part| part == "..") {
        return Err(format!("{stated:?} walks out of the app's own storage"));
    }
    let path = if stated.starts_with('/') {
        stated.to_owned()
    } else {
        format!("{data_dir}/{stated}")
    };
    if !path.starts_with(&format!("{data_dir}/")) {
        return Err(format!(
            "{stated:?} is not under this app's own storage ({data_dir})"
        ));
    }
    Ok(path)
}

/// The text, cut to `at` characters with the middle taken out and named.
/// Three quarters head and one quarter tail: the head carries the status and
/// the headers, which are what a caller reads first, and the tail is where a
/// page's own conclusion tends to be.
pub(crate) fn elide(text: &str, at: usize) -> String {
    let total = text.chars().count();
    if total <= at {
        return text.to_owned();
    }
    let head = at - at / 4;
    let tail = at / 4;
    let start: String = text.chars().take(head).collect();
    let end: String = text.chars().skip(total - tail).collect();
    let gone = total - head - tail;
    format!("{start}\n… {gone} characters elided …\n{end}")
}

/// The bridge, or the sentence a build without one gives.
#[cfg(not(target_os = "android"))]
fn bridge_http(_method: &str, _url: &str, _headers: &str, _body: &str, _save: &str) -> String {
    super::bridged::absent("Android network stack to ask")
}

#[cfg(target_os = "android")]
use bridge::bridge_http;
