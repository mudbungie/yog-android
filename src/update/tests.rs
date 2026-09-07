//! The offer decision, over the shapes the GitHub releases feed answers with
//! and the ones a hand-cut release can produce.

use super::{Offer, answered, newer, offer};

/// One release feed, as `GET /releases/latest` answers it — the two fields
/// this module reads and one asset that is not the APK, because a release
/// carrying more than one asset is the ordinary case.
fn feed(tag: &str, apk: &str) -> String {
    format!(
        "ok\n{{\"tag_name\":\"{tag}\",\"assets\":[\
         {{\"name\":\"checksums.txt\",\"browser_download_url\":\"https://x/c.txt\"}},\
         {{\"name\":\"yog-android-1.apk\",\"browser_download_url\":\"{apk}\"}}]}}"
    )
}

#[test]
fn offers_a_newer_release() {
    assert_eq!(
        offer(&feed("v0.0.2", "https://x/app.apk"), "0.0.1"),
        Some(Offer {
            version: "0.0.2".to_owned(),
            url: "https://x/app.apk".to_owned(),
        })
    );
}

#[test]
fn a_tag_without_the_v_prefix_reads_the_same() {
    assert_eq!(
        offer(&feed("0.0.2", "https://x/app.apk"), "0.0.1").map(|o| o.version),
        Some("0.0.2".to_owned())
    );
}

#[test]
fn the_installed_version_offers_nothing() {
    assert_eq!(offer(&feed("v0.0.1", "https://x/app.apk"), "0.0.1"), None);
}

#[test]
fn an_older_release_is_never_a_downgrade_offer() {
    assert_eq!(offer(&feed("v0.0.1", "https://x/app.apk"), "0.1.0"), None);
}

#[test]
fn a_refusal_from_the_bridge_offers_nothing() {
    assert_eq!(offer("err\nno network", "0.0.1"), None);
}

#[test]
fn an_answer_in_no_protocol_at_all_offers_nothing() {
    assert_eq!(offer("", "0.0.1"), None);
}

#[test]
fn a_body_that_is_not_json_offers_nothing() {
    assert_eq!(offer("ok\n<html>404</html>", "0.0.1"), None);
}

#[test]
fn a_release_naming_no_tag_offers_nothing() {
    assert_eq!(offer("ok\n{\"assets\":[]}", "0.0.1"), None);
}

#[test]
fn a_tag_that_is_not_a_string_offers_nothing() {
    assert_eq!(offer("ok\n{\"tag_name\":7,\"assets\":[]}", "0.0.1"), None);
}

#[test]
fn a_release_with_no_assets_field_offers_nothing() {
    assert_eq!(offer("ok\n{\"tag_name\":\"v9.9.9\"}", "0.0.1"), None);
}

#[test]
fn an_assets_field_that_is_not_a_list_offers_nothing() {
    assert_eq!(
        offer("ok\n{\"tag_name\":\"v9.9.9\",\"assets\":{}}", "0.0.1"),
        None
    );
}

#[test]
fn a_release_carrying_no_apk_offers_nothing() {
    assert_eq!(
        offer(
            "ok\n{\"tag_name\":\"v9.9.9\",\"assets\":[{\"name\":\"notes.txt\",\
             \"browser_download_url\":\"https://x/notes.txt\"}]}",
            "0.0.1"
        ),
        None
    );
}

#[test]
fn an_asset_with_no_name_is_not_the_apk() {
    assert_eq!(
        offer(
            "ok\n{\"tag_name\":\"v9.9.9\",\"assets\":[{\"browser_download_url\":\"https://x/a.apk\"}]}",
            "0.0.1"
        ),
        None
    );
}

#[test]
fn an_apk_offered_over_plaintext_is_refused() {
    assert_eq!(offer(&feed("v9.9.9", "http://x/app.apk"), "0.0.1"), None);
}

#[test]
fn an_apk_asset_with_no_url_is_refused() {
    assert_eq!(
        offer(
            "ok\n{\"tag_name\":\"v9.9.9\",\"assets\":[{\"name\":\"a.apk\"}]}",
            "0.0.1"
        ),
        None
    );
}

#[test]
fn an_apk_asset_whose_url_is_not_a_string_is_refused() {
    assert_eq!(
        offer(
            "ok\n{\"tag_name\":\"v9.9.9\",\"assets\":[{\"name\":\"a.apk\",\"browser_download_url\":3}]}",
            "0.0.1"
        ),
        None
    );
}

#[test]
fn versions_order_by_component_and_not_by_text() {
    assert!(newer("0.0.9", "0.0.10"));
    assert!(!newer("0.0.10", "0.0.9"));
    assert!(newer("0.9.0", "1.0.0"));
    assert!(!newer("1.0.0", "0.9.9"));
}

#[test]
fn a_missing_component_reads_as_zero() {
    assert!(!newer("0.1", "0.1.0"));
    assert!(!newer("0.1.0", "0.1"));
    assert!(newer("0.1", "0.1.1"));
}

#[test]
fn a_version_that_does_not_parse_is_never_newer() {
    assert!(!newer("0.0.1", "0.0.2-rc1"));
    assert!(!newer("0.0.1", ""));
    assert!(!newer("", "0.0.2"));
    assert!(!newer("nightly", "0.0.2"));
}

/// The one distinction the shell makes, and the reason a refusal counts as an
/// answer: a fetch that failed has CONCLUDED, and asking again every frame
/// would not change it. Only the silence before any answer is unsettled.
/// Everything else — what the answer says, and whether it earns an offer — is
/// [`offer`]'s above.
#[test]
fn a_concluded_fetch_is_answered_and_the_silence_before_one_is_not() {
    assert!(answered(&feed("v0.0.2", "https://x/app.apk")));
    assert!(answered("err\nthe release feed answered 404."));
    assert!(!answered(""));
}
