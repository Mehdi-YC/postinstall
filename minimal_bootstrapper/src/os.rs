//! Running commands: escalation, logging, and the run directory.
//!
//! Everything the tool executes goes through here, so three rules hold
//! everywhere:
//!
//!   * privilege escalation is per command, never for the whole tool
//!   * stdout and stderr go to a log file, never to our terminal
//!   * nothing noisy panics: a missing command is an ordinary outcome

use std::fs::{self, DirBuilder};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::OnceLock;

/// True when we are already root, so `sudo` would be pointless.
pub fn is_root() -> bool {
    static ROOT: OnceLock<bool> = OnceLock::new();
    *ROOT.get_or_init(|| {
        Command::new("id")
            .arg("-u")
            .output()
            .is_ok_and(|out| String::from_utf8_lossy(&out.stdout).trim() == "0")
    })
}

/// Build a command, escalating it if the operation needs root.
///
/// Escalation always uses `sudo -n`. The tool refuses up front to run an
/// escalated operation unless `--sudo` was passed (or it is already root), so
/// the sudo credential is always fresh by the time this is called. `-n` is what
/// keeps that promise: without it a worker thread could open `/dev/tty` and
/// stop on a password prompt whose text went into a log file.
pub fn command(program: &str, sudo: bool) -> Command {
    if !(sudo && !is_root()) {
        return Command::new(program);
    }
    let mut cmd = Command::new("sudo");
    cmd.arg("-n").arg(program);
    cmd
}

/// Ask for the password once, here, where the terminal is.
///
/// This is the whole reason to run the tool as yourself: one prompt up front
/// instead of one per escalated command, buried in a log file.
pub fn prime_sudo() -> Result<(), String> {
    if is_root() {
        return Ok(());
    }
    if !have("sudo") {
        return Err("`sudo` is not installed; install it or run as root".into());
    }
    let status = Command::new("sudo")
        .arg("-v")
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("`sudo`: {e}"))?;
    status.success().then_some(()).ok_or_else(|| {
        // Not "rejected": with no terminal, or a policy that refuses, the
        // password was never even read.
        "`sudo -v` failed, so nothing can be escalated: there was no terminal to \
         read the password on, or sudo refused. Run this from your own terminal."
            .to_string()
    })
}

/// Does this binary exist? `PATH` is the contract, so ask the shell.
pub fn have(binary: &str) -> bool {
    shell(&format!("command -v {binary}")).is_ok_and(|success| success)
}

/// Run a shell command, discarding its output. Success is all we need.
pub fn shell(script: &str) -> Result<bool, String> {
    shell_as(script, false)
}

/// Like [`shell`], under the operation's own privilege.
///
/// A `check` that needs root has to run as root too, or it fails as the user
/// and the operation it guards runs on every single apply. The script itself is
/// never echoed in the error: it may contain a secret that has yet to be
/// redacted from stdout.
pub fn shell_as(script: &str, sudo: bool) -> Result<bool, String> {
    command("sh", sudo)
        .arg("-c")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .map_err(|e| format!("could not run a check: {e}"))
}

/// The user's home directory.
///
/// `HOME` rather than a password-database lookup, because the whole point of
/// `--sudo` is that operations still run as the person who typed the command.
pub fn home() -> PathBuf {
    std::env::var("HOME").map_or_else(|_| PathBuf::from("/"), PathBuf::from)
}

/// The directory one run writes everything into.
pub struct Run {
    dir: PathBuf,
}

impl Run {
    /// Create a timestamped directory under the system temp directory.
    ///
    /// A suffix is added if the stamp is already taken, so two runs in the same
    /// second cannot truncate each other's logs.
    ///
    /// The directory is private (`0700`) and so are the logs (`0600`): a shared
    /// `/tmp` is not a place to leave a file that can still hold a secret
    /// between the command and the redaction sweep. A base someone else already
    /// made readable is refused rather than written into.
    pub fn create() -> Result<Self, String> {
        let base = std::env::temp_dir().join("minimal_bootstrapper");
        DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&base)
            .map_err(|e| format!("{}: {e}", base.display()))?;
        // A directory somebody else pre-created is refused, not written into.
        let private = fs::symlink_metadata(&base)
            .is_ok_and(|meta| meta.file_type().is_dir() && meta.permissions().mode() & 0o077 == 0);
        if !private {
            return Err(format!(
                "{} is not a private directory; remove it and run again",
                base.display()
            ));
        }

        let runs = base.join("runs");
        DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&runs)
            .map_err(|e| format!("{}: {e}", runs.display()))?;
        let stamp = stamp();
        for attempt in 0..100 {
            let name = match attempt {
                0 => stamp.clone(),
                n => format!("{stamp}-{n}"),
            };
            let dir = runs.join(name);
            match DirBuilder::new().mode(0o700).create(&dir) {
                Ok(()) => return Ok(Run { dir }),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(format!("{}: {e}", dir.display())),
            }
        }
        Err(format!(
            "{}: too many runs in one second; giving up",
            runs.display()
        ))
    }

    /// A run rooted at a directory the caller chose.
    #[cfg(test)]
    pub fn at(dir: PathBuf) -> Self {
        Run { dir }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Append a line to the run's own log.
    pub fn note(&self, line: &str) {
        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .mode(0o600)
            .open(self.dir.join("run.log"))
        {
            let _ = writeln!(file, "{line}");
        }
    }

    /// Start a command with both streams redirected into `name`'s log file.
    pub fn spawn(&self, script: &str, sudo: bool, name: &str) -> Result<Child, String> {
        let file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(self.dir.join(name))
            .map_err(|e| format!("{name}: {e}"))?;
        let err = file.try_clone().map_err(|e| e.to_string())?;
        command("sh", sudo)
            .args(["-c", script])
            .stdin(Stdio::null())
            .stdout(Stdio::from(file))
            .stderr(Stdio::from(err))
            .spawn()
            .map_err(|e| format!("could not start `sh`: {e}"))
    }
}

/// A secret shorter than this is not treated as one.
///
/// Replacing a two-character value everywhere would corrupt every log it
/// appears in and hide nothing, so short values are left alone.
const MIN_SECRET: usize = 8;

/// What a secret is replaced with.
pub const REDACTED: &str = "(withheld)";

/// Rewrite every file in `dir` with the secret values removed.
///
/// Withholding a secret from the run log is not enough on its own: the value can
/// still reach a log through a command that echoed it, or through a tool that
/// printed its own configuration. So the logs are swept once the run is over.
pub fn redact(dir: &Path, secrets: &[String]) {
    let needles: Vec<&String> = secrets
        .iter()
        .filter(|value| value.len() >= MIN_SECRET)
        .collect();
    if needles.is_empty() {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue; // not UTF-8: leave it rather than corrupt it
        };
        let mut clean = text.clone();
        for needle in &needles {
            clean = clean.replace(needle.as_str(), REDACTED);
        }
        if clean != text {
            // Write beside and rename, so a crash mid-sweep cannot truncate the
            // one copy of a log. The staging name is the file's own name plus a
            // suffix, so two files sharing a stem cannot stage over each other.
            let mut staged_name = path.file_name().unwrap_or_default().to_os_string();
            staged_name.push(".redacting");
            let staged = path.with_file_name(staged_name);
            if fs::write(&staged, clean).is_ok() && fs::rename(&staged, &path).is_err() {
                let _ = fs::remove_file(&staged);
            }
        }
    }
}

/// `YYYY-MM-DD-HHMMSS`, UTC.
///
/// Howard Hinnant's `civil_from_days`, small enough to keep inline and not
/// worth a date crate.
fn stamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    let days = secs.div_euclid(86_400);

    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);

    let midnight = secs.rem_euclid(86_400);
    format!(
        "{year:04}-{month:02}-{day:02}-{:02}{:02}{:02}",
        midnight / 3600,
        midnight % 3600 / 60,
        midnight % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn log_with(text: &str, name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("minimal-os-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("op.log");
        fs::write(&path, text).unwrap();
        dir
    }

    #[test]
    fn a_run_lives_in_the_temp_directory_and_is_private() {
        // /tmp is shared, and a log can hold a secret until the sweep.
        let run = Run::create().expect("run");
        assert!(
            run.dir().starts_with(std::env::temp_dir()),
            "{} is not under the temp directory",
            run.dir().display()
        );

        let mode = |path: &Path| fs::metadata(path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode(run.dir()), 0o700, "the run directory must be private");

        run.note("hello");
        let mut child = run.spawn("true", false, "op.log").expect("spawn");
        child.wait().unwrap();

        for name in ["run.log", "op.log"] {
            assert_eq!(
                mode(&run.dir().join(name)),
                0o600,
                "{name} must not be world readable"
            );
        }
        let _ = fs::remove_dir_all(run.dir());
    }

    #[test]
    fn a_secret_is_taken_back_out_of_the_log() {
        let dir = log_with("token=hunter2secret\nall done\n", "redact");

        redact(&dir, &["hunter2secret".to_string()]);

        assert_eq!(
            fs::read_to_string(dir.join("op.log")).unwrap(),
            format!("token={REDACTED}\nall done\n")
        );
    }

    #[test]
    fn a_short_value_is_left_alone() {
        // Replacing a two-character secret would corrupt every log and hide
        // nothing, so short values are not treated as secrets.
        let dir = log_with("ok ok ok\n", "short");

        redact(&dir, &["ok".to_string()]);

        assert_eq!(
            fs::read_to_string(dir.join("op.log")).unwrap(),
            "ok ok ok\n"
        );
    }

    #[test]
    fn an_unrelated_log_is_left_byte_for_byte() {
        let dir = log_with("nothing secret here\n", "untouched");

        redact(&dir, &["hunter2secret".to_string()]);

        assert_eq!(
            fs::read_to_string(dir.join("op.log")).unwrap(),
            "nothing secret here\n"
        );
        assert!(
            !dir.join("op.log.redacting").exists(),
            "no staging file left behind"
        );
    }

    #[test]
    fn the_last_line_is_redacted_too() {
        let dir = log_with("no newline at the end: hunter2secret", "nonewline");

        redact(&dir, &["hunter2secret".to_string()]);

        assert_eq!(
            fs::read_to_string(dir.join("op.log")).unwrap(),
            format!("no newline at the end: {REDACTED}")
        );
    }
}
