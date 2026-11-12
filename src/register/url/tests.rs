//! The registration URL: what is read out of one, and what is refused.

use super::{Offer, parse};

fn offer(host: &str, port: u16) -> Offer {
    Offer {
        host: host.to_owned(),
        port,
        code: "ABCD2345".to_owned(),
    }
}

/// The three shapes a host takes: a name on the standard port, a name with a
/// stated port, and a bracketed v6 literal either way.
#[test]
fn a_registration_url_names_a_host_a_port_and_a_code() {
    for (url, read) in [
        (
            "https://engine.example/yog/register/ABCD2345",
            offer("engine.example", 443),
        ),
        (
            "  https://engine.example:7737/yog/register/ABCD2345\n",
            offer("engine.example", 7737),
        ),
        (
            "https://[2001:db8::7]/yog/register/ABCD2345",
            offer("2001:db8::7", 443),
        ),
        (
            "https://[2001:db8::7]:7737/yog/register/ABCD2345",
            offer("2001:db8::7", 7737),
        ),
    ] {
        assert_eq!(parse(url), Ok(read), "{url}");
    }
}

/// The authority is what the `Host` header says: no port where it is the
/// scheme's own, and a v6 literal back in its brackets.
#[test]
fn the_authority_states_a_port_only_where_it_is_not_the_standard_one() {
    assert_eq!(offer("engine.example", 443).authority(), "engine.example");
    assert_eq!(
        offer("engine.example", 7737).authority(),
        "engine.example:7737"
    );
    assert_eq!(offer("2001:db8::7", 443).authority(), "[2001:db8::7]");
    assert_eq!(offer("2001:db8::7", 7737).authority(), "[2001:db8::7]:7737");
    assert_eq!(
        offer("engine.example", 443).path(),
        "/yog/register/ABCD2345"
    );
}

/// Anything else refuses before a socket opens, saying what one looks like.
#[test]
fn anything_that_is_not_a_registration_url_is_refused_by_shape() {
    for url in [
        "",
        "http://engine.example/yog/register/ABCD2345",
        "https://engine.example/yog/enroll/ABCD2345",
        "https://engine.example/yog/register/",
        "https://engine.example/yog/register/ABCD/2345",
        "https://engine.example/yog/register/ABCD2345?x=1",
        "https:///yog/register/ABCD2345",
        "https://engine.example:port/yog/register/ABCD2345",
        "https://[2001:db8::7/yog/register/ABCD2345",
        "https://[2001:db8::7]7737/yog/register/ABCD2345",
        "https://u@example.com/yog/register/ABCD2345",
        "https://engine.example/elsewhere/yog/register/ABCD2345",
    ] {
        let refusal = parse(url).unwrap_err();
        assert!(refusal.contains("is not a registration URL"), "{refusal}");
        assert!(refusal.contains("https://<host>/yog/register/<code>"));
    }
}
