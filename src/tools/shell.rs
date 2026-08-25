//! **The shell tool**: a command line, run by this device's own `sh`.
//!
//! It runs as the app's own user, which is the honest containment story and
//! the one stated in the description a model reads: an Android app is a uid
//! with its own private storage and network, and a command that needs more
//! than that fails the way it would in any other terminal on the box. REMOTE
//! §5's containment paragraph says this in general — execution happens on a
//! machine the adjudicator cannot inspect, and the design must not claim
//! otherwise.
//!
//! **The deadline terminates the child**, and the capture says so with the
//! shell's own `timeout` verdict, so an operator reading a transcript
//! recognizes it. Draining happens on two threads because a child that fills
//! a pipe nobody is reading blocks forever, and a deadline that could not be
//! enforced would be a deadline in name.

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Map, Value, json};

use super::{BAD_INPUT, arg, object_schema, refused};
use crate::codec::{Capture, Tool};

pub(crate) const NAME: &str = "shell";

/// The verdict a command that outran its deadline earns — the shell's own
/// convention for `timeout`.
pub(crate) const TIMED_OUT: i32 = 124;

/// The verdict a command that could not be started at all earns: no child ran,
/// so there is no child's own code to report.
const NO_SHELL: i32 = 126;

/// How long a command may run before the child is terminated. It is this
/// machine's own bound, not the caller's: the machine that spawned the process
/// is the one that can stop it, and the driver's longer patience stands behind
/// it for the case where this whole process went away.
const DEADLINE: Duration = Duration::from_mins(1);

/// How often a running child is looked at. A latency knob on the answer, not
/// on the run: the child streams into its pipes regardless.
const POLL: Duration = Duration::from_millis(20);

pub(crate) fn tool() -> Tool {
    super::tool(
        NAME,
        "Run a command line on this Android device through /system/bin/sh. It runs as the \
         app's own user: its private storage and the network are reachable, most of the \
         filesystem is not, and anything needing a system permission this app was not \
         granted will be refused by the platform. Returns the command's stdout, stderr and \
         exit status. A command still running after 60 seconds is terminated and reported \
         as exit 124.",
        object_schema(
            json!({ "command": { "type": "string",
                                 "description": "the command line, as sh would read it" } }),
            &["command"],
        ),
    )
}

pub(crate) fn run(o: &Map<String, Value>) -> Capture {
    match arg(o, "command") {
        Ok(command) => execute(&command, DEADLINE),
        Err(why) => refused(BAD_INPUT, &why),
    }
}

/// Spawn, drain, and wait — or terminate. Separated from [`run`] so the
/// deadline is testable with a bound a test can actually wait out.
pub(crate) fn execute(command: &str, deadline: Duration) -> Capture {
    execute_with("sh", command, deadline)
}

/// The interpreter is a parameter so the arm where there is no interpreter at
/// all is reachable from a test. It is not a knob: [`execute`] is the only
/// caller that ships, and the shell it names is the one every Android device
/// has.
pub(crate) fn execute_with(program: &str, command: &str, deadline: Duration) -> Capture {
    let spawned = Command::new(program)
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let mut child = match spawned {
        Ok(child) => child,
        Err(e) => return refused(NO_SHELL, &format!("could not run a shell: {e}")),
    };
    // Both pipes are drained on their own threads: a child that fills one
    // nobody reads blocks forever, and the wait below would then never end.
    let out = child.stdout.take().map(drain);
    let err = child.stderr.take().map(drain);
    let code = wait(&mut child, deadline);
    let (stdout, stderr) = (joined(out), joined(err));
    match code {
        Some(code) => Capture {
            stdout,
            stderr,
            exit_code: code,
        },
        None => Capture {
            stdout,
            stderr: format!("{stderr}the command was still running after {deadline:?}\n"),
            exit_code: TIMED_OUT,
        },
    }
}

/// Read one pipe to its end, off the thread that is waiting.
fn drain<R: Read + Send + 'static>(mut pipe: R) -> std::thread::JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = pipe.read_to_end(&mut bytes);
        bytes
    })
}

/// The drained bytes as the text a capture carries. Bytes stop being bytes
/// here, once; output that is not UTF-8 loses exactly the bytes no `String`
/// can name, which is the trade every capture already makes.
fn joined(reader: Option<std::thread::JoinHandle<Vec<u8>>>) -> String {
    reader
        .and_then(|handle| handle.join().ok())
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default()
}

/// Wait for the child, up to `deadline`; `None` is a child that outran it and
/// was killed. The kill is what makes the pipe readers finish, which is what
/// makes the caller's `join` return.
fn wait(child: &mut std::process::Child, deadline: Duration) -> Option<i32> {
    let started = Instant::now();
    loop {
        // A wait that FAILS is read as a child that has not finished, which
        // costs one deadline and then the kill below. The alternative — its
        // own arm and its own code — is a branch no test can reach (the
        // failure is a reaped-elsewhere child, and nothing in this process
        // reaps), and a branch that cannot be tested must not be built.
        if let Ok(Some(status)) = child.try_wait() {
            return Some(code_of(status));
        }
        if started.elapsed() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        std::thread::sleep(POLL);
    }
}

/// A status as the number a shell would report: the exit code, or the signal
/// in the shell's own `128 + n` convention when a signal ended it.
fn code_of(status: std::process::ExitStatus) -> i32 {
    status.code().unwrap_or_else(|| signalled(status))
}

fn signalled(status: std::process::ExitStatus) -> i32 {
    use std::os::unix::process::ExitStatusExt;
    status.signal().map_or(NO_SHELL, |n| 128 + n)
}
