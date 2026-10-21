//! **The rows** (DESIGN §23 item 11's table), in the order the pane paints
//! them. Each is three strings and two accessors; the help sentence says
//! what the knob does in the operator's terms and never the toolkit's.

use super::{Kind, Knob, Knobbed, Toggle};

/// One switch as a row: the key, the label, the sentence and the switch it
/// reads and writes. A macro rather than a `const fn`, because a function
/// spent only at compile time is a line the coverage floor never sees run.
macro_rules! toggle {
    ($key:literal, $label:literal, $help:literal, $switch:expr) => {
        Knob {
            key: $key,
            label: $label,
            help: $help,
            kind: Kind::Toggle,
            get: |s| Knobbed::On(s.on($switch)),
            set: |s, v| {
                if let Knobbed::On(on) = v {
                    s.set($switch, on);
                }
            },
        }
    };
}

/// **Every knob.**
pub const KNOBS: &[Knob] = &[
    Knob {
        key: "scale",
        label: "scale",
        help: "how large everything is drawn: text, air and controls alike",
        kind: Kind::Number {
            unit: "×",
            floor: 0.75,
            ceiling: 1.75,
            step: 0.05,
        },
        get: |s| Knobbed::At(s.scale),
        set: |s, v| {
            if let Knobbed::At(at) = v {
                s.scale = at;
            }
        },
    },
    toggle!(
        "help",
        "long-press help",
        "whether a control says what it does when held",
        Toggle::Help
    ),
    toggle!(
        "bubbles",
        "bubbles",
        "a fill under each block of the chat saying who it is from",
        Toggle::Bubbles
    ),
    toggle!(
        "fold",
        "fold the working",
        "whether what a turn did before its answer folds to one line once the answer stands",
        Toggle::Fold
    ),
    toggle!(
        "full",
        "full responses",
        "whether what was said stands open rather than folded to its first line",
        Toggle::Full
    ),
    toggle!(
        "steps",
        "steps open",
        "whether thinking, calls and results stand open rather than folded",
        Toggle::Steps
    ),
    toggle!(
        "dock",
        "dock unfolded",
        "whether the running conversations stand listed above the rule from the start",
        Toggle::Dock
    ),
];
