//! The client configuration's refusals, each on the input that earns it —
//! the happy path is exercised end-to-end by the transport tests.

use super::client_config;
use crate::test_support::{material, mint_ca, mint_leaf, scratch};

#[test]
fn a_provisioned_store_builds() {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "client", false);
    let m = material(&dir, "ca", "client", "127.0.0.1:1");
    client_config(&m).unwrap();
}

#[test]
fn a_missing_chain_refuses_by_path() {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "client", false);
    let mut m = material(&dir, "ca", "client", "127.0.0.1:1");
    m.chain = dir.join("gone.pem");
    assert!(client_config(&m).unwrap_err().contains("gone.pem"));
}

#[test]
fn a_missing_key_refuses_by_path() {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "client", false);
    let mut m = material(&dir, "ca", "client", "127.0.0.1:1");
    m.key = dir.join("gone.key");
    assert!(client_config(&m).unwrap_err().contains("gone.key"));
}

#[test]
fn an_empty_chain_file_is_no_certificate() {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "client", false);
    std::fs::write(dir.join("empty.pem"), "").unwrap();
    let mut m = material(&dir, "ca", "client", "127.0.0.1:1");
    m.chain = dir.join("empty.pem");
    assert!(client_config(&m).unwrap_err().contains("no certificate"));
}

#[test]
fn an_empty_anchor_file_is_no_certificate() {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "client", false);
    std::fs::write(dir.join("empty-ca.pem"), "").unwrap();
    let mut m = material(&dir, "ca", "client", "127.0.0.1:1");
    m.anchors = dir.join("empty-ca.pem");
    assert!(client_config(&m).unwrap_err().contains("no certificate"));
}

#[test]
fn a_garbage_anchor_refuses() {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "client", false);
    // A well-formed PEM block whose DER is nonsense: the pem iterator hands
    // it over and the trust-store add is what refuses it.
    std::fs::write(
        dir.join("junk.pem"),
        "-----BEGIN CERTIFICATE-----\nbm90cmVhbCBqdW5rIGRlcg==\n-----END CERTIFICATE-----\n",
    )
    .unwrap();
    let mut m = material(&dir, "ca", "client", "127.0.0.1:1");
    m.anchors = dir.join("junk.pem");
    assert!(client_config(&m).unwrap_err().contains("junk.pem"));
}

#[test]
fn a_key_that_does_not_match_the_chain_refuses_as_identity() {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "client", false);
    mint_leaf(&dir, "ca", "other", false);
    let mut m = material(&dir, "ca", "client", "127.0.0.1:1");
    m.key = dir.join("other.key");
    assert!(client_config(&m).unwrap_err().contains("client identity"));
}
