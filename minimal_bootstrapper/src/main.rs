//! minimal_bootstrapper — make the machine match a file.
//!
//! ```sh
//! bootstrapper list          # what would run, in what order
//! bootstrapper check         # what is missing; changes nothing
//! bootstrapper apply --sudo  # make it so, escalating one command at a time
//! ```
//!
//! The whole idea in one sentence: the configuration declares what should be
//! true, and the tool works out the difference and closes it. Everything else —
//! lanes, dependencies, privilege escalation — exists to make that safe to run
//! twice, on a laptop, while something else is using the network.

/// `println!` that treats a closed pipe as normal.
///
/// `bootstrapper check | head` is a reasonable thing to type, and `println!`
/// panics the moment the reader goes away — printing a Rust backtrace after the
/// output you asked for. A pipeline that stops reading is not an error.
macro_rules! say {
    ($($arg:tt)*) => {{
        use std::io::Write;
        let _ = writeln!(std::io::stdout().lock(), $($arg)*);
    }};
}

mod config;
mod dotfiles;
mod os;
mod plan;
mod run;

use std::collections::BTreeSet;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use config::{Error, Lane};
use run::{Report, State, Summary};

const USAGE: &str = "\
minimal_bootstrapper — declarative workstation configurator

USAGE:
    bootstrapper [check|plan|apply|list] [OPTIONS]

COMMANDS:
    check     report what is missing. Changes nothing. (`plan` is a synonym.)
    apply     make the system match the configuration.
    list      the operations, their lanes and their dependencies.

OPTIONS:
    -c, --config <PATH>    configuration file
                           (default: ./bootstrap.toml, then
                            ~/.config/bootstrapper/bootstrap.toml)
        --only <TAGS>      comma separated operation ids or suffixes
        --from <ID>        start at this operation and continue
        --sudo             run as yourself, escalate one command at a time.
                           Asks for the password once, up front. Do not run
                           the tool as root: that puts cargo and uv into /root.
    -h, --help             this text
";

fn main() -> ExitCode {
    match go(std::env::args().skip(1).collect()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}

/// What the command line asked for.
///
/// `check` and `plan` are two names for one behaviour — run everything, change
/// nothing — so they share a variant and a verb.
#[derive(Debug, PartialEq, Eq)]
enum Command {
    Check,
    Apply,
    List,
    Help,
}

/// The parsed command line.
struct Args {
    command: Command,
    config: Option<PathBuf>,
    only: Vec<String>,
    from: Option<String>,
    sudo: bool,
}

impl Args {
    /// Parse, or say what is wrong with the arguments.
    ///
    /// No printing and no exiting: `--help` is a [`Command`] like any other, so
    /// this stays a pure function of its input.
    fn parse(argv: &[String]) -> Result<Self, String> {
        let mut args = Args {
            command: Command::Apply,
            config: None,
            only: Vec::new(),
            from: None,
            sudo: false,
        };
        let mut rest = argv.iter();

        while let Some(arg) = rest.next() {
            match arg.as_str() {
                "check" | "plan" => args.command = Command::Check,
                "apply" => args.command = Command::Apply,
                "list" => args.command = Command::List,
                "--help" | "-h" => args.command = Command::Help,
                "--sudo" => args.sudo = true,
                "-c" | "--config" => {
                    let path = rest.next().ok_or(format!("{arg} needs a path"))?;
                    args.config = Some(path.into());
                }
                "--only" => {
                    let value = rest.next().ok_or("--only needs tags")?;
                    args.only.extend(
                        value
                            .split(',')
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .map(str::to_string),
                    );
                }
                "--from" => args.from = Some(rest.next().ok_or("--from needs an id")?.clone()),
                other => return Err(format!("unknown argument `{other}`; try --help")),
            }
        }
        Ok(args)
    }

    fn path(&self) -> PathBuf {
        self.config.clone().unwrap_or_else(default_path)
    }
}

/// The config to use when `--config` is not given.
///
/// The working directory wins, so a checkout is self-contained; the XDG path is
/// the fallback for a machine-wide install.
fn default_path() -> PathBuf {
    let local = Path::new("bootstrap.toml");
    if local.exists() {
        return local.to_path_buf();
    }
    let base =
        std::env::var("XDG_CONFIG_HOME").map_or_else(|_| os::home().join(".config"), Into::into);
    let global = base.join("bootstrapper/bootstrap.toml");
    if global.exists() {
        global
    } else {
        local.to_path_buf()
    }
}

fn go(argv: Vec<String>) -> Result<ExitCode, Error> {
    let args = Args::parse(&argv).map_err(Error)?;
    if args.command == Command::Help {
        say!("{USAGE}");
        return Ok(ExitCode::SUCCESS);
    }

    let path = args.path();
    // Every error carries the file name: it is the one thing the reader needs,
    // and no layer below this one knows it.
    let where_ = path.display();
    let text = std::fs::read_to_string(&path).map_err(|e| Error(format!("{where_}: {e}")))?;

    let config = config::Config::load(&text).map_err(|e| Error(format!("{where_}: {e}")))?;
    let all = plan::build(&config).map_err(|e| Error(format!("{where_}: {e}")))?;

    let ops = select(&all, &args);
    if ops.is_empty() {
        return Err(Error("nothing selected; check --only".into()));
    }
    // Validated against the whole plan, so a typo in `after =` is caught even
    // when `--only` happens to leave that operation out.
    run::validate(&all).map_err(|e| Error(format!("{where_}: {e}")))?;

    // Dependencies outside the selection are assumed already done. Say which,
    // so `--from` is never a quiet way to skip a step.
    let selected: BTreeSet<&str> = ops.iter().map(|op| op.id.as_str()).collect();
    let assumed: BTreeSet<&str> = ops
        .iter()
        .flat_map(|op| run::out_of_scope(op, &selected))
        .map(String::as_str)
        .collect();
    if !assumed.is_empty() {
        say!(
            "assuming already done: {}",
            assumed.into_iter().collect::<Vec<_>>().join(", ")
        );
    }

    if args.command == Command::List {
        return list(&ops, &config);
    }

    // `--sudo` while already root is a contradiction: the point of the flag is
    // to run as yourself, and as root `$HOME` is /root, which is where the
    // user-space tools would end up.
    if args.sudo && os::is_root() {
        return Err(Error(
            "--sudo means \"run as yourself, escalate per operation\", but this process is already \
             root, so every operation runs as root and cargo and uv install into /root.\n\
             Drop the sudo: run `bootstrapper apply` as yourself."
                .into(),
        ));
    }
    if args.sudo {
        os::set_sudo(true);
        os::prime_sudo()?;
    }

    let run = os::Run::create()?;
    let mutating = args.command == Command::Apply;
    let ctx = run::Ctx {
        run,
        home: os::home(),
        mutating,
    };

    for (key, value) in &config.variables {
        let shown = if config.is_secret(key) {
            "(withheld: named in `secret`)"
        } else {
            value.as_str()
        };
        ctx.run.note(&format!("var {key}={shown}"));
    }
    for op in &ops {
        ctx.run.note(&format!("{} sudo={}", op.id, op.sudo));
    }

    let style = Style::detect();
    say!();
    say!(
        "{} {} {}",
        style.loud(verb(&args.command)),
        style.dim("·"),
        style.dim(&format!("{} operation(s)", ops.len()))
    );

    // Reported per operation as one contiguous block, with a rule whenever the
    // lane changes. Lanes are barriers, so the completion stream is already
    // grouped by lane — which is the one thing about the wave structure that was
    // previously invisible.
    let mut lane: Option<Lane> = None;
    let summary = run::run(&ops, &ctx, |op, report| {
        if lane.as_ref() != Some(&op.lane) {
            if lane.is_some() {
                say!();
            }
            lane = Some(op.lane.clone());
            say!("{}", style.rule(&format!("lane {}", op.lane)));
        }

        match &report.skipped {
            Some(why) => say!(
                "  {} {}",
                style.dim("-"),
                style.dim(&format!("{}  {why}", op.id))
            ),
            // An operation with nothing to say beyond itself is one line. `vscode`
            // under `repos.vscode` is the same name twice, so a name that is
            // already the tail of the id does not earn its own row.
            None if report.lines.len() == 1 && repeats_itself(&report.lines[0].name, &op.id) => {
                say!(
                    "  {} {}  {}",
                    report_symbol(&style, report),
                    op.id,
                    style.dim(&report.lines[0].note)
                )
            }
            None => {
                say!("  {} {}", report_symbol(&style, report), op.id);
                for line in &report.lines {
                    say!(
                        "      {} {}  {}",
                        style.symbol(line.state),
                        line.name,
                        style.dim(&line.note)
                    );
                }
            }
        }
        if let Some(error) = &report.error {
            say!("      {}", style.failed(error));
        }
    });

    os::redact(ctx.run.dir(), &secret_values(&config));

    say!();
    say!("{}", style.rule(""));
    say!("{}", summary_line(summary, mutating, &style));
    say!(
        "{} {}",
        style.dim("logs"),
        style.dim(&ctx.run.dir().display().to_string())
    );
    Ok(ExitCode::from(u8::from(summary.failed > 0)))
}

/// Colour, but only where a person can see it.
///
/// `IsTerminal` is std, so knowing whether this is a pipe costs no dependency.
/// Colour in a log file or a `grep` result is noise, and `NO_COLOR` is a
/// convention worth honouring rather than fighting.
struct Style {
    on: bool,
    /// How wide a rule may be. `COLUMNS` is a convention every shell honours, and
    /// a fixed width would wrap in a narrow terminal, which is worse than no
    /// rule at all.
    width: usize,
}

impl Style {
    fn detect() -> Self {
        let columns = std::env::var("COLUMNS")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(72)
            .clamp(32, 100);
        Style {
            on: std::io::stdout().is_terminal()
                && std::env::var_os("NO_COLOR").is_none()
                && std::env::var("TERM").map_or(true, |term| term != "dumb"),
            width: columns,
        }
    }

    fn paint(&self, code: &str, text: &str) -> String {
        if self.on {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }

    fn kept(&self, text: &str) -> String {
        self.paint("32", text)
    }

    fn changed(&self, text: &str) -> String {
        self.paint("33", text)
    }

    fn failed(&self, text: &str) -> String {
        self.paint("31", text)
    }

    fn dim(&self, text: &str) -> String {
        self.paint("2", text)
    }

    fn loud(&self, text: &str) -> String {
        self.paint("1", text)
    }

    /// A status symbol, in the colour that matches its state.
    fn symbol(&self, state: State) -> String {
        let mark = state.symbol();
        match state {
            State::Kept => self.kept(mark),
            State::Changed => self.changed(mark),
            State::Failed => self.failed(mark),
        }
    }

    /// A horizontal rule, optionally titled.
    fn rule(&self, title: &str) -> String {
        let (label, used) = match title.is_empty() {
            true => (String::new(), 0),
            false => (format!(" {title} "), title.chars().count() + 2),
        };
        self.dim(&format!(
            "──{label}{}",
            "─".repeat(self.width.saturating_sub(2 + used))
        ))
    }
}

/// Is this item just the operation's own name, spelled the short way?
fn repeats_itself(name: &str, id: &str) -> bool {
    name == id || id.ends_with(&format!(".{name}"))
}

/// An operation's own mark, coloured by whether it converged.
fn report_symbol(style: &Style, report: &Report) -> String {
    if report.converged() {
        style.kept(State::Kept.symbol())
    } else {
        style.failed(State::Failed.symbol())
    }
}

/// The values of the variables named in `secret`.
fn secret_values(config: &config::Config) -> Vec<String> {
    config
        .variables
        .iter()
        .filter(|(name, _)| config.is_secret(name))
        .map(|(_, value)| value.clone())
        .collect()
}

fn verb(command: &Command) -> &'static str {
    match command {
        Command::Check => "check",
        Command::Apply => "apply",
        Command::List => "list",
        Command::Help => "help",
    }
}

fn summary_line(summary: Summary, mutating: bool, style: &Style) -> String {
    // Skipped is a fourth column, not a replacement for failed: a run can have
    // both, and overwriting one with the other hid the failures.
    let widest = [
        summary.kept,
        summary.changed,
        summary.failed,
        summary.skipped,
    ]
    .iter()
    .map(usize::to_string)
    .map(|n| n.len())
    .max()
    .unwrap_or(1);

    let cell = |mark: &str, count: usize, name: &str| format!("{mark} {count:>widest$} {name}");
    let mut columns = Vec::new();
    if summary.kept > 0 {
        columns.push(cell(&style.symbol(State::Kept), summary.kept, "kept"));
    }
    if summary.changed > 0 {
        columns.push(cell(
            &style.symbol(State::Changed),
            summary.changed,
            "changed",
        ));
    }
    if summary.failed > 0 {
        columns.push(cell(&style.symbol(State::Failed), summary.failed, "failed"));
    }
    if summary.skipped > 0 {
        columns.push(cell(&style.dim("✓"), summary.skipped, "skipped"));
    }
    let rendered = columns.join("   ");

    let settled = summary.failed == 0 && summary.changed == 0 && summary.skipped == 0;
    match (mutating, settled) {
        (true, true) => format!("{rendered}\n{}", style.kept("everything is configured")),
        (false, true) => format!("{rendered}\n{}", style.dim("nothing was changed")),
        _ => rendered,
    }
}

/// Apply `--only` and `--from`.
fn select<'a>(all: &'a [plan::Op], args: &Args) -> Vec<&'a plan::Op> {
    let selected: Vec<&plan::Op> = if args.only.is_empty() {
        all.iter().collect()
    } else {
        all.iter()
            .filter(|op| {
                args.only
                    .iter()
                    .any(|want| op.id == *want || op.id.ends_with(&format!(".{want}")))
            })
            .collect()
    };

    let Some(from) = &args.from else {
        return selected;
    };
    let Some(start) = selected.iter().position(|op| &op.id == from) else {
        return selected;
    };
    selected[start..].to_vec()
}

/// `list`: what would run, and in what order.
fn list(ops: &[&plan::Op], config: &config::Config) -> Result<ExitCode, Error> {
    let style = Style::detect();
    say!();
    for op in ops {
        say!("  {}", style.loud(&op.id));
        say!("      {}", style.dim(&plan::describe(op)));
        say!("      {}", style.dim(&format!("lane {}", op.lane)));
        if op.sudo {
            say!("      {}", style.changed("sudo"));
        }
        if !op.after.is_empty() {
            say!(
                "      {}",
                style.dim(&format!("after {}", op.after.join(", ")))
            );
        }
        if let Some(resource) = op.body.resource() {
            say!(
                "      {}",
                style.dim(&format!("resource {resource} (exclusive)"))
            );
        }
    }

    say!();
    say!(
        "{}",
        style.dim(&format!("{} operation(s) selected", ops.len()))
    );

    // An installer nothing installs through reads as enabled and never runs,
    // which is the kind of thing you only notice six months later.
    let used: Vec<String> = config.packages.keys().cloned().collect();
    let unused = plan::unused_installers(config, &used);
    if !unused.is_empty() {
        say!();
        say!(
            "{} {}",
            style.changed("declared but unused:"),
            unused.join(", ")
        );
    }

    let lanes: std::collections::BTreeSet<_> = ops.iter().map(|op| op.lane.clone()).collect();
    if lanes.is_empty() {
        say!("no lanes");
    } else {
        let lanes: Vec<String> = lanes.into_iter().map(|l| l.to_string()).collect();
        say!("{}", style.dim(&format!("lanes: {}", lanes.join(", "))));
    }
    say!("{}", style.dim("run `bootstrapper apply` to make it so"));
    say!();
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(argv: &[&str]) -> Result<Args, String> {
        Args::parse(&argv.iter().map(|s| (*s).to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn defaults_to_apply() {
        assert_eq!(args(&[]).unwrap().command, Command::Apply);
        assert!(!args(&[]).unwrap().sudo, "escalation is never implied");
    }

    #[test]
    fn parses_the_flags() {
        let a = args(&["check", "--sudo", "--only", "dnf,uv", "--from", "exec.b"]).unwrap();
        assert_eq!(a.command, Command::Check);
        assert!(a.sudo);
        assert_eq!(a.only, vec!["dnf", "uv"]);
        assert_eq!(a.from.as_deref(), Some("exec.b"));
    }

    /// Style with colour off, which is what a pipe or `NO_COLOR` sees.
    fn plain() -> Style {
        Style {
            on: false,
            width: 72,
        }
    }

    #[test]
    fn the_summary_shows_failures_and_skips_side_by_side() {
        // Overwriting one with the other hid the failures, which is the one
        // thing the last line of output exists to say.
        let line = summary_line(
            Summary {
                kept: 1,
                changed: 2,
                failed: 3,
                skipped: 4,
            },
            true,
            &plain(),
        );

        assert!(line.contains("3 failed"), "{line}");
        assert!(line.contains("4 skipped"), "{line}");
        assert!(!line.contains("everything is configured"), "{line}");
    }

    #[test]
    fn a_clean_apply_says_so() {
        let line = summary_line(
            Summary {
                kept: 5,
                ..Summary::default()
            },
            true,
            &plain(),
        );
        assert!(line.ends_with("everything is configured"), "{line}");
    }

    #[test]
    fn a_clean_check_does_not_claim_to_have_configured_anything() {
        // `check` changes nothing, so "everything is configured" would be a lie.
        let line = summary_line(
            Summary {
                kept: 5,
                ..Summary::default()
            },
            false,
            &plain(),
        );
        assert!(!line.contains("everything is configured"), "{line}");
    }

    #[test]
    fn help_is_a_command_rather_than_an_exit() {
        // If parsing exited, it could not be tested at all.
        assert_eq!(args(&["--help"]).unwrap().command, Command::Help);
        assert_eq!(args(&["-h"]).unwrap().command, Command::Help);
    }

    #[test]
    fn a_config_path_keeps_its_shape() {
        let a = args(&["-c", "../playbooks/fedora.toml"]).unwrap();
        assert_eq!(a.path(), PathBuf::from("../playbooks/fedora.toml"));
    }

    #[test]
    fn rejects_nonsense() {
        assert!(args(&["--wat"]).is_err());
        assert!(args(&["--only"]).is_err());
        assert!(args(&["--config"]).is_err());
    }

    #[test]
    fn selects_by_full_id_or_suffix() {
        let config = config::Config::load(
            r#"
            [installer.x]
            check = "true"
            install = "true"
            [packages.x]
            names = ["a"]
            [exec.tool]
            run = "true"
            "#,
        )
        .unwrap();
        let all = plan::build(&config).unwrap();

        let by_suffix = select(&all, &args(&["--only", "x"]).unwrap());
        assert_eq!(by_suffix.len(), 1);
        assert_eq!(by_suffix[0].id, "packages.x");

        let by_full = select(&all, &args(&["--only", "packages.x"]).unwrap());
        assert_eq!(by_full.len(), 1);

        let none = select(&all, &args(&["--only", "nope"]).unwrap());
        assert!(none.is_empty(), "an unmatched tag selects nothing");
    }

    #[test]
    fn from_starts_at_that_operation_and_keeps_the_rest() {
        let config = config::Config::load(
            r#"
            [exec.a]
            run = "true"
            [exec.b]
            run = "true"
            [exec.c]
            run = "true"
            "#,
        )
        .unwrap();
        let all = plan::build(&config).unwrap();
        let picked = select(&all, &args(&["--from", "exec.b"]).unwrap());
        assert_eq!(
            picked.iter().map(|o| o.id.as_str()).collect::<Vec<_>>(),
            vec!["exec.b", "exec.c"]
        );
    }
}
