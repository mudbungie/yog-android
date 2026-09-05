//! The vendored table reads, and the two columns nothing may be missing.

#[test]
fn the_vendored_table_reads_and_carries_the_two_columns_that_are_judged() {
    let rows = super::rows(super::TABLE).unwrap();
    assert!(rows.len() > 40, "{} rows", rows.len());
    assert!(rows.iter().all(|row| !row.verb.is_empty()));
    assert!(
        rows.iter()
            .all(|row| row.surface == "control" || row.surface == "machine")
    );
    let message = rows
        .iter()
        .find(|row| row.verb == "message")
        .cloned()
        .unwrap_or_else(|| unreachable!());
    assert_eq!(message.surface, "control");
    assert!(message.usage.starts_with("/message"));
    assert!(!message.detail.is_empty());
}

#[test]
fn a_table_that_is_not_one_refuses_naming_what_it_found() {
    assert!(
        super::rows("{")
            .unwrap_err()
            .starts_with("reply/help.json is not JSON"),
        "malformed JSON"
    );
    assert_eq!(
        super::rows("{}").unwrap_err(),
        "reply/help.json carries no frames[0].rows array"
    );
    assert_eq!(
        super::rows(r#"{"frames":[{"rows":[{}]}]}"#).unwrap_err(),
        "a help row states no verb"
    );
    assert_eq!(
        super::rows(r#"{"frames":[{"rows":[{"verb":"ack"}]}]}"#).unwrap_err(),
        "ack: a help row states no surface — re-vendor the corpus from a yog at \
         protocol 7 or later"
    );
}

/// **A row may say nothing about itself and still be a row**: the two judged
/// columns are required and the prose is not.
#[test]
fn a_row_with_no_prose_reads_as_a_thin_row_rather_than_a_broken_table() {
    let rows =
        super::rows(r#"{"frames":[{"rows":[{"verb":"ack","surface":"control"}]}]}"#).unwrap();
    let row = rows.first().cloned().unwrap_or_else(|| unreachable!());
    assert_eq!(
        (
            row.summary.as_str(),
            row.usage.as_str(),
            row.detail.as_str()
        ),
        ("", "", "")
    );
}
