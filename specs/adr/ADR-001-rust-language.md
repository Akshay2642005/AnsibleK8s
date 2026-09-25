# ADR-001: Rust as the implementation language

- Status: Accepted
- Date: 2026-09-25
- Context: Phase 0 grill compared Go and Rust for converting the Ansible k3s playbooks (in `reference/`) into a native orchestrator CLI.

## Decision

Implement `ansiblek8s-rs` in **Rust** (stable), with this stack:

- SSH: `russh` (0.63.x, actively maintained)
- Templates: `tera` (2.x, Jinja2-inspired)
- CLI: `clap`; Config: `serde` + YAML; Async: `tokio`; CIDR math: `ipnet`

## Consequences

- Tera's ternary expressions and Jinja-matching `default` semantics sit closest to our `.j2` files.
- No Go-style domain prior art (k3sup/k0sctl are Go); we borrow k0sctl's *phase model* as the design reference, not its code.
- Template ports still need a shim: keyword-arg rewrite (Tera requires keyword arguments), plus custom filters `ipwrap`, `ipsubnet`, `ipaddr_prefix`, `bool`, `join_by_name`.
- Build time and iteration are slower than Go; type safety on privileged code is the trade.

## Alternatives considered

- Go: faster to build, proven in this exact tool category; rejected over template-semantics fit and the user's language choice.
