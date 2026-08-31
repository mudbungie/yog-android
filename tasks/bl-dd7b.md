+++
title = "the enroll envelope, pasted: the QR payload as text, validated and landed"
created = 1788147813
updated = 1788147825
claimant = "OrderGreeter"
priority = 8
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
+++
yog is landing an **enroll act** whose reply is a QR envelope: compact JSON, contract recorded in yog `docs/REMOTE.md`.

    {"yog-enroll":1,"grade":…,"name":…,"address":…,"ca":…,"cert":…,"key":…}

**This ball is the envelope itself and its degraded path — paste — not the camera.** The paste path must exist anyway: a scan that will not focus, a phone with the camera permission denied, an operator reading the material off a laptop screen into a text field. Building it first also means the camera ball, when it lands, is *only* a decoder feeding an already-proven sink.

What ships:

- **A parser** over the envelope: the version field is checked first and by name, a wrong or absent one refusing with both versions in the sentence (the same fail-closed shape `src/hello.rs` gives the wire preface). Every field required, none silently defaulted.
- **Landing** into the app's channel store per `src/material.rs`'s layout — `WANTED`, the reader's own list, so a fifth file cannot be required by the reader and unwritten by the writer.
- **The grade is not taken on the envelope's word.** REMOTE §4.2 puts the grade on the certificate and DESIGN §9 derives the component from it; an envelope field saying otherwise is a second authority for one fact. The envelope's `grade` must AGREE with the leaf's own `OU`, and a disagreement refuses naming both — it is a defect in whatever minted the envelope, and landing it would enroll a device as something its certificate is not.
- **The derived component comes up on the spot**: landing re-runs the boot derivation, so foot material yields Thrall advertising and operator material yields Lernie dialing, with no relaunch.

REMOTE §1.4 is untouched. Pasting material an operator carried here by eye is the third delivery channel DESIGN §5 already names — *"a trusted client mints and displays the credential; the new device scans it"* — and the app still dials nothing until it holds a leaf.