//! Turning a configuration into a list of operations.
//!
//! Every section of every kind becomes the same `Op`, because every one of them
//! answers the same two questions: how do I check whether this is done, and how
//! do I make it done. That is the whole reason this crate is small.

use crate::config::{Config, Dotfiles, Error, Installer, Repo, Schedule, Wait};

/// One thing to do.
#[derive(Debug)]
pub struct Op {
    /// Stable identity, e.g. `packages.dnf` or `exec.bun`.
    pub id: String,
    /// What this operation does.
    pub body: Body,
    pub lane: crate::config::Lane,
    pub after: Vec<String>,
    pub sudo: bool,
    /// Whether the config said so, as opposed to it being a default.
    pub lane_declared: bool,
}

/// The shapes an operation can take.
///
/// Three of the four are the same shape wearing different names: check, then
/// fix. They stay separate because the *reporting* differs — a batch reports
/// one line per package, a wait reports nothing — and merging them would make
/// the interesting case worse to serve the trivial one.
#[derive(Debug)]
pub enum Body {
    /// Ask about each package, install whatever is missing.
    Packages {
        installer: Installer,
        names: Vec<String>,
    },
    /// Run `run` unless `check` says otherwise.
    Command { check: Option<String>, run: String },
    /// Write a `.repo` file and import its key.
    Repo(Repo),
    /// Clone or update a dotfiles repository and stow it into `$HOME`.
    Dotfiles(Dotfiles),
    /// Sleep. Exists to pace a run and to exercise the scheduler.
    Wait(Wait),
}

impl Body {
    /// The resource token this body claims, if any.
    ///
    /// Two operations holding one never overlap, whatever their lanes say:
    /// `dnf` and `rpm --import` both take the rpmdb lock.
    pub fn resource(&self) -> Option<&str> {
        match self {
            Body::Packages { installer, .. } => installer.resource.as_deref(),
            Body::Repo(_) => Some("rpm"),
            _ => None,
        }
    }
}

/// Read the configuration into a plan.
///
/// Returns the operations in a stable order, so `list` prints the same thing
/// twice and a diff of two runs means something.
pub fn build(config: &Config) -> Result<Vec<Op>, Error> {
    let mut ops = Vec::new();

    for (name, repo) in &config.repos {
        ops.push(op(
            format!("repos.{name}"),
            Body::Repo(repo.clone()),
            &repo.schedule,
            true,
        ));
    }

    for (manager, packages) in &config.packages {
        let installer = config.installer.get(manager).ok_or_else(|| {
            Error(format!(
                "no installer named `{manager}`; declare [installer.{manager}]"
            ))
        })?;
        ops.push(op(
            format!("packages.{manager}"),
            Body::Packages {
                installer: installer.clone(),
                names: packages.names.clone(),
            },
            &packages.schedule,
            // The installer's own setting is the default; the section may
            // override it either way.
            installer.sudo,
        ));
    }

    if let Some(dotfiles) = &config.dotfiles {
        ops.push(op(
            "dotfiles".into(),
            Body::Dotfiles(dotfiles.clone()),
            &dotfiles.schedule,
            false,
        ));
    }

    for (name, exec) in &config.exec {
        ops.push(op(
            format!("exec.{name}"),
            Body::Command {
                check: exec.check.clone(),
                run: exec.run.clone(),
            },
            &exec.schedule,
            false,
        ));
    }

    for (name, wait) in &config.wait {
        ops.push(op(
            format!("wait.{name}"),
            Body::Wait(wait.clone()),
            &wait.schedule,
            false,
        ));
    }

    ops.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(ops)
}

/// Build one operation, resolving its lane, escalation and dependencies.
fn op(id: String, body: Body, schedule: &Schedule, default_sudo: bool) -> Op {
    Op {
        id,
        body,
        lane: schedule.lane(),
        after: schedule.after.clone(),
        sudo: schedule.sudo.unwrap_or(default_sudo),
        lane_declared: schedule.parallel.is_some(),
    }
}

/// The installers a config actually uses.
///
/// Worth reporting on its own: an installer nothing installs through looks
/// enabled and never runs.
pub fn unused_installers<'a>(config: &'a Config, used: &[String]) -> Vec<&'a str> {
    config
        .installer
        .keys()
        .filter(|name| !used.iter().any(|u| u == *name))
        .map(String::as_str)
        .collect()
}

/// A short one-line description, for `list`.
pub fn describe(op: &Op) -> String {
    match &op.body {
        Body::Packages { names, .. } => format!("{} package(s)", names.len()),
        Body::Command { check: None, .. } => "run every time".into(),
        Body::Command { check: Some(_), .. } => "check, then run".into(),
        Body::Repo(_) => "write a .repo file".into(),
        Body::Dotfiles(d) => format!("stow {}", d.stow.join(", ")),
        Body::Wait(w) => format!("wait {}s", w.seconds),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Lane};

    fn ids(text: &str) -> Vec<String> {
        build(&Config::load(text).expect("config"))
            .expect("plan")
            .into_iter()
            .map(|o| o.id)
            .collect()
    }

    #[test]
    fn every_section_kind_becomes_an_operation() {
        let ops = ids(r#"
            [installer.fake]
            check = "true"
            install = "true"

            [repos.terra]
            url = "https://terra/$release/$basearch"
            key = "https://terra/key"

            [packages.fake]
            names = ["a", "b"]

            [exec.setup]
            run = "true"

            [wait.slow]
            seconds = 1

            [dotfiles]
            repo = "https://example.invalid/dots"
            stow = ["config"]
            "#);

        assert_eq!(
            ops,
            vec![
                "dotfiles",
                "exec.setup",
                "packages.fake",
                "repos.terra",
                "wait.slow"
            ]
        );
    }

    #[test]
    fn operations_come_out_in_a_stable_order() {
        // `list` twice must print the same thing, or a diff of two runs is noise.
        let text = "[installer.x]\ncheck = \"t\"\ninstall = \"t\"\n[packages.x]\nnames = [\"a\"]\n[exec.b]\nrun = \"t\"\n";
        assert_eq!(ids(text), ids(text));
    }

    #[test]
    fn a_missing_installer_names_the_section_to_write() {
        let err =
            build(&Config::load("[packages.pacman]\nnames = [\"bash\"]\n").unwrap()).unwrap_err();
        assert!(err.0.contains("[installer.pacman]"), "{err}");
    }

    #[test]
    fn lanes_are_opt_in() {
        let ops = build(
            &Config::load("[exec.a]\nrun = \"t\"\n[exec.b]\nrun = \"t\"\nparallel = 1\n").unwrap(),
        )
        .unwrap();
        assert!(
            ops.iter().any(|o| o.lane_declared),
            "one section declared a lane"
        );
        // Undeclared operations join the default lane rather than floating.
        assert!(ops.iter().all(|o| o.lane == Lane::default()));
    }

    #[test]
    fn sudo_comes_from_the_installer_and_the_section_may_override_it() {
        let config = Config::load(
            r#"
            [installer.system]
            check = "true"
            install = "true"
            sudo = true

            [installer.user]
            check = "true"
            install = "true"

            [packages.system]
            names = ["vim"]

            [packages.user]
            names = ["bacon"]
            sudo = true

            [repos.terra]
            url = "https://x"
            key = "https://k"
            "#,
        )
        .expect("config loads");
        let ops = build(&config).expect("plan");
        let sudo = |id: &str| ops.iter().find(|op| op.id == id).unwrap().sudo;

        assert!(sudo("packages.system"), "the installer escalates");
        assert!(
            sudo("packages.user"),
            "`sudo = true` on the section overrides a user-space installer"
        );
        assert!(sudo("repos.terra"), "a .repo file is written into /etc");
    }

    #[test]
    fn a_config_override_beats_the_kind_default() {
        let config = Config::load(
            r#"
            [installer.apt]
            check = "true"
            install = "true"
            sudo = true

            [packages.apt]
            names = ["vim"]
            sudo = false
            "#,
        )
        .unwrap();
        let ops = build(&config).unwrap();
        assert!(!ops[0].sudo, "`sudo = false` on the section wins");
    }

    #[test]
    fn resources_are_claimed_by_the_bodies_that_hold_them() {
        let ops = build(
            &Config::load(
                r#"
            [installer.dnf]
            check = "t"
            install = "t"
            resource = "rpm"

            [packages.dnf]
            names = ["git"]

            [exec.plain]
            run = "t"
            "#,
            )
            .unwrap(),
        )
        .unwrap();

        let resource = |id: &str| ops.iter().find(|op| op.id == id).unwrap().body.resource();
        assert_eq!(resource("packages.dnf"), Some("rpm"));
        assert_eq!(
            resource("exec.plain"),
            None,
            "a plain command holds nothing"
        );
    }

    #[test]
    fn an_unused_installer_is_visible() {
        let config = Config::load(
            r#"
            [installer.used]
            check = "t"
            install = "t"
            [installer.unused]
            check = "t"
            install = "t"
            [packages.used]
            names = ["a"]
            "#,
        )
        .unwrap();
        let used: Vec<String> = config.packages.keys().cloned().collect();
        assert_eq!(unused_installers(&config, &used), vec!["unused"]);
    }
}
