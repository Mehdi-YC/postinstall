---
name: new-feature
description: Start a new task in an isolated Git worktree branched from origin/main so multiple agents can work on the same repo in parallel without conflicts. Use at the beginning of every new feature, fix, or task — before writing any code.
---

# New Feature

Every task gets its own worktree and branch from `origin/main`. Never build on `main`, never reuse another agent's worktree or branch.

## Harness deltas — read first

- **Claude Code**: harness creates/manages worktrees under `.claude/worktrees/<name>`. Skip steps 3–4 (no manual `git worktree add`/`remove`), keep harness-assigned branch name.
- **Cursor** (branches `worktree-*`): same — keep assigned branch/worktree, apply steps 2 and 5.
- Any other harness: follow all steps.

## Steps

1. **Sync**: `git fetch origin`.

2. **Scope check**: `gh pr list` + `gh pr diff <n> --name-only`. If your task needs files another open PR edits, **stop and ask**. Check for uncommitted work too.

3. **Name**: `lowercase-with-hyphens-XXXXX` (short unique suffix). If `git worktree add` fails because the name exists, pick another — never force or reuse.

4. **Create** from repo root:

   ```bash
   git worktree add <worktrees-dir>/<task-name> \
     -b <branch-prefix>/<task-name> origin/main
   ```

   Use a **gitignored** directory (`.claude/worktrees/` or `.worktrees/`) and a consistent prefix (e.g. `agent/`).

5. **Enter and verify**:

   ```bash
   cd <worktrees-dir>/<task-name>
   git branch --show-current   # must print your new branch, not main
   ```

   Install dependencies fresh (worktrees don't share `node_modules`/virtualenvs) and confirm the required runtime version.

## Remember

- Worktrees don't isolate shared resources: dev-server ports, databases, lockfiles are global. Confirm a port answers *your* process (`lsof -i :<port>`) before trusting it. Resolve lockfile conflicts by regenerating, never hand-merging.
- Keep the worktree until the PR is merged or closed. Cleanup after merge:

  ```bash
  git worktree remove <worktrees-dir>/<task-name>
  git branch -D <branch-prefix>/<task-name>
  ```

  `-D` is expected: after squash/rebase merge, `-d` refuses even though the work is merged.
