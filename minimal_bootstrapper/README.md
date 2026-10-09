# minimal_bootstrapper

Make a machine match a file.

```sh
bootstrapper list          # what would run, in what order
bootstrapper check         # what is missing; changes nothing
bootstrapper apply --sudo  # make it so, escalating one command at a time
```

The configuration declares what should be true. The tool works out the
difference and closes it. Everything else — lanes, dependencies, privilege
escalation — exists to make that safe to run twice, on a laptop, while something
else is using the network.

One dependency: [`toml`](https://crates.io/crates/toml) (and `serde`, which it
pulls in). Everything else is `std`.

## Why it is small

Every section of the config answers the same two questions: *how do I check
whether this is done, and how do I make it done?* So there is one `Op` type and
one scheduler, and `[packages.dnf]`, `[exec.bun]`, `[repos.terra]` and
`[dotfiles]` are four faces of it rather than four subsystems.

```
src/config.rs     the file: typed, with `${name}` expansion
src/plan.rs       the file becomes a Vec<Op>
src/run.rs        the scheduler, and what happens when an Op runs
src/os.rs         running commands, escalating them, logging them
src/dotfiles.rs   clone, back up what is in the way, stow
src/main.rs       the command line and the output
```

## Lanes

`parallel` puts an operation in a lane. Everything in the lowest ready lane runs
together, then the next lane. Segments compare as numbers, so `1.2` comes before
`1.10`.

```toml
[packages.dnf]
parallel = 2      # an integer
names = ["git", "fish"]

[exec.bun]
parallel = "3.1"  # or a string, when the lane has segments
check = "test -x $HOME/.bun/bin/bun"
run = "curl -fsSL https://bun.sh/install | bash"
```

Concurrency is **opt in**. With no `parallel =` anywhere, operations run one at
a time, which is what a laptop on battery wants.

An `installer` may claim a `resource`. Two operations holding the same resource
never overlap, whatever their lanes say — `dnf` and `rpm --import` both take the
rpmdb lock.

## after

Dependencies are not inferred. Nothing in the graph knows that `bun` has to
exist before `bun add -g` can work, so say it:

```toml
[packages.bun]
after = ["exec.bun"]
```

A dependency that fails skips what depended on it, and says which dependency was
to blame.

## Escalation

`sudo = true` on any section escalates it. Run the tool **as yourself**:

```sh
bootstrapper apply --sudo      # asks once, up front, then sudo -n per command
```

Never `sudo ./bootstrapper`. As root, `$HOME` is `/root`, so `cargo install` and
`uv tool install` put your tools somewhere your `PATH` never looks. `--sudo`
exists precisely to make the root case unnecessary: it refreshes the credential
once, where a human can see the prompt, and every escalated command afterwards
runs `sudo -n`, which fails immediately instead of hanging on an invisible
password prompt on a worker thread.

The tool refuses `--sudo` if it detects it is already root.

## Secrets

A variable named in `secret` has its value withheld from the run log.

```toml
secret = ["token"]

[variables]
token = "hunter2"
```

The spelling is the opposite of what it looks like: listing a name *hides* it.

Note the position: TOML puts every key after a `[table]` header inside that
table, so top-level keys like `secret` have to come before the first header.

## Dotfiles

```toml
[dotfiles]
repo = "https://github.com/me/dots"
branch = "main"
stow = ["config", "vim"]
```

Cloned to `~/.local/share/dotfiles` if it is not there, then `git reset --hard`
so the repository version wins.

Stow will not replace a real file with a symlink, and a real `~/.tmux.conf` is
the normal state of a machine set up some other way. So **an existing file is
moved to `<name>.bk` first**. An earlier backup is never overwritten: the second
run produces `<name>.bk.1`, and you still have both versions.

Directories are left alone — stow links *into* a directory rather than replacing
one, so your own files inside it are none of stow's business.

## check / plan / apply

`check` and `plan` are the same thing under two names: run everything, change
nothing. Both report every difference — a missing package, an unguarded command
that would run, a repository that is not written. They exit non-zero when the
machine does not match the file, which makes them usable from a health check.

Output for an operation with no `check` says so, rather than pretending to be
idempotent.

## Logs

Every command's stdout and stderr goes to a per-run directory, never to your
terminal:

```
~/.local/state/minimal_bootstrapper/runs/2026-10-09-110937/
├── run.log
├── packages-dnf.log
└── dotfiles-git.log
```

## Commands

```
bootstrapper [check|plan|apply|list] [OPTIONS]

    -c, --config <PATH>    configuration file
                           (default: ./bootstrap.toml, then
                            ~/.config/bootstrapper/bootstrap.toml)
        --only <TAGS>      comma separated operation ids or suffixes
        --from <ID>        start at this operation and continue
        --sudo             run as yourself, escalate one command at a time
```

`--only dnf` matches `packages.dnf`; `--only packages.dnf` matches exactly.

Exit status is `0` when nothing failed and `1` otherwise.

## Tests

```sh
cargo test
cargo clippy --all-targets
```

Operations in the tests really run — against a scratch directory, with `$HOME`
in the config text swapped for its path rather than exported, because `$HOME` is
process-global and the tests share a process.