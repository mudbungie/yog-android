//! Test scaffolding for the wire: a throwaway PKI minted with the `openssl`
//! CLI (the same recipe the server's own provisioning shells out to — this
//! crate links no certificate library, in tests as in prod). The answering
//! servers live in `serve` (re-exported here), split out when the seat
//! model's scripted multi-connection server joined the one-shot original.

use std::path::Path;
use std::process::Command;

pub mod serve;

pub use held::{Beat, serve_held};
pub use serve::{Turn, serve_lanes, serve_many, serve_once, serve_turns, serve_versioned};

pub mod held;

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
    subject(name, None)
}

/// The same three names with an optional organizational unit — REMOTE §4.2's
/// grade, which rides in the leaf's own subject and nowhere else.
fn subject(name: &str, unit: Option<&str>) -> (String, String, String) {
    let ou = unit.map(|u| format!("/OU={u}")).unwrap_or_default();
    (
        format!("{name}.key"),
        format!("{name}.pem"),
        format!("/CN=notreal-{name}{ou}"),
    )
}

/// Mint a leaf signed by `ca`, as `<name>.pem`/`<name>.key`. The server leaf
/// carries the loopback IP SAN the client verifies the dialled address
/// against; a client leaf needs none.
pub fn mint_leaf(dir: &Path, ca: &str, name: &str, ip_san: bool) {
    mint_graded(dir, ca, name, ip_san, None);
}

/// A **foot-grade** leaf (REMOTE §4.2): the same mint with `OU=foot` written
/// into the subject, which is the one place a grade may be stated.
pub fn mint_foot(dir: &Path, ca: &str, name: &str) {
    mint_graded(dir, ca, name, false, Some("foot"));
}

/// A leaf carrying an arbitrary organizational unit — the near miss the grade
/// walk must not read as a promotion.
pub fn mint_unit(dir: &Path, ca: &str, name: &str, unit: &str) {
    mint_graded(dir, ca, name, false, Some(unit));
}

fn mint_graded(dir: &Path, ca: &str, name: &str, ip_san: bool, unit: Option<&str>) {
    let (key, pem, subj) = subject(name, unit);
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
        pairing: None,
    }
}

/// **Time a test turns by hand** — `crate::ladder::Clock` over a shared
/// offset, so the silence bound and the backoff walk without waiting.
pub struct FakeClock {
    started: std::time::Instant,
    offset: std::sync::Mutex<std::time::Duration>,
}

impl FakeClock {
    pub fn new() -> std::sync::Arc<FakeClock> {
        std::sync::Arc::new(FakeClock {
            started: std::time::Instant::now(),
            offset: std::sync::Mutex::new(std::time::Duration::ZERO),
        })
    }

    /// Move the clock forward by `by`.
    pub fn advance(&self, by: std::time::Duration) {
        let mut offset = self
            .offset
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *offset += by;
    }
}

impl crate::ladder::Clock for FakeClock {
    fn now(&self) -> std::time::Instant {
        let offset = *self
            .offset
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.started + offset
    }

    fn unix(&self) -> i64 {
        i64::try_from(self.now().duration_since(self.started).as_secs()).unwrap_or(0)
            + 1_700_000_000
    }
}

/// Poll `ready` every few milliseconds for up to `wait`: whether it came true.
pub fn until(ready: &mut dyn FnMut() -> bool, wait: std::time::Duration) -> bool {
    let started = std::time::Instant::now();
    while started.elapsed() < wait {
        if ready() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    ready()
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
