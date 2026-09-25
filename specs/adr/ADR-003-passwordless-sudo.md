# ADR-003: Passwordless sudo required

- Status: Accepted
- Date: 2026-09-25
- Context: The legacy setup set `ask_become_pass = True`. Prior-art tools (k3sup, k0sctl) require passwordless sudo. Interactive sudo passwords over SSH need stdin/pty plumbing for every command.

## Decision

Require passwordless sudo (`NOPASSWD`) for the SSH user. All privileged commands run via `sudo -n` and fail loudly when sudo would prompt.
Document the prerequisite in the README: `user ALL=(ALL) NOPASSWD: ALL`.

## Consequences

- Simple, testable command execution; matches k3sup/k0sctl expectations.
- Behavior change vs the legacy Ansible flow; users with password sudo must configure NOPASSWD first.
- `sudo -n` failure produces a clear, actionable error instead of a hang.

## Alternatives considered

- Support sudo passwords (`--sudo-password`, `sudo -S`) — deferred; revisit post-MVP if requested.
