# Agent Workflow

Software factory workflow. Follow these rules for every task.

## Architecture

* **Django:** follow established Django best practices and the existing project's conventions.
* **Odoo:** follow established Odoo best practices and the existing project's conventions.
* **Everything else:** organize around features/capabilities and use Hexagonal Architecture / Ports & Adapters when it provides real value.
* Prefer simple architecture. Do not introduce abstractions for their own sake.
* Follow existing project conventions before introducing new patterns.

## Workflow

### 1. Inspect & Plan

Before coding:

* Inspect the existing implementation, project structure, tests, CI, and development commands.
* Read applicable `AGENTS.md` files and project documentation.
* Search for existing implementations before creating new ones.
* For UI work, inspect existing screens, components, design-system primitives, and patterns before creating anything new.
* Identify risks, ambiguities, and dependencies.

Create a concise implementation plan.

**Ask questions when requirements, behavior, scope, architecture, or acceptance criteria are unclear. Do not guess about important decisions.**

Do not modify code before the plan is approved.

### 2. Human Approval

Wait for explicit human approval such as:

`go ahead`

Plan approval authorizes implementation of the approved plan.

If implementation reveals a material change in scope, architecture, or approach, stop and ask for approval again.

Plan approval does not mean PR approval.

### 3. Isolate

* New feature / independent task → create a fresh Git worktree and branch from `origin/main`.
* Fix/refinement of the current task → keep using the current worktree.
* Never build on `main`.
* One worktree + one branch per task per agent.
* Never reuse another agent's worktree, branch, or uncommitted work.

Before starting, check for overlapping work:

* Open PRs and their changed files.
* Existing uncommitted changes.
* Other agents working on related areas.

If there is overlap or ambiguity, stop and ask.

### 4. Build

* Implement only the approved scope.
* Keep changes focused.
* Reuse existing code and patterns where appropriate.
* Do not refactor unrelated code.
* Do not silently expand scope.
* If an unrelated bug is discovered, report it rather than fixing it.
* Never discard or overwrite user work.

For implementation structure, use the project's established conventions and the architecture rules above.

## UI Reuse

Before creating a new UI element:

* Search for existing reusable components.
* Check the design system/component library.
* Check similar screens and existing UI patterns.
* Reuse an existing component when possible.
* Prefer extending or composing an existing component over creating a duplicate.
* Follow existing spacing, typography, icons, states, interactions, and responsive behavior.
* If an existing component is close but insufficient, explain why before creating a new one.

Never create a duplicate UI component without a reason.

## Prove

Every task must provide evidence appropriate to the change.

Capture:

* **Before:** establish the relevant existing behavior/state, ideally while reproducing the issue.
* **After:** demonstrate the behavior/state after the change.
 for UI changes use gh cli to send the images you captured, use chromium_browser with playwright
Use all relevant available repository tooling:

* Tests
* Linting
* Formatting
* Type checking
* Static analysis
* Build/compile checks
* Framework checks
* Database/migration checks
* Browser/UI tests
* Other CI/project checks

Discover and use the project's existing commands rather than inventing commands.

Never claim a check passed unless it was actually run.

If a check fails:

* Investigate it.
* Fix it when within scope.
* Do not weaken, disable, or remove the check.
* Report unresolved failures explicitly.

### UI Evidence

For **every UI change**, provide screenshots showing the relevant before/after states.

When appropriate, also provide:

* Browser verification
* Responsive states
* UI test results
* Console output
* Relevant interaction states

A UI task is not complete without visual verification unless the environment genuinely makes it impossible. Explain the limitation if so.

## Git

* Never commit to `main`.
* Keep commits focused and meaningful.
* Inspect `git diff` before committing.
* Inspect the complete final diff before opening the PR.
* Never commit secrets, credentials, tokens, `.env` files, or private configuration.
* Never discard existing user work.
* Never use `--force`.
* Use `--force-with-lease` only when rebasing your own task branch requires it.
* Resolve lockfile conflicts by regenerating rather than hand-merging.
* Worktrees do not isolate shared resources such as ports, databases, or external services. Check for conflicts before relying on isolation.

If a conflict cannot be resolved confidently, stop and report it.

## Ship

Before opening a PR:

1. Review the complete diff.
2. Confirm the implementation matches the approved plan.
3. Run all relevant repository checks.
4. Assemble before/after evidence.
5. Commit with a clear message.
6. Rebase onto `origin/main`.
7. Rerun relevant checks after the rebase.
8. Push the task branch.
9. Open the PR.

PR body must include:

* What changed and why.
* How it was tested.
* Exact relevant checks and their results.
* Before/after evidence.
* Screenshots for UI changes.
* Risks, limitations, or follow-ups.

Do not merge unless explicitly instructed.

Keep the worktree until the PR is merged or closed.

## Scope

Keep changes limited to the assigned task.

If a required change is outside the approved scope:

**stop and ask.**

Do not silently expand the task.

## Human Review

The human is the final authority.

There are two separate approval gates:

1. **Plan approval** → implementation may begin.
2. **PR approval** → work is accepted.

Never assume approval from silence.
when a new feature is added, recheck the readme if existing and change it if needed
Present the PR URL when ready for review.

## Writing for Humans

When writing text intended for humans, keep it concise, clear, and natural.

This includes:

* Commit messages
* PR titles/bodies
* Documentation
* Code comments
* User-facing messages
* Replies

Use `/unslop` when available for text you wrote or changed.

## Multi-Agent Rules

* One task = one worktree + one branch per agent.
* Never reuse another agent's worktree or branch.
* Check open PRs and uncommitted changes for overlap before starting.
* Never commit to `main`.
* Never use `--force`.
* Use `--force-with-lease` only on your own task branch.
* Do not assume worktrees isolate shared resources.
* Stop when conflicts or ownership are unclear.

## Completion Report

When the task is ready for review, report:

* Status
* What changed
* Checks run and results
* Evidence
* Known limitations
* PR URL
* Anything requiring human attention

Be factual. Never claim success without evidence.

## Repo-Specific

Each repository may append:

* Commands and checks
* Hard invariants
* Security requirements
* Architecture constraints
* Environment information
* Local test infrastructure
* Fixtures/stubs
* Known limitations
* Anything that cannot be tested locally

Repository-specific instructions should be followed in addition to these global rules.

## Anti-Patterns

| Anti-Pattern | Problem |
|---|---|
| **God service** | One huge function hides all control flow |
| **Leaky service** | Service mutates database tables directly |
| **Inconsistent API** | Different argument styles and error semantics |
| **Over-abstraction** | Extracting logic used by one caller |
