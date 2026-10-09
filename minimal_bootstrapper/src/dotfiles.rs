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
pub fn run(dotfiles: &Dotfiles, home: &Path, run: &Run) -> Result<Report, String> {
    let repo = repo_path(home);
    let mut report = Report::new();

    let script = if repo.join(".git").exists() {
        // The repository version wins: local edits are overwritten on apply.
        report.changed("repository", "updating");
        format!(
            "git -C {} fetch --all --quiet && git -C {} reset --hard --quiet",
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
        format!("git clone --quiet {branch}{} {}", dotfiles.repo, repo.display())
    };

    let mut child = run.spawn(&script, false, "dotfiles-git.log")?;
    if !child.wait().map_err(|e| e.to_string())?.success() {
        return Err(format!("`{script}` failed; see dotfiles-git.log"));
    }

    let moved = back_up(home, &repo, &dotfiles.stow)?;
    for backup in &moved {
        report.changed(backup.display().to_string(), "backed up");
    }

    for package in &dotfiles.stow {
        let mut child = run.spawn(
            &format!("stow --dir={} --target={} --restow {package}", repo.display(), home.display()),
            false,
            &format!("stow-{package}.log"),
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
        let root = std::env::temp_dir().join(format!(
            "minimal-dotfiles-{}-{name}",
            std::process::id()
        ));
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
        assert!(!home.join(".tmux.conf").exists(), "the path is free for stow");
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

        assert!(back_up(&home, &repo, &["config".to_string()]).unwrap().is_empty());
        assert!(home.join(".config/fish/mine.fish").exists());
    }

    #[test]
    fn backing_up_twice_is_a_no_op() {
        let (home, repo) = fixture("idempotent");
        fs::write(home.join(".tmux.conf"), "ORIGINAL\n").unwrap();
        assert_eq!(back_up(&home, &repo, &["config".to_string()]).unwrap().len(), 1);

        // As stow would have left it: a symlink, so nothing left to move.
        std::os::unix::fs::symlink(repo.join("config/.tmux.conf"), home.join(".tmux.conf")).unwrap();
        assert!(back_up(&home, &repo, &["config".to_string()]).unwrap().is_empty());
    }

    #[test]
    fn a_file_the_user_replaced_is_moved_aside_again() {
        // If something overwrote a stowed path with a real file, that file is
        // now the one worth keeping.
        let (home, repo) = fixture("replaced");
        std::os::unix::fs::symlink(repo.join("config/.tmux.conf"), home.join(".tmux.conf")).unwrap();
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