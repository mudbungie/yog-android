+++
title = "paste into the composer is unproven: Gboard's clipboard chip and KEYCODE_PASTE did not reach the text mirror on the emulator; verify on a real device and fix the IME bridge if it reproduces"
created = 1788935360
updated = 1789002764
priority = 3
root_commit = "b8421205e882caeadc666ccff26464e4e0f60dda"
tags = ["usability-r3"]
+++
From bl-7781's landing (227a205): copy-out via long press is proven (system clip preview + Gboard list), but pasting INTO the message field did not work on the AVD — Gboard's clipboard feature was off, and a raw KEYCODE_PASTE did not reach the GameActivity text mirror. Re-drive on the operator's phone (long-press the field → Paste); if it fails, the IME bridge must accept the paste action (commitText / the clipboard content) into the mirror. Also the composer should accept a long-press → Paste menu the way every Android field does.

---

Answered by bl-8bbb's landing (main 02309b7): the composer's field is now a native Android EditText overlaid at the rect egui lays out, so paste is the platform's own. Proven on the emulator against a live engine — a long press on the field raised Cut/Copy/Paste/Share/Select all, and a Copy then a Paste at the caret doubled the draft; the selection handles the mirror could never offer are there too. The IME bridge needed no paste arm: the question this ball asked — does KEYCODE_PASTE reach the text mirror — is dissolved rather than fixed, because the mirror no longer owns this field. It still owns the search field, the enroll envelope and the five single-purpose fields that borrow the composer's widget id; paste into those is unproven and is the residual.
