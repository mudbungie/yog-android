//! **The packaged executables, as an environment** (DESIGN §16.1's net rung,
//! bl-f22a): what a command line runs under so that `curl` and `busybox` are
//! names `sh` can resolve.
//!
//! **Why a directory of symlinks and not a wrapper script.** Android has
//! executed nothing from an app's writable data directory since API 29 —
//! W^X, enforced by `SELinux` — so a shell script written there could never be
//! `exec`d, whatever its mode bits. A SYMLINK in that directory is not
//! executed: the kernel resolves it and executes the target, which lives
//! under `nativeLibraryDir`, the one place this uid may execute from. The
//! links are made by `dev.yog.Kit` once per process, and this side only
//! learns where they are.
//!
//! **Two variables, and neither is a knob.** `PATH` gains that directory at
//! the FRONT, so a name the platform also carries resolves to the packaged
//! copy — the point of packaging one at all. `SSL_CERT_FILE` names the PEM
//! bundle `dev.yog.Kit` builds beside the links out of the device's own
//! trusted roots, which is how the packaged curl verifies TLS: it is built
//! with no CA bundle and no CA path of its own, so the trust store is the
//! platform's, exactly as the `http` tool's is.
//!
//! **A FILE and not the platform's directory, and the reason is exact.**
//! Android names the files in its certificate store by OpenSSL's *old*
//! subject hash (`X509_NAME_hash_old`, the 0.9.8 one) while OpenSSL 1.0 and
//! later look a directory up by the new hash — measured on a current
//! emulator, the store holds `01419da9.0` and OpenSSL 3 goes looking for
//! `8d89cda1.0`. So a `capath` pointed at the platform's own store finds
//! nothing and every https fetch fails to build a chain, which is a failure
//! that looks exactly like a bad certificate. A concatenated PEM has no
//! naming convention to disagree about, and rebuilding it at every launch is
//! what keeps it a projection of the platform's store rather than a second
//! copy of it.
//!
//! **A build or a device with no packaged executables changes nothing.** The
//! fold answers an empty set, the command line runs under exactly the
//! environment it would have run under, and the shell tool's description is
//! the only thing that has to be honest about it.

/// The variable OpenSSL reads for a file of trusted roots.
const CERTS: &str = "SSL_CERT_FILE";

/// The environment a command line runs under, over what this process holds.
/// Impure by construction — it asks the platform where the links are — which
/// is why the decision it makes is [`folded`], beside it and tested.
pub(crate) fn additions() -> Vec<(String, String)> {
    folded(
        &bridge_bin(),
        &std::env::var("PATH").unwrap_or_else(|_| String::new()),
    )
}

/// What to add to a child's environment, given the platform's answer and the
/// `PATH` this process inherited. The answer is the links' directory and, on
/// a second line when this device had a readable store, the bundle built from
/// it. An answer in no protocol, a refusal, and an empty directory are one
/// case and it is the empty set: the command line then runs exactly as it
/// would have, which is the honest reading of *this device packages nothing*.
/// A directory with no bundle under it is not that case — the executables are
/// there and only TLS is unbacked, which curl says in its own words.
pub(crate) fn folded(reply: &str, inherited: &str) -> Vec<(String, String)> {
    let Some(("ok", payload)) = reply.split_once('\n') else {
        return Vec::new();
    };
    let mut lines = payload.lines();
    let Some(dir) = lines.next().filter(|dir| !dir.is_empty()) else {
        return Vec::new();
    };
    let path = if inherited.is_empty() {
        dir.to_owned()
    } else {
        format!("{dir}:{inherited}")
    };
    let mut added = vec![("PATH".to_owned(), path)];
    if let Some(roots) = lines.next().filter(|roots| !roots.is_empty()) {
        added.push((CERTS.to_owned(), roots.to_owned()));
    }
    added
}

/// The bridge, or the sentence a build with no Android under it gives.
#[cfg(not(target_os = "android"))]
fn bridge_bin() -> String {
    crate::tools::bridged::absent("Android to ask for its packaged executables")
}

#[cfg(target_os = "android")]
use super::bridge::bridge_bin;

#[cfg(test)]
mod tests {
    use super::{CERTS, additions, folded};

    #[test]
    fn a_directory_of_links_goes_to_the_front_of_the_path_beside_the_devices_own_roots() {
        let added = folded(
            "ok\n/data/user/0/dev.yog/files/bin\n/data/user/0/dev.yog/files/bin/roots.pem",
            "/system/bin:/vendor/bin",
        );
        assert_eq!(
            added,
            vec![
                (
                    "PATH".to_owned(),
                    "/data/user/0/dev.yog/files/bin:/system/bin:/vendor/bin".to_owned()
                ),
                (
                    CERTS.to_owned(),
                    "/data/user/0/dev.yog/files/bin/roots.pem".to_owned()
                ),
            ]
        );
    }

    #[test]
    fn links_with_no_readable_store_behind_them_still_go_on_the_path() {
        // The executables are there; only TLS is unbacked, and curl says that
        // in its own words rather than this app inventing a trust store.
        for reply in ["ok\n/bin", "ok\n/bin\n"] {
            let added = folded(reply, "/system/bin");
            assert_eq!(added.len(), 1, "for {reply:?}: {added:?}");
            assert_eq!(added[0].0, "PATH");
        }
    }

    #[test]
    fn a_process_holding_no_path_at_all_gets_the_directory_and_no_stray_separator() {
        let added = folded("ok\n/bin\n/bin/roots.pem", "");
        assert_eq!(added.first(), Some(&("PATH".to_owned(), "/bin".to_owned())));
    }

    #[test]
    fn nothing_packaged_is_the_empty_set_and_never_a_broken_path() {
        for reply in [
            "err\nthis app has not finished starting",
            "ok\n",
            "",
            "what",
        ] {
            assert!(folded(reply, "/system/bin").is_empty(), "for {reply:?}");
        }
        // And that is the arm a host build takes, every time.
        assert!(additions().is_empty());
    }
}
