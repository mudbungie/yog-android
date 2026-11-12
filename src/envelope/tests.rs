//! What is left of the envelope: the mint's own writing of it.

use super::{Envelope, TAG, VERSION, write};
use crate::leaf::Grade;

fn held(grade: Grade) -> Envelope {
    Envelope {
        grade,
        name: "notreal-phone".to_owned(),
        address: "server.example.com:7737".to_owned(),
        ca: "ca".to_owned(),
        cert: "cert".to_owned(),
        key: "key".to_owned(),
    }
}

/// The tag, the version and the six fields, each grade under its own word —
/// and a word this build cannot spell written back as the server said it
/// (REMOTE §3.2).
#[test]
fn an_envelope_is_written_as_its_tag_and_its_six_fields() {
    for (grade, word) in [
        (Grade::Foot, "foot"),
        (Grade::Operator, "operator"),
        (Grade::Unknown("admin".to_owned()), "admin"),
    ] {
        let value: serde_json::Value = serde_json::from_str(&write(&held(grade))).unwrap();
        assert_eq!(value[TAG], VERSION);
        assert_eq!(value["grade"], word);
        assert_eq!(value["name"], "notreal-phone");
        assert_eq!(value["address"], "server.example.com:7737");
        assert_eq!(value.as_object().unwrap().len(), 7);
    }
}
