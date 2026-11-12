//! **The registration door's worker, and the one JNI call under it** (DESIGN
//! §5): the claim, then the watch, on a thread of their own — split from
//! `door.rs` at the cap, on the seam between what the page paints and what
//! runs behind it. Android-only and excluded from coverage for `door.rs`'
//! reason: everything decided here is `crate::register`.

use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::time::Duration;

use eframe::egui;
use jni::objects::JValue;

use crate::register::{Answer, Registered, Watch, Watcher};
use crate::shell::jvm::{Bridge, attached};

/// The class the static entry point lives on, dotted for the class loader.
const CLASS: &str = "dev.yog.Register";

/// What this bridge's failures are reported as.
const LABEL: &str = "registration bridge";

/// How long the worker rests between dials while the offer waits.
const EVERY: Duration = Duration::from_secs(3);

/// What the worker tells the page.
pub(super) enum Told {
    Claimed(Result<Registered, String>),
    Watched(Watch),
}

/// The worker: the claim, then the watch until the server's answer changes or
/// the page stops listening.
pub(super) fn work(dir: PathBuf, url: String, tell: Sender<Told>, ctx: egui::Context) {
    std::thread::spawn(move || {
        let outcome = crate::register::claim(&dir, &url, &mut post);
        let claimed = outcome.is_ok();
        let heard = tell.send(Told::Claimed(outcome)).is_ok();
        ctx.request_repaint();
        if !claimed || !heard {
            return;
        }
        let mut watcher = Watcher::new(&dir);
        loop {
            let watch = watcher.look();
            let over = watch == Watch::Approved;
            let heard = tell.send(Told::Watched(watch)).is_ok();
            ctx.request_repaint();
            if over || !heard {
                return;
            }
            std::thread::sleep(EVERY);
        }
    });
}

/// The one request, through the platform's HTTPS client.
fn post(url: &str, body: &str) -> Result<Answer, String> {
    let mut env = attached()?;
    let bridge = Bridge::open(&mut env, CLASS, LABEL)?;
    let (Ok(url), Ok(body)) = (env.new_string(url), env.new_string(body)) else {
        return Err("the request could not be handed to the platform".to_owned());
    };
    crate::register::answer(&bridge.string(
        &mut env,
        "post",
        "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
        &[JValue::Object(&url), JValue::Object(&body)],
    ))
}
