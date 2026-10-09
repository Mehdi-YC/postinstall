//! The configuration file, and the operations it describes.
//!
//! ```toml
//! [variables]
//! mirror = "https://mirror.internal"
//!
//! [packages.dnf]
//! parallel = 2
//! names = ["git", "fish"]
//! ```
//!
//! `${name}` expands from `[variables]` or the environment. Installers use
//! `{package}` and `{packages}` instead: a different sigil, so the two can
//! never be confused for one another.

use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;
use std::time::Duration;

use serde::Deserialize;

/// Anything wrong with the configuration, said well enough to fix.
#[derive(Debug)]
pub struct Error(pub String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Error(message)
    }
}

/// Where in the lane graph an operation runs.
///
/// Segments are numbers and compare segment by segment, so `1.2` sorts between
/// `1.1` and `1.10`, which string comparison would get wrong.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Lane(Vec<u32>);

/// A lane as it appears in the file.
///
/// Both spellings are accepted because TOML has one obvious way to write each:
/// `parallel = 2` is an integer, and `parallel = "1.2"` has to be a string.
/// Asking people to quote the first one would be a trap.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum LaneSpec {
    Segment(u32),
    Dotted(String),
}

impl TryFrom<LaneSpec> for Lane {
    type Error = Error;

    fn try_from(spec: LaneSpec) -> Result<Self, Self::Error> {
        match spec {
            LaneSpec::Segment(n) => Ok(Lane(vec![n])),
            LaneSpec::Dotted(text) => text.parse(),
        }
    }
}

fn de_lane<'de, D: serde::Deserializer<'de>>(de: D) -> Result<Option<Lane>, D::Error> {
    Option::<LaneSpec>::deserialize(de)?
        .map(Lane::try_from)
        .transpose()
        .map_err(serde::de::Error::custom)
}

impl Default for Lane {
    /// Where operations land when the config says nothing: lane 1, the first.
    fn default() -> Self {
        Lane(vec![1])
    }
}

impl fmt::Display for Lane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let segments: Vec<String> = self.0.iter().map(u32::to_string).collect();
        f.write_str(&segments.join("."))
    }
}

impl FromStr for Lane {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let mut segments = Vec::new();
        for part in text.trim().split('.') {
            segments.push(
                part.parse()
                    .map_err(|_| Error(format!("`{text}` is not a lane; expected `2` or `1.2`")))?,
            );
        }
        Ok(Lane(segments))
    }
}

/// `parallel`, `sudo` and `after`, which every kind of operation accepts.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Schedule {
    /// The lane this operation runs in.
    #[serde(default, deserialize_with = "de_lane")]
    pub parallel: Option<Lane>,
    /// Overrides the default for this kind.
    #[serde(default)]
    pub sudo: Option<bool>,
    /// Operation ids that must succeed first.
    #[serde(default)]
    pub after: Vec<String>,
}

impl Schedule {
    /// The lane, falling back to the default.
    pub fn lane(&self) -> Lane {
        self.parallel.clone().unwrap_or_default()
    }
}

/// A package manager: how to ask about a package, how to install it.
#[derive(Debug, Clone, Deserialize)]
pub struct Installer {
    /// Run with `{package}` substituted. Exit status is the answer.
    pub check: String,
    /// Run with `{packages}` substituted, once, for everything missing.
    pub install: String,
    #[serde(default)]
    pub sudo: bool,
    /// Directories prepended to `PATH` for this installer's commands.
    /// Directories prepended to `PATH` for this installer's commands, for
    /// managers that install somewhere a login shell has not reached yet.
    #[serde(default)]
    pub path: Vec<String>,
    /// Exclusive token: operations sharing one never overlap.
    #[serde(default)]
    pub resource: Option<String>,
}

impl Installer {
    /// `check` for one package.
    pub fn check_one(&self, package: &str) -> String {
        self.with_path(&self.check.replace("{package}", &shell_quote(package)))
    }

    /// `install` for a set of packages.
    pub fn install_many(&self, packages: &[String]) -> String {
        let list = packages
            .iter()
            .map(|p| shell_quote(p))
            .collect::<Vec<_>>()
            .join(" ");
        self.with_path(&self.install.replace("{packages}", &list))
    }

    /// Prepend this installer's directories to `PATH`.
    ///
    /// `command -v` is how a `check` decides, so `PATH` *is* the contract: a
    /// manager installed under `~/.cargo/bin` has to be findable, or the check
    /// fails and the install runs again on every single apply.
    fn with_path(&self, script: &str) -> String {
        if self.path.is_empty() {
            return script.to_string();
        }
        let dirs: String = self
            .path
            .iter()
            .map(|dir| format!("{dir}:"))
            .collect::<String>();
        format!("PATH={dirs}$PATH {script}")
    }
}

/// One argument to `sh -c`, quoted.
///
/// Package names come from the user's own config, so this is hygiene rather
/// than a security boundary: it keeps `a; rm -rf /` one argument instead of two.
fn shell_quote(value: &str) -> String {
    let plain = !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-+@:/~".contains(c));
    if plain {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', r"'\''"))
    }
}

/// Packages to install with one installer.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Packages {
    #[serde(default)]
    pub names: Vec<String>,
    #[serde(flatten)]
    pub schedule: Schedule,
}

/// An imperative escape hatch. Idempotent only if you give it a `check`.
#[derive(Debug, Clone, Deserialize)]
pub struct Exec {
    pub run: String,
    /// If this succeeds, the operation is already done.
    pub check: Option<String>,
    #[serde(flatten)]
    pub schedule: Schedule,
}

/// A `.repo` file written into `/etc/yum.repos.d`.
#[derive(Debug, Clone, Deserialize)]
pub struct Repo {
    pub url: String,
    /// URL of the GPG key. Optional only with `gpgcheck = false`.
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default = "default_true")]
    pub gpgcheck: bool,
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Label in dnf's output; defaults to the section name.
    #[serde(default)]
    pub name: Option<String>,
    #[serde(flatten)]
    pub schedule: Schedule,
}

const fn default_true() -> bool {
    true
}

/// A git repository of dotfiles, stowed into `$HOME`.
#[derive(Debug, Clone, Deserialize)]
pub struct Dotfiles {
    pub repo: String,
    #[serde(default)]
    pub branch: Option<String>,
    /// Directories inside the repository to stow.
    #[serde(default)]
    pub stow: Vec<String>,
    #[serde(flatten)]
    pub schedule: Schedule,
}

/// A sleep, for pacing a run or exercising the scheduler.
#[derive(Debug, Clone, Deserialize)]
pub struct Wait {
    #[serde(default = "one_second")]
    pub seconds: f64,
    #[serde(flatten)]
    pub schedule: Schedule,
}

const fn one_second() -> f64 {
    1.0
}

impl Wait {
    pub fn duration(&self) -> Duration {
        Duration::from_secs_f64(self.seconds.max(0.0))
    }
}

/// The whole file.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub variables: BTreeMap<String, String>,
    /// Variables whose values must not reach the run log.
    #[serde(default)]
    pub secret: Vec<String>,
    #[serde(default)]
    pub installer: BTreeMap<String, Installer>,
    #[serde(default)]
    pub packages: BTreeMap<String, Packages>,
    #[serde(default)]
    pub exec: BTreeMap<String, Exec>,
    #[serde(default)]
    pub repos: BTreeMap<String, Repo>,
    #[serde(default)]
    pub wait: BTreeMap<String, Wait>,
    pub dotfiles: Option<Dotfiles>,
}

impl Config {
    /// Parse, then expand every `${...}`.
    pub fn load(text: &str) -> Result<Self, Error> {
        let mut config: Self = toml::from_str(text).map_err(|e| Error(e.to_string()))?;
        let vars = resolve(&config.variables)?;
        expand(&mut config, &vars);
        Ok(config)
    }

    /// Should this variable be kept out of the run log?
    pub fn is_secret(&self, name: &str) -> bool {
        self.secret.iter().any(|s| s == name)
    }
}

/// Resolve `[variables]`, in dependency order.
///
/// They are a map, so `b = ${a}/x` may be read before `a`. Resolution is by
/// dependency rather than position, which is what makes a composed URL work —
/// the reason to have a variable in a URL at all.
fn resolve(declared: &BTreeMap<String, String>) -> Result<BTreeMap<String, String>, Error> {
    let mut resolved = BTreeMap::new();
    let mut pending: Vec<&String> = declared.keys().collect();

    while !pending.is_empty() {
        let before = pending.len();
        pending.retain(|key| {
            let raw = declared[*key].trim();
            match whole_env_name(raw) {
                // `$NAME` on its own means the environment.
                Some(name) => match std::env::var(name) {
                    Ok(value) => {
                        resolved.insert((*key).clone(), value);
                        false
                    }
                    // The environment may be filled in later; not an error yet.
                    Err(_) => true,
                },
                None => match substitute(raw, &resolved) {
                    Ok(value) => {
                        resolved.insert((*key).clone(), value);
                        false
                    }
                    Err(_) => true,
                },
            }
        });

        if pending.len() == before {
            // Nothing moved, so either the remaining variables refer to each
            // other in a circle or they name something that does not exist. The
            // second is far more common, and `substitute` already knows how to
            // say which — so let it speak rather than reporting the variable
            // that merely *contains* the typo.
            let name = pending[0].as_str();
            let raw = declared[name].trim();
            let outcome = match whole_env_name(raw) {
                Some(env) => format!("`{name}` reads ${env}, which is not set"),
                None => substitute(raw, &resolved).unwrap_err().0,
            };
            return Err(Error(format!("{outcome} (in `{name}`)")));
        }
    }
    Ok(resolved)
}

/// A value that is exactly `$NAME`, meaning "read the environment".
///
/// The strictness is the point: `$release/$basearch` is a dnf URL fragment, and
/// guessing there would break every repository that uses them.
fn whole_env_name(value: &str) -> Option<&str> {
    let name = value.strip_prefix('$')?;
    let mut chars = name.chars();
    let first = chars.next()?;
    (first.is_ascii_alphabetic() || first == '_')
        .then(|| chars.all(|c| c.is_ascii_alphanumeric() || c == '_'))
        .filter(|ok| *ok)
        .map(|_| name)
}

/// Replace every `${name}` with its value.
///
/// An unknown name is an error, never an empty string: a `baseurl` of
/// `https:///repo` fails much later as an unexplained dnf error.
fn substitute(input: &str, vars: &BTreeMap<String, String>) -> Result<String, Error> {
    if !input.contains("${") {
        return Ok(input.to_string());
    }

    let mut out = String::with_capacity(input.len());
    let mut rest = input;

    while let Some((head, tail)) = rest.split_once("${") {
        out.push_str(head);
        let (name, after) = tail
            .split_once('}')
            .ok_or_else(|| Error(format!("unterminated `${{` in `{input}`")))?;
        let value = vars
            .get(name)
            .ok_or_else(|| Error(format!("undefined variable `${{{name}}}`")))?;
        out.push_str(value);
        rest = after;
    }
    out.push_str(rest);
    Ok(out)
}

/// Rewrite every variable reference in the config, in place.
///
/// Installer commands are deliberately left alone: their `{package}` is this
/// tool's own placeholder, substituted per command at run time.
fn expand(config: &mut Config, vars: &BTreeMap<String, String>) {
    for repo in config.repos.values_mut() {
        repo.url = substitute_lossy(&repo.url, vars);
        repo.key = repo.key.as_deref().map(|k| substitute_lossy(k, vars));
    }
    if let Some(dotfiles) = &mut config.dotfiles {
        dotfiles.repo = substitute_lossy(&dotfiles.repo, vars);
    }
    for exec in config.exec.values_mut() {
        exec.run = substitute_lossy(&exec.run, vars);
        exec.check = exec.check.as_deref().map(|c| substitute_lossy(c, vars));
    }
}

/// Expansion during a second pass, where every name is known to resolve.
///
/// `load` resolves `[variables]` first and errors on anything undefined, so by
/// the time the rest of the config is rewritten every reference in it resolves
/// too. The fallback keeps that property local instead of relying on ordering.
fn substitute_lossy(input: &str, vars: &BTreeMap<String, String>) -> String {
    substitute(input, vars).unwrap_or_else(|_| input.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lane(text: &str) -> Lane {
        text.parse().expect("a lane")
    }

    #[test]
    fn lanes_compare_segment_by_segment() {
        assert!(lane("1") < lane("1.1"));
        assert!(lane("1.2") < lane("1.10"), "1.2 comes before 1.10, not after");
        assert!(lane("1.10") < lane("2"));
    }

    #[test]
    fn a_lane_is_an_integer_or_a_dotted_string() {
        // Both spellings are natural in TOML, so both are accepted.
        let config = Config::load(
            r#"
            [exec.a]
            run = "true"
            parallel = 2

            [exec.b]
            run = "true"
            parallel = "1.3"
            "#,
        )
        .expect("config loads");

        assert_eq!(config.exec["a"].schedule.lane(), lane("2"));
        assert_eq!(config.exec["b"].schedule.lane(), lane("1.3"));
    }

    #[test]
    fn rejects_a_nonsense_lane() {
        let err = Config::load("[exec.a]\nrun = \"true\"\nparallel = \"first\"\n").unwrap_err();
        assert!(err.0.contains("not a lane"), "{err}");
    }

    #[test]
    fn variables_resolve_in_dependency_order() {
        // `b` is read before `a` is written, and `${}` forces resolution rather
        // than copying the literal, which is the whole reason to have a variable
        // inside a URL at all.
        let config = Config::load(
            r#"
            [variables]
            repo = "${base}/repo"
            base = "https://mirror.internal"

            [repos.internal]
            url = "${repo}/$release/$basearch"
            "#,
        )
        .expect("config loads");

        assert_eq!(
            config.repos["internal"].url,
            "https://mirror.internal/repo/$release/$basearch",
            "`$release` belongs to dnf and must survive untouched"
        );
    }

    #[test]
    fn a_whole_dollar_name_reads_the_environment() {
        assert_eq!(whole_env_name("$HOME"), Some("HOME"));
        assert_eq!(whole_env_name("$release/$basearch"), None);
        assert_eq!(whole_env_name("${HOME}"), None, "braces are the other form");
        assert_eq!(whole_env_name("$"), None);
    }

    #[test]
    fn an_undefined_variable_names_itself() {
        // Saying `cannot resolve url` would send the reader looking in the wrong
        // file: the typo is in the reference, not in the variable holding it.
        let err = Config::load("[variables]\nurl = \"${nowhere}\"\n").unwrap_err();
        assert!(err.0.contains("nowhere"), "{err}");
    }

    #[test]
    fn a_variable_cycle_is_reported_rather_than_looped_on() {
        let err = Config::load("[variables]\na = \"${b}\"\nb = \"${a}\"\n").unwrap_err();
        assert!(err.0.contains("a") && err.0.contains("b"), "{err}");
    }

    #[test]
    fn installer_placeholders_are_not_variables() {
        // Different sigils on purpose. `${package}` would be ambiguous with a
        // variable, so the installer's own placeholders are `{package}` and
        // expansion can never mistake one for the other.
        let config = Config::load(
            r#"
            [installer.fake]
            check = "test -f {package}"
            install = "touch {packages}"
            "#,
        )
        .expect("config loads");

        let fake = &config.installer["fake"];
        assert_eq!(fake.check_one("git"), "test -f git");
        assert_eq!(
            fake.install_many(&["a".into(), "b c".into()]),
            "touch a 'b c'",
            "a name with a space stays one argument"
        );
    }

    #[test]
    fn an_installer_extends_path_before_it_checks() {
        // `command -v` is how a check decides, so a binary outside the login
        // PATH would be reinstalled on every single run.
        let config = Config::load(
            r#"
            [installer.cargo]
            check = "command -v {package}"
            install = "cargo install {packages}"
            path = ["$HOME/.cargo/bin", "/opt/bin"]
            "#,
        )
        .expect("config loads");

        let cargo = &config.installer["cargo"];
        assert_eq!(
            cargo.check_one("ripgrep"),
            "PATH=$HOME/.cargo/bin:/opt/bin:$PATH command -v ripgrep"
        );
    }

    #[test]
    fn no_path_means_no_prefix() {
        let config = Config::load("[installer.x]\ncheck = \"c {package}\"\ninstall = \"i {packages}\"\n")
            .expect("config loads");
        assert_eq!(config.installer["x"].check_one("a"), "c a");
    }

    #[test]
    fn every_kind_accepts_the_same_schedule_keys() {
        let config = Config::load(
            r#"
            [exec.setup]
            run = "true"
            parallel = "2.1"
            sudo = true
            after = ["exec.other"]
            "#,
        )
        .expect("config loads");

        let setup = &config.exec["setup"];
        assert_eq!(setup.schedule.lane(), lane("2.1"));
        assert_eq!(setup.schedule.sudo, Some(true));
        assert_eq!(setup.schedule.after, vec!["exec.other"]);
    }

    #[test]
    fn secret_names_variables_that_stay_out_of_the_log() {
        let config = Config::load("secret = [\"token\"]\n\n[variables]\ntoken = \"t\"\n").unwrap();
        assert!(config.is_secret("token"));
        assert!(!config.is_secret("mirror"));
    }

    #[test]
    fn an_unknown_top_level_key_is_an_error() {
        // A typo should be caught while parsing, not by an operation that
        // quietly never runs.
        let err = Config::load("[packges.dnf]\nnames = [\"git\"]\n").unwrap_err();
        assert!(err.0.contains("packges"), "{err}");
    }

    #[test]
    fn a_wait_takes_fractional_seconds() {
        let config = Config::load("[wait.short]\nseconds = 0.25\n").unwrap();
        assert_eq!(config.wait["short"].duration(), Duration::from_millis(250));
    }

    #[test]
    fn a_config_with_nothing_in_it_is_valid() {
        // An empty file should be an empty plan, not an error.
        let ops = crate::plan::build(&Config::load("").unwrap()).unwrap();
        assert!(ops.is_empty());
    }
}
