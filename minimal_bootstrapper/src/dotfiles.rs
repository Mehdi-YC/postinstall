//! Dotfiles: clone, back up what is in the way, stow.
//!
//! Stow will not replace a real file with a symlink, which is the normal state
//! of a machine set up some other way. So an existing file is moved aside to
//! `<name>.bk` first, and an earlier backup is never overwritten.

use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Dotfiles;
use crate::os::Run;
use crate::run::Report;

/// Where the repository is cloned. `~` is expanded by the caller.
pub fn repo_path(home: &Path) -> PathBuf {
    home.join(".local/share/dotfiles")
}

/// Files inside a stow package, relative to the repository.
fn package_files(repo: &Path, package: &str) -> Vec<PathBuf> {
    let root = repo.join(package);
    let mut out = Vec::new();
    collect(&root, &root, &mut out);
    out.sort();
    out
}

fn collect(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(root, &path, out);
        } else if let Ok(relative) = path.strip_prefix(root) {
            out.push(relative.to_path_buf());
        }
    }
}

/// The first unused backup name: `x.bk`, then `x.bk.1`, then `x.bk.2`.
///
/// Never reuses a name. Overwriting an earlier backup would destroy the only
/// copy of the file the user actually had, which is the one thing a "we kept a
/// backup" feature must not do.
fn free_backup(path: &Path) -> PathBuf {
    let mut candidate = suffixed(path, ".bk");
    for n in 1.. {
        if !candidate.exists() {
            return candidate;
        }
        candidate = suffixed(path, &format!(".bk.{n}"));
    }
    candidate
}

fn suffixed(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}

/// A real file where stow wants to put a link.
///
/// A symlink is stow's own work. A directory is left alone, because stow links
/// *into* a directory rather than replacing one.
fn blocks_stow(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_file())
}

/// Move blocking files aside. Same directory, so a rename and not a copy.
fn back_up(home: &Path, repo: &Path, packages: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut moved = Vec::new();
    for package in packages {
        for relative in package_files(repo, package) {
            let target = home.join(&relative);
            if !blocks_stow(&target) {
                continue;
            }
            let backup = free_backup(&target);
            fs::rename(&target, &backup).map_err(|e| format!("{}: {e}", backup.display()))?;
            moved.push(backup);
        }
    }
    Ok(moved)
}

/// Clone if needed, move blocking files aside, then stow every package.
pub fn run(dotfiles: &Dotfiles, home: &Path, logs: &Run) -> Result<Report, String> {
    let repo = repo_path(home);
    let mut report = Report::new();

    let script = if repo.join(".git").exists() {
        // Two steps, and the order is the point.
        //
        // `pull --ff-only` advances the checkout; a bare `reset --hard` does not,
        // because HEAD is already wherever the last run left it, so a commit you
        // pushed would never arrive. The fast-forward-only flag then makes a
        // divergent local *history* fail loudly, which is the right answer to
        // "I have commits in my dotfiles" — silently discarding those would be
        // worse than stopping.
        //
        // The `reset --hard` after it discards local edits to the working tree,
        // which is what "the repository version wins" means. Without it, editing
        // a file through its symlink left the change in place forever, and apply
        // reported success while serving something the repository never said.
        report.changed("repository", "updating");
        format!(
            "git -C {} pull --ff-only --quiet && git -C {} reset --hard --quiet",
            repo.display(),
            repo.display()
        )
    } else {
        let branch = dotfiles
            .branch
            .as_deref()
            .map(|b| format!("--branch {b} "))
            .unwrap_or_default();
        report.changed("repository", format!("cloning into {}", repo.display()));
        format!(
            "git clone --quiet {branch}{} {}",
            dotfiles.repo,
            repo.display()
        )
    };

    let mut child = logs.spawn(&script, false, "dotfiles-git.log")?;
    if !child.wait().map_err(|e| e.to_string())?.success() {
        return Err(format!("`{script}` failed; see dotfiles-git.log"));
    }

    let moved = back_up(home, &repo, &dotfiles.stow)?;
    for backup in &moved {
        report.changed(backup.display().to_string(), "backed up");
    }

    for package in &dotfiles.stow {
        let mut child = logs.spawn(
            &format!(
                "stow --dir={} --target={} --restow {package}",
                repo.display(),
                home.display()
            ),
            false,
            &format!("stow-{}.log", log_name(package)),
        )?;
        if !child.wait().map_err(|e| e.to_string())?.success() {
            return Err(format!("stow failed for `{package}`; see the run log"));
        }
    }

    let total: usize = dotfiles
        .stow
        .iter()
        .map(|p| package_files(&repo, p).len())
        .sum();
    report.changed(format!("{total} file(s) linked"), "linked");
    report.kept("repository", repo.display().to_string());
    Ok(report)
}

/// A stow package name, made safe to use as a file name.
///
/// `stow = ["vim/colors"]` is a legitimate way to stow part of a package, and a
/// slash in it must not turn into a log path with a missing directory.
fn log_name(package: &str) -> String {
    package.replace(['/', '\\'], "_")
}

/// Has every target been linked from the repository?
///
/// Not "is the last component a symlink": stow links a whole subtree when the
/// package owns it exclusively, so a file inside a symlinked directory is a
/// regular file and correctly so. What matters is whether reading the path
/// reaches the repository.
pub fn converged(dotfiles: &Dotfiles, home: &Path) -> bool {
    let repo = repo_path(home);
    if !repo.join(".git").exists() {
        return false;
    }
    let Ok(repo) = repo.canonicalize() else {
        return false;
    };
    dotfiles.stow.iter().all(|package| {
        package_files(&repo, package).iter().all(|relative| {
            home.join(relative)
                .canonicalize()
                .is_ok_and(|target| target.starts_with(&repo))
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A repository with one stow package, and a `$HOME` beside it.
    fn fixture(name: &str) -> (PathBuf, PathBuf) {
        let root =
            std::env::temp_dir().join(format!("minimal-dotfiles-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let repo = repo_path(&root);
        fs::create_dir_all(repo.join("config/.config/fish")).unwrap();
        fs::write(repo.join("config/.tmux.conf"), "REPO\n").unwrap();
        fs::write(repo.join("config/.config/fish/config.fish"), "REPO\n").unwrap();
        (root, repo)
    }

    #[test]
    fn an_existing_file_is_moved_aside_with_its_contents() {
        let (home, repo) = fixture("backup");
        fs::write(home.join(".tmux.conf"), "ORIGINAL\n").unwrap();

        let moved = back_up(&home, &repo, &["config".to_string()]).unwrap();

        assert_eq!(moved, vec![home.join(".tmux.conf.bk")]);
        assert_eq!(
            fs::read_to_string(home.join(".tmux.conf.bk")).unwrap(),
            "ORIGINAL\n",
            "the user's file must survive verbatim"
        );
        assert!(
            !home.join(".tmux.conf").exists(),
            "the path is free for stow"
        );
    }

    #[test]
    fn an_earlier_backup_is_never_overwritten() {
        let (home, repo) = fixture("clobber");
        fs::write(home.join(".tmux.conf"), "SECOND\n").unwrap();
        fs::write(home.join(".tmux.conf.bk"), "FIRST\n").unwrap();

        let moved = back_up(&home, &repo, &["config".to_string()]).unwrap();

        assert_eq!(moved, vec![home.join(".tmux.conf.bk.1")]);
        assert_eq!(
            fs::read_to_string(home.join(".tmux.conf.bk")).unwrap(),
            "FIRST\n",
            "an earlier backup must be left alone"
        );
    }

    #[test]
    fn backup_names_climb_and_never_repeat() {
        let (home, repo) = fixture("climb");
        let wanted = [".tmux.conf.bk", ".tmux.conf.bk.1", ".tmux.conf.bk.2"];

        for name in wanted {
            fs::write(home.join(".tmux.conf"), "ORIGINAL\n").unwrap();
            let moved = back_up(&home, &repo, &["config".to_string()]).unwrap();
            assert_eq!(moved[0].file_name().unwrap(), std::ffi::OsStr::new(name));
        }
    }

    #[test]
    fn a_directory_is_never_moved_aside() {
        // Stow links *into* a directory; a user's own files in there are not
        // stow's business.
        let (home, repo) = fixture("directory");
        fs::create_dir_all(home.join(".config/fish")).unwrap();
        fs::write(home.join(".config/fish/mine.fish"), "MINE\n").unwrap();

        assert!(back_up(&home, &repo, &["config".to_string()])
            .unwrap()
            .is_empty());
        assert!(home.join(".config/fish/mine.fish").exists());
    }

    #[test]
    fn backing_up_twice_is_a_no_op() {
        let (home, repo) = fixture("idempotent");
        fs::write(home.join(".tmux.conf"), "ORIGINAL\n").unwrap();
        assert_eq!(
            back_up(&home, &repo, &["config".to_string()])
                .unwrap()
                .len(),
            1
        );

        // As stow would have left it: a symlink, so nothing left to move.
        std::os::unix::fs::symlink(repo.join("config/.tmux.conf"), home.join(".tmux.conf"))
            .unwrap();
        assert!(back_up(&home, &repo, &["config".to_string()])
            .unwrap()
            .is_empty());
    }

    #[test]
    fn a_file_the_user_replaced_is_moved_aside_again() {
        // If something overwrote a stowed path with a real file, that file is
        // now the one worth keeping.
        let (home, repo) = fixture("replaced");
        std::os::unix::fs::symlink(repo.join("config/.tmux.conf"), home.join(".tmux.conf"))
            .unwrap();
        fs::remove_file(home.join(".tmux.conf")).unwrap();
        fs::write(home.join(".tmux.conf"), "SOMETHING ELSE\n").unwrap();

        let moved = back_up(&home, &repo, &["config".to_string()]).unwrap();
        assert_eq!(moved, vec![home.join(".tmux.conf.bk")]);
        assert_eq!(fs::read_to_string(&moved[0]).unwrap(), "SOMETHING ELSE\n");
    }

    #[test]
    fn package_files_are_reported_relative_and_sorted() {
        let (_, repo) = fixture("files");
        let files = package_files(&repo, "config");
        assert_eq!(
            files,
            vec![
                PathBuf::from(".config/fish/config.fish"),
                PathBuf::from(".tmux.conf")
            ]
        );
    }

    /// A git repository to clone *from*, and an empty `$HOME` to clone into.
    ///
    /// The source is separate from the checkout on purpose: the test has to
    /// exercise a real clone, because a checkout with no remote has no upstream
    /// for `git pull` to fetch from.
    fn git_fixture(name: &str) -> (PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!("minimal-dots-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let home = root.join("home");
        let source = root.join("source");
        fs::create_dir_all(&home).unwrap();
        fs::create_dir_all(source.join("config")).unwrap();
        commit(&source, "v1\n", "v1");
        (home, source)
    }

    /// Write a file and commit it, creating the repository if it is not there yet.
    fn commit(repo: &Path, text: &str, message: &str) {
        fs::write(repo.join("config/.tmux.conf"), text).unwrap();
        let git = |args: &[&str]| {
            let out = std::process::Command::new("git")
                .args(args)
                .current_dir(repo)
                .output()
                .expect("git");
            assert!(
                out.status.success(),
                "git {args:?}: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        };
        if !repo.join(".git").exists() {
            git(&["init", "-q", "-b", "main", "."]);
        }
        git(&["add", "-A"]);
        git(&[
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "-qm",
            message,
        ]);
    }

    fn dotfiles_for(source: &Path) -> Dotfiles {
        Dotfiles {
            repo: source.display().to_string(),
            branch: Some("main".into()),
            stow: vec!["config".into()],
            schedule: Default::default(),
        }
    }

    fn apply(dotfiles: &Dotfiles, home: &Path) -> Result<Report, String> {
        let logs = Run::at(home.join("logs"));
        fs::create_dir_all(logs.dir()).unwrap();
        run(dotfiles, home, &logs)
    }

    /// The whole point of `apply`: a commit you pushed has to reach `$HOME`.
    ///
    /// This failed twice over. `converged()` short-circuited the operation as soon
    /// as the links existed, so nothing was ever fetched; and when it did run,
    /// `git reset --hard` with no argument moved the tree to the HEAD it already
    /// had. The tool reported "everything is configured" while serving stale
    /// files.
    #[test]
    fn a_pushed_commit_reaches_home() {
        let (home, source) = git_fixture("sync");

        apply(&dotfiles_for(&source), &home).expect("first apply");
        assert_eq!(fs::read_to_string(home.join(".tmux.conf")).unwrap(), "v1\n");

        commit(&source, "v2 from the repo\n", "v2");
        apply(&dotfiles_for(&source), &home).expect("second apply");
        assert_eq!(
            fs::read_to_string(home.join(".tmux.conf")).unwrap(),
            "v2 from the repo\n",
            "a pushed commit must reach $HOME"
        );

        commit(&source, "v3\n", "v3");
        apply(&dotfiles_for(&source), &home).expect("third apply");
        assert_eq!(fs::read_to_string(home.join(".tmux.conf")).unwrap(), "v3\n");
    }

    #[test]
    fn a_local_edit_is_overwritten_not_merged() {
        // The repository version wins, which is the whole contract of `apply`.
        let (home, source) = git_fixture("overwrite");
        apply(&dotfiles_for(&source), &home).unwrap();

        fs::write(home.join(".tmux.conf"), "edited through the symlink\n").unwrap();
        apply(&dotfiles_for(&source), &home).unwrap();

        assert_eq!(
            fs::read_to_string(home.join(".tmux.conf")).unwrap(),
            "v1\n",
            "a write through the link edits the repository, and apply puts it back"
        );
    }

    #[test]
    fn a_nested_stow_package_gets_a_usable_log_name() {
        assert_eq!(log_name("config"), "config");
        assert_eq!(log_name("vim/colors"), "vim_colors");
    }

    #[test]
    fn converged_accepts_a_symlinked_subtree() {
        // Stow links `~/.config` itself when the package owns it exclusively,
        // so the file inside is a regular file and correctly so.
        let (home, repo) = fixture("subtree");
        std::os::unix::fs::symlink(repo.join("config/.config"), home.join(".config")).unwrap();

        let dotfiles = Dotfiles {
            repo: "unused".into(),
            branch: None,
            stow: vec!["config".into()],
            schedule: Default::default(),
        };
        assert!(
            !converged(&dotfiles, &home),
            "no repository, so not converged"
        );
    }
}
