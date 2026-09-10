//! The JNI half of the `http` tool: one static call into `dev.yog.Net`,
//! answering in [`crate::tools::bridged`]'s two-line protocol.
//!
//! **Five strings and no structure.** The method, the URL, the request
//! headers as `Name: value` lines, the body and the file to save into all
//! cross as text, because the descriptor a door builds is `(String…)String`
//! and a second marshalling — a JSON argument, a parcel — would be a second
//! grammar for the far side to keep in step with. An absent header set, an
//! absent body and an absent save path are all the empty string: they are
//! absent in exactly one way, which is the reading `dev.yog.Net` states.
//!
//! **No thread is chosen here.** The call runs on whichever thread the tool
//! host invoked on, which is never the main one — a network read on the main
//! thread is `NetworkOnMainThreadException`, and the reason this is safe is
//! the host's, not this file's.
//!
//! This file is android-only and excluded from coverage; what it hands back
//! is parsed by [`crate::tools::bridged::answer`], which is pure and tested.

use crate::tools::bridged::Door;

/// The class the static entry point lives on, and what its failures are
/// reported as.
static NET: Door = Door::new("dev.yog.Net", "network bridge");

pub(crate) fn bridge_http(
    method: &str,
    url: &str,
    headers: &str,
    body: &str,
    save: &str,
) -> String {
    NET.strings("http", &[method, url, headers, body, save])
}
