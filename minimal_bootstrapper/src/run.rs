//! The scheduler, and what happens when an operation runs.
//!
//! Two rules decide everything:
//!
//!   * dependencies are honoured first, and a failure skips what depended on it
//!   * a lane is a barrier: everything in the lowest ready lane runs together,
//!     then the next one
//!
//! Concurrency is opt-in. With no `parallel =` anywhere, operations run one at
//! a time, which is what a laptop on battery wants.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::config::{Error, Lane, Wait};
use crate::dotfiles;
use crate::os::{self, Run};
use crate::plan::{Body, Op};

/// How one item ended up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Already in the desired state.
    Kept,
    /// Changed by this run.
    Changed,
    /// Could not be changed.
    Failed,
}

impl State {
    pub fn symbol(self) -> &'static str {
        match self {
            State::Kept => "✓",
            State::Changed => "+",
            State::Failed => "✗",
        }
    }
}

/// One line of output about one thing.
#[derive(Debug)]
pub struct Line {
    pub name: String,
    pub state: State,
    pub note: String,
}

/// What an operation did.
#[derive(Debug, Default)]
pub struct Report {
    pub lines: Vec<Line>,
    /// Set when the operation failed as a whole, e.g. `dnf` exited non-zero.
    pub error: Option<String>,
    /// Set when the operation never ran, and why.
    pub skipped: Option<String>,
}

impl Report {
    /// An operation that did nothing and had nothing to report.
    pub fn new() -> Self {
        Report::default()
    }

    fn add(&mut self, state: State, name: impl Into<String>, note: impl Into<String>) {
        self.lines.push(Line {
            name: name.into(),
            state,
            note: note.into(),
        });
    }

    /// An item that was already correct.
    pub fn kept(&mut self, name: impl Into<String>, note: impl Into<String>) {
        self.add(State::Kept, name, note);
    }

    /// An item this run changed.
    pub fn changed(&mut self, name: impl Into<String>, note: impl Into<String>) {
        self.add(State::Changed, name, note);
    }

    /// An item that could not be changed.
    pub fn missed(&mut self, name: impl Into<String>, note: impl Into<String>) {
        self.add(State::Failed, name, note);
    }

    /// An operation that never ran, because something it needed failed first.
    pub fn skip(why: impl Into<String>) -> Self {
        Report {
            skipped: Some(why.into()),
            ..Report::new()
        }
    }

    /// Did this operation end up in the state it wanted?
    pub fn converged(&self) -> bool {
        self.error.is_none() && !self.lines.iter().any(|l| l.state == State::Failed)
    }

    /// Its symbol.
    pub fn symbol(&self) -> &'static str {
        if self.converged() { State::Kept.symbol() } else { State::Failed.symbol() }
    }
}

/// Counts for the final summary.
///
/// `kept` and `changed` count items — packages, commands, files. `failed` and
/// `skipped` count operations, because an operation is the thing that either
/// happened or did not.
#[derive(Debug, Default, Clone, Copy)]
pub struct Summary {
    pub kept: usize,
    pub changed: usize,
    pub failed: usize,
    pub skipped: usize,
}

/// Everything a run needs that is not in the plan.
pub struct Ctx {
    pub run: Run,
    pub home: PathBuf,
    /// `check` and `plan` report; only `apply` changes anything.
    pub mutating: bool,
}

/// Run every operation and collect the reports.
///
/// The engine never prints. Everything it learns arrives through `on_done`, so
/// the presentation is the caller's business and the scheduler stays testable.
pub fn run<G>(ops: &[&Op], ctx: &Ctx, mut on_done: G) -> Result<Summary, Error>
where
    G: FnMut(&Op, &Report),
{
    let ids: BTreeSet<&str> = ops.iter().map(|op| op.id.as_str()).collect();
    let mut settled: HashMap<&str, Option<String>> = HashMap::new();
    let mut summary = Summary::default();

    loop {
        // Operations whose dependencies have finished but did not succeed are
        // skipped, and recorded at once so nothing waits on them for ever.
        for op in ops {
            if settled.contains_key(op.id.as_str()) {
                continue;
            }
            let Some(why) = blame(&wanted(op, &ids), &settled) else {
                continue;
            };
            on_done(op, &Report::skip(why));
            summary.skipped += 1;
            settled.insert(&op.id, None);
        }

        let ready: Vec<&Op> = ops
            .iter()
            .copied()
            .filter(|op| {
                !settled.contains_key(op.id.as_str())
                    && wanted(op, &ids)
                        .iter()
                        .all(|d| settled.contains_key(d.as_str()))
            })
            .collect();

        let Some(lane) = ready.iter().map(|op| op.lane.clone()).min() else {
            break;
        };

        let wave = take_wave(&ready, &lane);
        for (op, report) in wave.iter().zip(execute(&wave, ctx)) {
            on_done(op, &report);
            // kept and changed count *items*; failed counts *operations*, so a
            // dnf that never ran cannot hide behind zero reported items.
            for line in &report.lines {
                match line.state {
                    State::Kept => summary.kept += 1,
                    State::Changed => summary.changed += 1,
                    State::Failed => {}
                }
            }
            summary.failed += usize::from(!report.converged());
            settled.insert(&op.id, report.error.clone());
        }
    }

    Ok(summary)
}

/// The dependencies of `op` that are part of this run.
///
/// A dependency on an operation that was filtered out by `--only` or `--from` is
/// out of scope, not unsatisfiable. `--from exec.bun` means "everything before
/// this is done", and treating the excluded predecessor as missing would
/// deadlock on the first wave.
fn wanted<'a>(op: &'a Op, ids: &BTreeSet<&str>) -> Vec<&'a String> {
    op.after
        .iter()
        .filter(|dep| ids.contains(dep.as_str()))
        .collect()
}

/// Dependencies this run is relying on something else to have done.
pub fn out_of_scope<'a>(op: &'a Op, ids: &BTreeSet<&str>) -> Vec<&'a String> {
    op.after
        .iter()
        .filter(|dep| !ids.contains(dep.as_str()))
        .collect()
}

/// Why this operation cannot run, if it cannot.
///
/// `None` means "not decided yet": some dependency has not finished.
fn blame(deps: &[&String], settled: &HashMap<&str, Option<String>>) -> Option<String> {
    let mut blamed: Option<String> = None;
    for dep in deps {
        match settled.get(dep.as_str()) {
            None => return None,
            Some(Some(error)) if blamed.is_none() => {
                blamed = Some(format!("{dep} failed: {error}"));
            }
            _ => {}
        }
    }
    blamed
}

/// The operations in `lane` that can start now, minus resource conflicts.
fn take_wave<'a>(ready: &[&'a Op], lane: &Lane) -> Vec<&'a Op> {
    let mut held: HashSet<&str> = HashSet::new();
    ready
        .iter()
        .filter(|op| &op.lane == lane)
        .filter(|op| match op.body.resource() {
            // Already held this run: leave it for the next wave.
            Some(resource) => held.insert(resource),
            None => true,
        })
        .copied()
        .collect()
}

/// Run a wave, in parallel when the config asked for concurrency.
fn execute(wave: &[&Op], ctx: &Ctx) -> Vec<Report> {
    let concurrent = wave.len() > 1 && wave.iter().any(|op| op.lane_declared);

    if !concurrent {
        return wave.iter().map(|op| one(op, ctx)).collect();
    }

    std::thread::scope(|scope| {
        // Spawn every one first, then wait. Joining inside the loop that
        // spawns would make the wave serial and quietly defeat the whole point.
        let handles: Vec<_> = wave.iter().map(|op| scope.spawn(|| one(op, ctx))).collect();
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|_| failed("the worker thread panicked"))
            })
            .collect()
    })
}

fn failed(message: &str) -> Report {
    Report {
        error: Some(message.to_string()),
        ..Report::new()
    }
}

/// Check one operation, and make it true if the mode allows.
fn one(op: &Op, ctx: &Ctx) -> Report {
    // Reporting is read-only: ask, then stop.
    let report = match &op.body {
        Body::Packages { installer, names } => packages(op, installer, names, ctx),
        Body::Command { check, run } => command(op, check.as_deref(), run, ctx),
        Body::Repo(repo) => self::repo(op, repo, ctx),
        Body::Dotfiles(dotfiles) => return dotfiles_body(dotfiles, ctx),
        Body::Wait(wait) => return waiting(op, wait),
    };

    match report {
        Ok(report) => report,
        Err(error) => failed(&error),
    }
}

/// Install whichever packages are missing.
///
/// In `check` mode nothing is installed, but everything missing is still
/// reported: a checker that stays quiet about the one thing you asked it to
/// look for is worse than no checker.
fn packages(
    op: &Op,
    installer: &crate::config::Installer,
    names: &[String],
    ctx: &Ctx,
) -> Result<Report, String> {
    let mut report = Report::new();
    let mut missing = Vec::new();

    for name in names {
        if os::shell(&installer.check_one(name))? {
            report.kept(name.clone(), "installed");
        } else {
            missing.push(name.clone());
        }
    }

    if missing.is_empty() {
        return Ok(report);
    }
    if !ctx.mutating {
        for name in &missing {
            report.missed(name.clone(), "missing");
        }
        return Ok(report);
    }

    // One transaction for everything missing: dependency resolution is the
    // expensive part, so doing it per package is pure waste.
    let script = installer.install_many(&missing);
    let ok = logged(&op.id, ctx, &script, installer.sudo)?;
    if !ok {
        return Err(format!("`{script}` failed; see the run log"));
    }

    // Re-check each one: an installer exiting 0 is not proof of what was asked.
    for name in &missing {
        if os::shell(&installer.check_one(name))? {
            report.changed(name.clone(), "installed");
        } else {
            report.missed(name.clone(), "still not installed");
        }
    }
    Ok(report)
}

/// Run `run`, unless `check` says it is already done.
fn command(op: &Op, check: Option<&str>, run: &str, ctx: &Ctx) -> Result<Report, String> {
    let mut report = Report::new();

    // Without a check there is no way to know whether this has to run, so it
    // is treated as needed and said out loud rather than quietly assumed done.
    let needed = match check {
        Some(check) => !os::shell(check)?,
        None => true,
    };

    if !needed {
        report.kept(op.id.clone(), "already correct");
        return Ok(report);
    }

    if !ctx.mutating {
        let note = match check {
            Some(_) => "would run",
            None => "no check: would run every time",
        };
        report.missed(op.id.clone(), note);
        return Ok(report);
    }

    if !logged(&op.id, ctx, run, op.sudo)? {
        return Err(format!("`{run}` failed; see the run log"));
    }
    report.changed(op.id.clone(), "done");
    Ok(report)
}

/// Write a `.repo` file and import its key.
fn repo(op: &Op, repo: &crate::config::Repo, ctx: &Ctx) -> Result<Report, String> {
    let name = repo.name.clone().unwrap_or_else(|| op.id.split('.').nth(1).unwrap_or("").to_string());
    let path = format!("/etc/yum.repos.d/{name}.repo");
    let mut report = Report::new();

    let mut body = format!(
        "[{name}]\nname={name}\nbaseurl={}\nenabled={}\ngpgcheck={}",
        repo.url,
        u8::from(repo.enabled),
        u8::from(repo.gpgcheck),
    );
    if let Some(key) = &repo.key {
        body.push_str(&format!("\ngpgkey={key}"));
    }

    if let Some(parent) = Path::new(&path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let already = std::fs::read_to_string(&path).is_ok_and(|current| current == body);
    if already {
        report.kept(name.clone(), "enabled");
        return Ok(report);
    }
    if !ctx.mutating {
        report.missed(name, "would be written");
        return Ok(report);
    }

    // `tee` keeps the write inside the escalated command, so this process never
    // holds a root file descriptor.
    let ok = logged(
        &op.id,
        ctx,
        &format!("printf '%s' {} | sudo tee {path} >/dev/null", quote(&body)),
        true,
    )?;
    if !ok {
        return Err(format!("could not write {path}; see the run log"));
    }

    if let Some(key) = &repo.key {
        let imported = logged(&op.id, ctx, &format!("rpm --import {key}"), true)?;
        if !imported {
            return Err(format!("could not import the GPG key {key}; see the run log"));
        }
    }

    report.changed(name.clone(), "enabled");
    Ok(report)
}

fn quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', r"'\''"))
}

/// Clone and stow dotfiles.
fn dotfiles_body(config: &crate::config::Dotfiles, ctx: &Ctx) -> Report {
    if dotfiles::converged(config, &ctx.home) {
        let mut report = Report::new();
        report.kept("repository", "up to date");
        return report;
    }
    if !ctx.mutating {
        let mut report = Report::new();
        report.missed("repository", "not stowed yet");
        return report;
    }
    dotfiles::run(config, &ctx.home, &ctx.run).unwrap_or_else(|e| failed(&e))
}

/// Sleep, and say so.
///
/// The only reason this body exists is to pace a run and to give the scheduler
/// something with a known duration to test against.
fn waiting(op: &Op, wait: &Wait) -> Report {
    let duration = wait.duration();
    std::thread::sleep(duration);

    let mut report = Report::new();
    report.kept(&op.id, format!("waited {duration:?}"));
    report
}

/// Run a command into the run log and wait for it.
fn logged(op_id: &str, ctx: &Ctx, script: &str, sudo: bool) -> Result<bool, String> {
    let mut child = ctx
        .run
        .spawn(script, sudo, &format!("{}.log", op_id.replace('.', "-")))?;
    Ok(child.wait().map_err(|e| e.to_string())?.success())
}

/// Reject an `after =` that names nothing, before the run starts.
pub fn validate(ops: &[Op]) -> Result<(), Error> {
    let ids: BTreeSet<&str> = ops.iter().map(|op| op.id.as_str()).collect();
    match ops.iter().find_map(|op| {
        op.after
            .iter()
            .find(|dep| !ids.contains(dep.as_str()))
            .map(|missing| format!("`{}` depends on `{missing}`, which is not an operation", op.id))
    }) {
        Some(message) => Err(Error(message)),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;
    use crate::config::Config;
    use crate::plan;

    /// A scratch directory, and the context that points at it.
    ///
    /// Operations really run, so tests need somewhere of their own to write.
    fn scratch(name: &str) -> (PathBuf, Ctx) {
        let home = std::env::temp_dir().join(format!("minimal-run-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).expect("home");
        let run = Run::at(home.join("run"));
        std::fs::create_dir_all(run.dir()).expect("run dir");
        let ctx = Ctx {
            run,
            home: home.clone(),
            mutating: true,
        };
        (home, ctx)
    }

    /// Build operations from a config written against `$HOME`.
    ///
    /// The placeholder is swapped for a real directory rather than exported
    /// into the environment: `$HOME` is process-global and these tests share a
    /// process.
    fn plan_for(home: &Path, text: &str) -> Vec<Op> {
        let config = Config::load(&text.replace("$HOME", &home.to_string_lossy())).expect("config");
        plan::build(&config).expect("plan")
    }

    /// The engine runs a *selection*, which is a slice of references.
    fn run_all(ops: &[Op], ctx: &Ctx) -> Summary {
        let selected: Vec<&Op> = ops.iter().collect();
        run(&selected, ctx, |_, _| {}).expect("run")
    }

    #[test]
    fn a_lane_runs_to_completion_before_the_next_one_starts() {
        let (home, ctx) = scratch("barrier");
        let ops = plan_for(
            &home,
            r#"
            [installer.first]
            check = "test -f $HOME/{package}"
            install = "cd $HOME && touch {packages}"

            [installer.second]
            check = "test -f $HOME/{package}"
            install = "cd $HOME && touch {packages}"

            [packages.first]
            parallel = 1
            names = ["a", "b"]

            [packages.second]
            parallel = 2
            names = ["c"]
            after = ["packages.first"]
            "#,
        );

        let summary = run_all(&ops, &ctx);

        assert_eq!(summary.failed, 0, "{summary:?}");
        for name in ["a", "b", "c"] {
            assert!(home.join(name).exists(), "{name} should have been created");
        }
    }

    #[test]
    fn concurrency_is_opt_in() {
        // Nothing declares a lane, so the run is serial even though the
        // operations are independent: a laptop on battery should not have four
        // things happening at once.
        let (home, ctx) = scratch("serial");
        let ops = plan_for(
            &home,
            r#"
            [exec.a]
            run = "sleep 0.2 && touch $HOME/a"

            [exec.b]
            run = "sleep 0.2 && touch $HOME/b"
            "#,
        );
        let started = Instant::now();

        run_all(&ops, &ctx);

        assert!(started.elapsed() >= Duration::from_millis(400), "they overlapped");
    }

    #[test]
    fn a_declared_lane_runs_its_operations_together() {
        let (home, ctx) = scratch("together");
        let ops = plan_for(
            &home,
            r#"
            [exec.a]
            run = "sleep 0.2 && touch $HOME/a"
            parallel = 1

            [exec.b]
            run = "sleep 0.2 && touch $HOME/b"
            parallel = 1
            "#,
        );
        let started = Instant::now();

        run_all(&ops, &ctx);

        assert!(
            started.elapsed() < Duration::from_millis(400),
            "two 0.2s operations in one lane should overlap"
        );
        assert!(home.join("a").exists() && home.join("b").exists());
    }

    #[test]
    fn a_shared_resource_never_overlaps() {
        // Both claim `rpm`, so the second waits for the first however the lanes
        // are written: dnf and rpm --import both take the rpmdb lock.
        let (home, ctx) = scratch("resource");
        let ops = plan_for(
            &home,
            r#"
            [installer.slow]
            check = "false"
            install = "sleep 0.2 && cd $HOME && touch {packages}"
            resource = "rpm"

            [installer.slow2]
            check = "false"
            install = "sleep 0.2 && cd $HOME && touch {packages}"
            resource = "rpm"

            [packages.slow]
            parallel = 1
            names = ["a"]

            [packages.slow2]
            parallel = 1
            names = ["b"]
            "#,
        );
        let started = Instant::now();

        run_all(&ops, &ctx);

        assert!(home.join("a").exists() && home.join("b").exists());
        assert!(
            started.elapsed() >= Duration::from_millis(400),
            "two 0.2s installs holding one token cannot overlap"
        );
    }

    #[test]
    fn a_failure_skips_what_depended_on_it() {
        let (home, ctx) = scratch("skip");
        let ops = plan_for(
            &home,
            r#"
            [exec.explode]
            run = "exit 3"

            [exec.downstream]
            run = "touch $HOME/should-not-exist"
            after = ["exec.explode"]
            "#,
        );

        let summary = run_all(&ops, &ctx);

        assert_eq!(summary.skipped, 1, "the dependent is skipped: {summary:?}");
        assert!(!home.join("should-not-exist").exists());
    }

    #[test]
    fn check_mode_changes_nothing() {
        let (home, mut ctx) = scratch("readonly");
        ctx.mutating = false;
        let ops = plan_for(
            &home,
            "[exec.write]\ncheck = \"false\"\nrun = \"touch $HOME/written\"\n",
        );

        run_all(&ops, &ctx);

        assert!(!home.join("written").exists(), "check must not change anything");
    }

    #[test]
    fn a_package_is_installed_once_and_then_kept() {
        let (home, ctx) = scratch("idempotent");
        let ops = plan_for(
            &home,
            r#"
            [installer.marked]
            check = "test -f $HOME/{package}"
            install = "cd $HOME && touch {packages}"

            [packages.marked]
            parallel = 1
            names = ["marker"]
            "#,
        );

        let first = run_all(&ops, &ctx);
        assert_eq!((first.kept, first.changed), (0, 1), "{first:?}");

        let second = run_all(&ops, &ctx);
        assert_eq!((second.kept, second.changed), (1, 0), "{second:?}");
    }

    #[test]
    fn an_installer_that_lies_is_not_believed() {
        // A `check` that keeps failing after a successful install is reported,
        // not hidden behind a zero exit status.
        let (home, ctx) = scratch("liar");
        let ops = plan_for(
            &home,
            r#"
            [installer.liar]
            check = "false"
            install = "true"

            [packages.liar]
            parallel = 1
            names = ["thing"]
            "#,
        );

        let summary = run_all(&ops, &ctx);

        assert_eq!(summary.failed, 1, "the package is still not installed: {summary:?}");
        let _ = home;
    }

    #[test]
    fn check_reports_what_is_missing_rather_than_staying_quiet() {
        // A checker that says nothing about the one thing you asked it to look
        // for is worse than no checker at all.
        let (home, mut ctx) = scratch("checkmode");
        ctx.mutating = false;
        let ops = plan_for(
            &home,
            r#"
            [installer.marked]
            check = "test -f $HOME/{package}"
            install = "cd $HOME && touch {packages}"

            [packages.marked]
            parallel = 1
            names = ["absent", "also-absent"]

            [exec.unguarded]
            run = "cd $HOME && touch $HOME/ran"

            [exec.guarded]
            check = "true"
            run = "cd $HOME && touch $HOME/never"
            "#,
        );

        let summary = run_all(&ops, &ctx);

        assert_eq!(
            summary.failed, 2,
            "two operations are short of what they want: {summary:?}"
        );
        assert_eq!(summary.kept, 1, "the guarded command is already correct");
        assert_eq!(summary.changed, 0);
        assert!(!home.join("ran").exists(), "check must not run anything");
    }

    #[test]
    fn check_says_so_when_everything_is_already_right() {
        let (home, mut ctx) = scratch("converged");
        ctx.mutating = false;
        let ops = plan_for(
            &home,
            r#"
            [installer.marked]
            check = "test -f $HOME/{package}"
            install = "cd $HOME && touch {packages}"

            [packages.marked]
            parallel = 1
            names = ["present"]
            "#,
        );
        std::fs::write(home.join("present"), "").unwrap();

        let summary = run_all(&ops, &ctx);

        assert_eq!((summary.kept, summary.failed), (1, 0), "{summary:?}");
    }

    #[test]
    fn an_after_naming_nothing_is_rejected_up_front() {
        let (home, _ctx) = scratch("baddep");
        let ops = plan_for(&home, "[exec.a]\nrun = \"true\"\nafter = [\"exec.missing\"]\n");

        let err = validate(&ops).unwrap_err();

        assert!(err.0.contains("exec.missing"), "{err}");
    }

    #[test]
    fn a_cycle_terminates_instead_of_spinning() {
        // Two operations each declared as a prerequisite of the other. Nothing
        // is ready, so nothing runs — but the loop must still end.
        let (home, ctx) = scratch("cycle");
        let mut ops = plan_for(&home, "[exec.a]\nrun = \"true\"\n");
        ops[0].after = vec!["exec.b".into()];
        ops.push(Op {
            id: "exec.b".into(),
            body: Body::Command {
                check: None,
                run: "true".into(),
            },
            lane: Lane::default(),
            after: vec!["exec.a".into()],
            sudo: false,
            lane_declared: false,
        });
        let selected: Vec<&Op> = ops.iter().collect();

        let summary = run(&selected, &ctx, |_, _| {}).expect("terminates");

        assert_eq!(summary.kept + summary.changed, 0, "{summary:?}");
    }

    #[test]
    fn waiting_returns_and_reports() {
        let (home, ctx) = scratch("wait");
        let ops = plan_for(&home, "[wait.short]\nseconds = 0.05\n");

        let summary = run_all(&ops, &ctx);

        assert_eq!(summary.kept, 1, "{summary:?}");
    }
}
