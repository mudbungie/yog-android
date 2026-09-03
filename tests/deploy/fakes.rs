//! **A phone-less box for `scripts/deploy-phone.sh`.** The deploy target's
//! three real tools — `make`, `gradle` and `adb` — are the three things a gate
//! run may not have: no NDK, no SDK, no device. So the harness builds a whole
//! world out of scripts instead, and the target runs against it unmodified.
//!
//! **`PATH` is CONSTRUCTED, not inherited**, and that is what makes the
//! resolution order testable at all. The environment is cleared and `PATH`
//! holds exactly one directory: this fixture's `bin/`, carrying the fakes plus
//! a symlink to each real tool the script itself spends (`bash` for the
//! shebang's `env`, `git` for the repo root, `ls`/`sort`/`tail` for the dists
//! probe). A box that happens to have a real `gradle` installed therefore
//! cannot make the "nothing on PATH" arm pass by accident — the arm is a fact
//! about the fixture, not about the machine running the suite.
//!
//! `HOME` points into the fixture for the same reason: the SDK default and the
//! gradle wrapper cache both hang off it, so an operator's real SDK is never
//! what a test found.
//!
//! Every fake writes its own argv to one log, in order, so the assertions can
//! read what the target DID — which command it built with, whether it reached
//! `adb` at all after a failed build, whether it installed after a refused
//! connect. The knobs are environment variables the fakes read at run time:
//! the exit code and the sentence each tool answers with.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The address every case passes. A NAME and a port, never an address: a
/// routable one is a disclosure this repo's own gate refuses, and the target
/// hands whatever it is given straight to `adb`.
pub const ADDR: &str = "phone:5555";

const FAKE_MAKE: &str = r#"#!/usr/bin/env bash
printf 'make %s\n' "$*" >> "$DEPLOY_LOG"
[ -n "${MAKE_APK_TOUCH:-}" ] && : > "$MAKE_APK_TOUCH"
exit "${MAKE_CODE:-0}"
"#;

const FAKE_ADB: &str = r#"#!/usr/bin/env bash
printf 'adb %s\n' "$*" >> "$DEPLOY_LOG"
if [ "$1" = connect ]; then
  printf '%s\n' "$ADB_CONNECT_SAY"; exit "$ADB_CONNECT_CODE"
fi
printf '%s\n' "$ADB_INSTALL_SAY"; exit "$ADB_INSTALL_CODE"
"#;

const FAKE_GRADLE: &str = "#!/usr/bin/env bash\nexit 0\n";

/// What one run of the target answered: its exit code, what it said on stderr,
/// and the ordered log of every tool it spent.
pub struct Run {
    pub code: i32,
    pub err: String,
    pub log: String,
}

pub struct Fixture {
    pub root: PathBuf,
    env: BTreeMap<String, String>,
}

impl Fixture {
    /// A world with `make` fake and nothing else: no adb, no gradle anywhere.
    /// Each case adds back exactly the pieces its arm needs.
    pub fn new(name: &str) -> Result<Self, String> {
        let root = std::env::temp_dir().join(format!("yog-deploy-{name}"));
        drop(std::fs::remove_dir_all(&root));
        let bin = root.join("bin");
        mkdir(&bin)?;
        for tool in ["bash", "env", "git", "ls", "sort", "tail"] {
            let real = on_path(tool)?;
            std::os::unix::fs::symlink(real, bin.join(tool))
                .map_err(|why| format!("link {tool}: {why}"))?;
        }
        script(&bin.join("make"), FAKE_MAKE)?;
        let apk = root.join("app-debug.apk");
        let mut fixture = Self {
            root: root.clone(),
            env: BTreeMap::new(),
        };
        for (key, value) in [
            ("PATH", bin.display().to_string()),
            ("HOME", root.display().to_string()),
            ("DEPLOY_LOG", root.join("log").display().to_string()),
            ("ANDROID_HOME", root.join("sdk").display().to_string()),
            ("APK", apk.display().to_string()),
            ("MAKE_APK_TOUCH", apk.display().to_string()),
            ("ADB_CONNECT_SAY", "connected to the device".to_owned()),
            ("ADB_CONNECT_CODE", "0".to_owned()),
            ("ADB_INSTALL_SAY", "Success".to_owned()),
            ("ADB_INSTALL_CODE", "0".to_owned()),
        ] {
            fixture = fixture.set(key, &value);
        }
        Ok(fixture)
    }

    pub fn set(mut self, key: &str, value: &str) -> Self {
        self.env.insert(key.to_owned(), value.to_owned());
        self
    }

    /// An SDK at `$ANDROID_HOME` with a fake `adb` where the real one lives.
    pub fn with_adb(self) -> Result<Self, String> {
        let tools = self.root.join("sdk/platform-tools");
        mkdir(&tools)?;
        script(&tools.join("adb"), FAKE_ADB)?;
        Ok(self)
    }

    /// One gradle bin distribution in the wrapper cache, laid out exactly as
    /// the wrapper lays it out: `gradle-<v>-bin/<hash>/gradle-<v>/bin/gradle`.
    pub fn with_dist(self, version: &str) -> Result<Self, String> {
        let dir = self
            .root
            .join(".gradle/wrapper/dists")
            .join(format!("gradle-{version}-bin"))
            .join("aaaaaaaaaaaaaaaaaaaaaaaaa")
            .join(format!("gradle-{version}"))
            .join("bin");
        mkdir(&dir)?;
        script(&dir.join("gradle"), FAKE_GRADLE)?;
        Ok(self)
    }

    /// A `gradle` on the fixture's PATH.
    pub fn with_path_gradle(self) -> Result<Self, String> {
        script(&self.root.join("bin/gradle"), FAKE_GRADLE)?;
        Ok(self)
    }

    /// A gradle somewhere neither probe would ever look, for the arm that
    /// proves an explicit `GRADLE=` outranks both.
    pub fn with_named_gradle(self) -> Result<Self, String> {
        let dir = self.root.join("elsewhere");
        mkdir(&dir)?;
        script(&dir.join("gradle"), FAKE_GRADLE)?;
        Ok(self.set("GRADLE", &dir.join("gradle").display().to_string()))
    }

    pub fn run(&self, args: &[&str]) -> Result<Run, String> {
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let out = Command::new(repo.join("scripts/deploy-phone.sh"))
            .current_dir(&repo)
            .env_clear()
            .envs(&self.env)
            .args(args)
            .output()
            .map_err(|why| format!("spawning the target: {why}"))?;
        Ok(Run {
            code: out.status.code().unwrap_or(-1),
            err: String::from_utf8_lossy(&out.stderr).into_owned(),
            log: std::fs::read_to_string(self.root.join("log")).unwrap_or_default(),
        })
    }
}

/// Every refusal answers the same two ways: non-zero, and a sentence naming
/// the fix. Asserting on both is the point — a target that dies silently and
/// one that dies loudly are the same exit code to a caller and very different
/// to a human.
pub fn refused(run: &Run, code: i32, says: &str) {
    assert_eq!(run.code, code, "exit code; it said: {}", run.err);
    assert!(run.err.contains(says), "expected {says:?} in: {}", run.err);
}

/// The lines of the tool log, which is the record of what the run DID.
pub fn spent(run: &Run) -> Vec<&str> {
    run.log.lines().collect()
}

fn mkdir(at: &Path) -> Result<(), String> {
    std::fs::create_dir_all(at).map_err(|why| format!("{}: {why}", at.display()))
}

fn script(at: &Path, body: &str) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::write(at, body).map_err(|why| format!("{}: {why}", at.display()))?;
    std::fs::set_permissions(at, std::fs::Permissions::from_mode(0o755))
        .map_err(|why| format!("{}: {why}", at.display()))
}

fn on_path(tool: &str) -> Result<PathBuf, String> {
    let path = std::env::var_os("PATH").ok_or_else(|| "no PATH to read".to_owned())?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(tool))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| format!("{tool} is not on this box's PATH"))
}
