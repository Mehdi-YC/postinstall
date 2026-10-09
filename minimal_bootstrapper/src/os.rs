//! Running commands: escalation, logging, and the run directory.
//!
//! Everything the tool executes goes through here, so three rules hold
//! everywhere:
//!
//!   * privilege escalation is per command, never for the whole tool
//!   * stdout and stderr go to a log file, never to our terminal
//!   * nothing noisy panics: a missing command is an ordinary outcome

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
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

/// Whether `--sudo` was asked for. Set once, at startup.
static SUDO: AtomicBool = AtomicBool::new(false);

pub fn set_sudo(on: bool) {
    SUDO.store(on, Ordering::Relaxed);
}

/// Build a command, escalating it if the operation needs root.
///
/// Under `--sudo` the credential was refreshed by [`prime_sudo`] up front, and
/// `-n` guarantees a worker thread can never stop on an invisible password
/// prompt: it runs or it fails, immediately.
pub fn command(program: &str, sudo: bool) -> Command {
    if !(sudo && !is_root()) {
        return Command::new(program);
    }
    let mut cmd = Command::new("sudo");
    if SUDO.load(Ordering::Relaxed) {
        cmd.arg("-n");
    }
    cmd.arg(program);
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
    status
        .success()
        .then_some(())
        .ok_or_else(|| "`sudo -v` failed: authentication rejected".to_string())
}

/// Does this binary exist? `PATH` is the contract, so ask the shell.
pub fn have(binary: &str) -> bool {
    shell(&format!("command -v {binary}")).is_ok_and(|success| success)
}

/// Run a shell command, discarding its output. Success is all we need.
pub fn shell(script: &str) -> Result<bool, String> {
    Command::new("sh")
        .arg("-c")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .map_err(|e| format!("{script}: {e}"))
}

/// Expand a leading `~` using `$HOME`.
pub fn home() -> PathBuf {
    std::env::var("HOME").map_or_else(|_| PathBuf::from("/"), PathBuf::from)
}

/// The directory one run writes everything into.
pub struct Run {
    dir: PathBuf,
}

impl Run {
    /// Create a timestamped directory under the state directory.
    pub fn create() -> Result<Self, String> {
        let base = std::env::var("XDG_STATE_HOME").map_or_else(|_| home().join(".local/state"), PathBuf::from);
        let dir = base.join("minimal_bootstrapper/runs").join(stamp());
        fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        Ok(Run { dir })
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
            .open(self.dir.join("run.log"))
        {
            let _ = writeln!(file, "{line}");
        }
    }

    /// Start a command with both streams redirected into `name`'s log file.
    pub fn spawn(&self, script: &str, sudo: bool, name: &str) -> Result<Child, String> {
        let file = File::create(self.dir.join(name)).map_err(|e| format!("{name}: {e}"))?;
        let err = file.try_clone().map_err(|e| e.to_string())?;
        command("sh", sudo)
            .args(["-c", script])
            .stdin(Stdio::null())
            .stdout(Stdio::from(file))
            .stderr(Stdio::from(err))
            .spawn()
            .map_err(|e| format!("{script}: {e}"))
    }
}

/// Take secret values back out of the logs.
///
/// Withholding a secret from the run log is not enough on its own: the value can
/// reach a log through a command that echoed it, or through a tool that printed
/// its own configuration. So the logs are swept once the run is over.
///
/// Values shorter than [`MIN_SECRET`] are skipped. Replacing a two-character
/// secret everywhere would corrupt the logs and hide nothing useful.
const MIN_SECRET: usize = 8;

/// The value to put in place of a secret.
pub const REDACTED: &str = "(withheld)";

/// Rewrite every file under `dir` with the secrets removed.
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
            // one copy of a log.
            let staged = path.with_extension("redacting");
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
    fn a_secret_is_taken_back_out_of_the_log() {
        let dir = log_with("token=hunter2secret\nall done\n", "redact");

        redact(&dir, &["hunter2secret".to_string()]);

        assert_eq!(fs::read_to_string(dir.join("op.log")).unwrap(), format!("token={REDACTED}\nall done\n"));
    }

    #[test]
    fn a_short_value_is_left_alone() {
        // Replacing a two-character secret would corrupt every log and hide
        // nothing, so short values are not treated as secrets.
        let dir = log_with("ok ok ok\n", "short");

        redact(&dir, &["ok".to_string()]);

        assert_eq!(fs::read_to_string(dir.join("op.log")).unwrap(), "ok ok ok\n");
    }

    #[test]
    fn an_unrelated_log_is_left_byte_for_byte() {
        let dir = log_with("nothing secret here\n", "untouched");

        redact(&dir, &["hunter2secret".to_string()]);

        assert_eq!(fs::read_to_string(dir.join("op.log")).unwrap(), "nothing secret here\n");
        assert!(!dir.join("op.redacting").exists(), "no staging file left behind");
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
