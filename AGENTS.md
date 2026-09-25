# ansiblek8s-rs — OpenCode

Read CONVENTIONS.md before any GitHub or git operation.

<!-- BEGIN bigpowers:project -->
## Project

Rust CLI that deploys HA k3s clusters over SSH, replacing the Ansible playbooks in `reference/`.
Stack: Rust (stable), tokio, russh, tera, clap, serde, ipnet.

## Commands

| Action | Command |
|--------|---------|
| Run | `cargo run -- <args>` |
| Test | `cargo test` |
| Build | `cargo build` |
| Lint | `cargo clippy -- -D warnings` |
| Preflight | `cargo test && cargo clippy -- -D warnings && cargo build` |
| CI | `gh pr checks` (when a PR is open) |

## Architecture

Config loader parses `cluster.yaml` (or converts legacy inventory) into validated structs.
SSH executor runs commands with `sudo -n` and trust-on-first-use host keys.
Phased orchestrator applies prereq → download → server → agent → post → teardown phases.
Tera templates render systemd units and manifests; golden-file tests diff them against Jinja2 output.

## Conventions

- Write Conventional Commits: `type(scope): subject`.
- Write all planning output to `specs/` before any code.
- Bound every retry with backoff; bound every remote operation with a timeout.
- Verify template output byte-for-byte against the Python Jinja2 oracle.
- Re-run every mutating step to prove idempotency.

## Never

- Never touch `reference/`; it is the Ansible oracle and stays locally ignored.
- Never proceed on red Preflight or red CI; invoke quick-fix or fix-bug first.
- Never weaken, skip, or delete a golden-file test to make it pass.
- Never run a remote command without an explicit timeout.
- Never dismiss reproducible gate failures as pre-existing or out of scope.

## Agent Rules

- **Workflow Mandate:** You MUST use the bigpowers skills (e.g. `plan-work`, `develop-tdd`) to perform tasks. DO NOT write code directly in response to a user prompt like "build this feature".
- **Always Green:** Preflight must be green before forward work. Reproducible gate failures require fix-or-log per CONVENTIONS § Discovered Defects.
- Read `specs/` before writing code.
- All planning and specifications MUST be written to `specs/` before any code is generated.
- Write the minimum code that solves the stated problem. Nothing extra.
- Run tests after every change. Show evidence before declaring done.
- One clarifying question beats a wrong assumption baked into 200 lines.
<!-- END bigpowers:project -->

<!-- BEGIN bigpowers:context-routing -->
## Context Routing

| Glob | Route to |
|------|----------|
| `specs/**` | Project planning state |
| `reference/**` | Read-only Ansible oracle; NEVER modify |

<!-- END bigpowers:context-routing -->

<!-- BEGIN bigpowers:learned-preferences -->
## Learned User Preferences

- (none yet)

## Workspace Facts

- Old Ansible project lives in `reference/`, excluded via `.git/info/exclude`.
- Old git history preserved at `reference/.git-orig`.
<!-- END bigpowers:learned-preferences -->
