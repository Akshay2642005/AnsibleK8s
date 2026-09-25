# ADR-002: New single-file config with legacy converter

- Status: Accepted
- Date: 2026-09-25
- Context: `reference/inventory/sample/group_vars/all.yaml` is executable Jinja: `hostvars[groups[...]]`, `ansible_facts[...]`, `ipaddr(...)` netaddr expressions. Full compatibility would require building a Jinja expression interpreter plus Ansible's fact model.

## Decision

Define a new declarative `cluster.yaml` (k0sctl-style single file) as the config format.
Provide a `convert` subcommand that reads a legacy `inventory/*` directory (hosts.ini + group_vars) and emits `cluster.yaml`.
Computed values (node IP, apiserver endpoint, MetalLB range) become built-in logic in the tool, not user expressions.

## Consequences

- No expression-language interpreter to build or maintain.
- Migration for existing users is one command, then hand-edit the output.
- The legacy group_vars values that are plain data survive conversion; Jinja expressions resolve to their evaluated equivalents where derivable, otherwise to documented placeholders.

## Alternatives considered

- Full Jinja group_vars compat (rejected: largest MVP cost); same-files-plain-YAML (rejected: still leaves users with a second config dialect).
