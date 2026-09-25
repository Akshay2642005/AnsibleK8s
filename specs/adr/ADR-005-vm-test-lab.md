# ADR-005: Build a VM test lab for differential verification

- Status: Accepted
- Date: 2026-09-25
- Context: Phase 4 VERIFY diffs cluster state produced by the legacy Ansible oracle against the new tool. That requires real machines. The legacy sample inventory (`192.168.30.38-42`, `remote_user = vagrant`) suggests VMs, but no lab exists in the new repo.

## Decision

Create a reproducible VM lab as a Phase 2 task: **3 servers + 2 agents**, reachable over SSH with NOPASSWD sudo, runnable via Vagrant or multipass (choose at implementation time based on what the host supports).
The same lab must accept both `ansible-playbook` (from `reference/`) and the new CLI for back-to-back differential runs.

## Consequences

- Phase 4 verification and idempotency re-runs are unblocked.
- Lab creation itself becomes an early epic task with its own verify command.
- Cost: local resources for 5 small VMs (concurrent runs optional; reset between runs).

## Alternatives considered

- Real machines (rejected: no targets identified); no lab (rejected: verification would be claim-only, violating "evidence over claims").
