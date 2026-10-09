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

mod config;
mod dotfiles;
mod os;
mod plan;
mod run;

use std::collections::BTreeSet;
use std::process::ExitCode;

use config::Error;
use run::{State, Summary};

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
}

/// The parsed command line.
struct Args {
    command: Command,
    config: Option<String>,
    only: Vec<String>,
    from: Option<String>,
    sudo: bool,
}

impl Args {
    fn parse(argv: &[String]) -> Result<Self, String> {
        let mut args = Args {
            command: Command::Apply,
            config: None,
            only: Vec::new(),
            from: None,
            sudo: false,
        };
        let mut argv = argv.iter();

        while let Some(arg) = argv.next() {
            match arg.as_str() {
                "check" | "plan" => args.command = Command::Check,
                "apply" => args.command = Command::Apply,
                "list" => args.command = Command::List,
                "--help" | "-h" => {
                    print!("{USAGE}");
                    std::process::exit(0);
                }
                "--sudo" => args.sudo = true,
                "-c" | "--config" => {
                    args.config = Some(argv.next().ok_or(format!("{arg} needs a path"))?.clone());
                }
                "--only" => {
                    let value = argv.next().ok_or("--only needs tags")?;
                    args.only
                        .extend(value.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string));
                }
                "--from" => args.from = Some(argv.next().ok_or("--from needs an id")?.clone()),
                other => return Err(format!("unknown argument `{other}`; try --help")),
            }
        }
        Ok(args)
    }

    fn path(&self) -> std::path::PathBuf {
        self.config.as_deref().map_or_else(default_path, Into::into)
    }
}

/// The config to use when `--config` is not given.
///
/// The working directory wins, so a checkout is self-contained; the XDG path is
/// the fallback for a machine-wide install.
fn default_path() -> std::path::PathBuf {
    let local = std::path::Path::new("bootstrap.toml");
    if local.exists() {
        return local.to_path_buf();
    }
    let base = std::env::var("XDG_CONFIG_HOME").map_or_else(|_| os::home().join(".config"), Into::into);
    let global = base.join("bootstrapper/bootstrap.toml");
    if global.exists() {
        global
    } else {
        local.to_path_buf()
    }
}

fn go(argv: Vec<String>) -> Result<ExitCode, Error> {
    let args = Args::parse(&argv).map_err(Error)?;

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
        println!("assuming already done: {}", assumed.into_iter().collect::<Vec<_>>().join(", "));
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

    println!();
    println!(
        "{} · {} operation(s) · {}",
        verb(&args.command),
        ops.len(),
        ctx.run.dir().display()
    );

    // Reported per operation, when it finishes, as one contiguous block.
    // Announcing the start instead would interleave: with a wave running
    // concurrently, a slow operation's heading ends up far from its results.
    let summary = run::run(&ops, &ctx, |op, report| match &report.skipped {
        Some(why) => println!("  - {}  skipped: {why}", op.id),
        None => {
            println!("  {} {}", report.symbol(), op.id);
            for line in &report.lines {
                println!("      {} {}  {}", line.state.symbol(), line.name, line.note);
            }
            if let Some(error) = &report.error {
                println!("      {error}");
            }
        }
    })?;

    // A variable named in `secret` is withheld from the run log above, but the
    // value can still reach a log through a command that printed it.
    os::redact(ctx.run.dir(), &secret_values(&config));

    println!();
    println!("{}", summary_line(summary, mutating));
    Ok(ExitCode::from(u8::from(summary.failed > 0)))
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
    }
}

fn summary_line(summary: Summary, mutating: bool) -> String {
    let mut parts = [
        ("kept", summary.kept, State::Kept),
        ("changed", summary.changed, State::Changed),
        ("failed", summary.failed, State::Failed),
    ];
    if summary.skipped > 0 {
        parts[2] = ("skipped", summary.skipped, State::Kept);
    }
    let rendered = parts
        .iter()
        .map(|(name, count, state)| format!("{} {count} {name}", state.symbol()))
        .collect::<Vec<_>>()
        .join("  ");

    match (mutating, summary.failed == 0 && summary.changed == 0 && summary.skipped == 0) {
        (true, true) => format!("{rendered}\neverything is configured"),
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
                args.only.iter().any(|want| op.id == *want || op.id.ends_with(&format!(".{want}")))
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
    println!();
    for op in ops {
        println!("  {}", op.id);
        println!("      {}", plan::describe(op));
        println!("      lane {}", op.lane);
        if op.sudo {
            println!("      sudo");
        }
        if !op.after.is_empty() {
            println!("      after {}", op.after.join(", "));
        }
        if let Some(resource) = op.body.resource() {
            println!("      resource {resource} (exclusive)");
        }
    }

    println!();
    println!("{} operation(s) selected", ops.len());

    // An installer nothing installs through reads as enabled and never runs,
    // which is the kind of thing you only notice six months later.
    let used: Vec<String> = config.packages.keys().cloned().collect();
    let unused = plan::unused_installers(config, &used);
    if !unused.is_empty() {
        println!();
        println!("declared but unused: {}", unused.join(", "));
    }

    let lanes: std::collections::BTreeSet<_> = ops.iter().map(|op| op.lane.clone()).collect();
    if lanes.is_empty() {
        println!("no lanes");
    } else {
        let lanes: Vec<String> = lanes.into_iter().map(|l| l.to_string()).collect();
        println!("lanes: {}", lanes.join(", "));
    }
    println!("run `bootstrapper apply` to make it so");
    println!();
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