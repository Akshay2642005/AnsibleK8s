# ADR-004: Trust-on-first-use SSH host keys

- Status: Accepted
- Date: 2026-09-25
- Context: The legacy config set `host_key_checking = False` (no verification at all). Strict known_hosts adds first-connection friction; k0sctl exposes `StrictHostkeyChecking: false` only as an opt-in.

## Decision

Default to trust-on-first-use (TOFU): accept and pin the host key in `known_hosts` on first connection; hard-fail with a clear fingerprint-mismatch error on any later change.

## Consequences

- Zero friction on first connect, real protection against MITM after pinning.
- Strict improvement over the legacy policy.
- A legitimately reinstalled host requires the user to remove the stale entry (documented).

## Alternatives considered

- Auto-accept (legacy parity, no protection); strict known_hosts (pre-populate via ssh-keyscan — most friction).
