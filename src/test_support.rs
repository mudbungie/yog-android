//! Test scaffolding for the wire: a throwaway PKI minted with the `openssl`
//! CLI (the same recipe the server's own provisioning shells out to — this
//! crate links no certificate library, in tests as in prod), and a one-shot
//! mTLS answering server built from rustls' server half, which prod here
//! deliberately does not have (the phone never listens; DESIGN §1).

use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::net::TcpListener;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::thread::JoinHandle;

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
    let ext = dir.join(format!("{name}.ext"));
    std::fs::write(&ext, "subjectAltName=IP:127.0.0.1\n").unwrap();
    let (ca_key, ca_pem, _) = names(ca);
    let extfile = ext.display().to_string();
    let mut args = vec![
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
    ];
    if ip_san {
        args.extend_from_slice(&["-extfile", extfile.as_str()]);
    }
    run(dir, &args);
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

/// A one-shot mTLS server on loopback: accepts one connection, requires a
/// client certificate chaining to `ca`, reads one frame, answers with each of
/// `replies` then the terminator. Returns the bound address and the join
/// handle carrying the request it read.
pub fn serve_once(
    dir: &Path,
    ca: &str,
    leaf: &str,
    replies: Vec<Vec<u8>>,
) -> (String, JoinHandle<Vec<u8>>) {
    let mut store = rustls::RootCertStore::empty();
    for anchor in CertificateDer::pem_file_iter(dir.join(format!("{ca}.pem"))).unwrap() {
        store.add(anchor.unwrap()).unwrap();
    }
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let verifier = rustls::server::WebPkiClientVerifier::builder_with_provider(
        Arc::new(store),
        Arc::clone(&provider),
    )
    .build()
    .unwrap();
    let chain: Vec<CertificateDer<'static>> =
        CertificateDer::pem_file_iter(dir.join(format!("{leaf}.pem")))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
    let key = PrivateKeyDer::from_pem_file(dir.join(format!("{leaf}.key"))).unwrap();
    let config = Arc::new(
        rustls::ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_client_cert_verifier(verifier)
            .with_single_cert(chain, key)
            .unwrap(),
    );
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("127.0.0.1:{}", listener.local_addr().unwrap().port());
    let handle = std::thread::spawn(move || {
        let (tcp, _) = listener.accept().unwrap();
        let conn = rustls::ServerConnection::new(config).unwrap();
        let mut tls = rustls::StreamOwned::new(conn, tcp);
        let request = crate::frame::read_frame(&mut tls).unwrap().unwrap();
        for reply in &replies {
            crate::frame::write_frame(&mut tls, reply).unwrap();
        }
        crate::frame::write_end(&mut tls).unwrap();
        request
    });
    (address, handle)
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
