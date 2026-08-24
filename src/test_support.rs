//! Test scaffolding for the wire: a throwaway PKI minted with the `openssl`
//! CLI (the same recipe the server's own provisioning shells out to — this
//! crate links no certificate library, in tests as in prod). The answering
//! servers live in `serve` (re-exported here), split out when the seat
//! model's scripted multi-connection server joined the one-shot original.

use std::path::Path;
use std::process::Command;

pub mod serve;

pub use serve::{serve_many, serve_once};

/// Mint a CA under `dir` as `<name>.pem`/`<name>.key`.
pub fn mint_ca(dir: &Path, name: &str) {
    let (key, pem, subj) = names(name);
    run(
        dir,
        &[
            "req", "-x509", "-newkey", "rsa:2048", "-nodes", "-days", "2", "-keyout", &key, "-out",
            &pem, "-subj", &subj,
        ],
    );
}

fn names(name: &str) -> (String, String, String) {
    (
        format!("{name}.key"),
        format!("{name}.pem"),
        format!("/CN=notreal-{name}"),
    )
}

/// Mint a leaf signed by `ca`, as `<name>.pem`/`<name>.key`. The server leaf
/// carries the loopback IP SAN the client verifies the dialled address
/// against; a client leaf needs none.
pub fn mint_leaf(dir: &Path, ca: &str, name: &str, ip_san: bool) {
    let (key, pem, subj) = names(name);
    let csr = format!("{name}.csr");
    run(
        dir,
        &[
            "req", "-newkey", "rsa:2048", "-nodes", "-keyout", &key, "-out", &csr, "-subj", &subj,
        ],
    );
    // EVERY leaf gets an extensions file, not only the server's: webpki
    // requires X.509 v3, and `openssl x509 -req` emits v1 when no extensions
    // are present — OpenSSL forces v3 unconditionally only since 3.2, so a
    // bare mint is v3 on one box and v1 on another (an older-openssl runner
    // minted v1 client leaves and every handshake test refused them with
    // UnsupportedCertVersion, bl-afe2). A fixture must not lean on a tool's
    // version-dependent default.
    let ext = dir.join(format!("{name}.ext"));
    let extensions = if ip_san {
        "basicConstraints=CA:FALSE\nsubjectAltName=IP:127.0.0.1\n"
    } else {
        "basicConstraints=CA:FALSE\n"
    };
    std::fs::write(&ext, extensions).unwrap();
    let (ca_key, ca_pem, _) = names(ca);
    let extfile = ext.display().to_string();
    run(
        dir,
        &[
            "x509",
            "-req",
            "-days",
            "2",
            "-in",
            csr.as_str(),
            "-out",
            pem.as_str(),
            "-CA",
            ca_pem.as_str(),
            "-CAkey",
            ca_key.as_str(),
            "-CAcreateserial",
            "-extfile",
            extfile.as_str(),
        ],
    );
}

fn run(dir: &Path, args: &[&str]) {
    let out = Command::new("openssl")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("openssl not runnable — the test PKI cannot be minted");
    assert!(
        out.status.success(),
        "openssl {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The provisioned-material shape over minted files, addressed at `address`.
pub fn material(dir: &Path, ca: &str, leaf: &str, address: &str) -> crate::material::Material {
    crate::material::Material {
        anchors: dir.join(format!("{ca}.pem")),
        chain: dir.join(format!("{leaf}.pem")),
        key: dir.join(format!("{leaf}.key")),
        address: address.to_owned(),
    }
}

/// A fresh scratch directory under the OS temp root — no `tempfile`
/// dependency (rule 6: zero unapproved deps, dev-deps included). Left for the
/// OS to reap; a test tree is a few PEM files.
pub fn scratch() -> std::path::PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "yog-android-test-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
