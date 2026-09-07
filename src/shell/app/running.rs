//! **What this launch IS**, read off the one derivation that decides it — the
//! `Running` the boot produced, and the material behind it.
//!
//! Split from [`super`] (bl-5070) on the seam that file's own first sentence
//! draws and `app/fields.rs` already took once: what the shell HOLDS is the
//! struct, and what it answers about the component it is running is here.
//! Every method below reads `running` or the activity behind it and touches no
//! other field, which is what makes the seam a real one rather than a line
//! drawn to fit under a cap.

use super::Shell;
use crate::seat::Model;
use crate::shell::boot::{Running, boot};

impl Shell {
    /// Re-read what is provisioned and start whatever it now names. A read of
    /// this app's own storage — never a dial — and the act that makes material
    /// pushed over a cable land without relaunching the process.
    pub(crate) fn reboot(&mut self) {
        self.running = boot(&self.android);
        self.chose = None;
        // A recheck is the configuration's exit: whatever the derivation now
        // says is the screen the operator asked to see.
        self.settings = false;
    }

    /// Where material goes — the same directory the boot derivation reads.
    /// The configuration surface paints it when it is opened over a running
    /// component, where no `Running::Cold` carries it along (bl-387f).
    pub(crate) fn material_dir(&self) -> String {
        crate::shell::boot::wire_dir(&self.android)
            .display()
            .to_string()
    }

    /// The seat model, when this launch is running one.
    pub(crate) fn model(&self) -> Option<&Model> {
        match &self.running {
            Running::Seat { model, .. } => Some(model.as_ref()),
            _ => None,
        }
    }

    pub(crate) fn model_mut(&mut self) -> Option<&mut Model> {
        match &mut self.running {
            Running::Seat { model, .. } => Some(model.as_mut()),
            _ => None,
        }
    }

    /// Who this device is on the wire, and as what: the leaf's own common
    /// name and the component its grade enrolled it as (REMOTE §2, §4.2).
    /// Painted rather than logged, because a seat showing an empty roster and
    /// a seat registered in no workspace look identical until this line says
    /// which client the engine was answering.
    pub(crate) fn identity(&self) -> String {
        use crate::bootstrap::Component;
        match &self.running {
            Running::Seat { client, .. } => format!("{client} · {}", Component::Seat.brand()),
            Running::Foot { client, .. } => format!("{client} · {}", Component::Foot.brand()),
            Running::Cold { .. } => String::new(),
        }
    }
}
