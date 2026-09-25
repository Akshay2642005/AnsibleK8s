# e01s01 — Config schema and loader

## 1. Story
As an operator, I load a single validated `cluster.yaml` so every later phase reads one trusted config source.

## 2. Type
feat

## 3. Risk
P0 (BCP 5) — foundation for every other story.

## 4. Context
domain

## 5. Context
This story creates the cargo project and the typed configuration model (ADR-002). It turns the declarative cluster description — hosts and roles, network ranges, k3s version, load-balancer settings, SSH options — into validated structs with typed errors. Nothing else in the codebase exists before it; nothing downstream may parse raw YAML directly.

## 6. Requirements
#### ADDED: Load cluster.yaml into typed structs
Given a `cluster.yaml`, the loader returns a `ClusterConfig` or a typed error naming the offending field. Unknown fields are rejected, not ignored.

#### ADDED: Validate before any remote operation
Invalid CIDR, duplicate host entries, missing required fields, and unknown roles fail validation before a single SSH connection is attempted.

#### ADDED: Round-trip fidelity
Serializing a loaded config back to YAML and reloading it yields an equal value.

## 7. Actors
- Operator (interactive CLI)
- Converter (`e01s02`) and apply pipeline (later epics) as programmatic callers

## 8. Preconditions
- Task `e01s01t1` (scaffold) completed.

## 9. Inputs
`cluster.yaml` file path; example file ships in the repo.

## 10. Outputs
`ClusterConfig` value or `ConfigError` (missing field / invalid CIDR / duplicate host / unknown role / parse failure).

## 11. Dependencies (SLOPCHECK)
- `serde` 1.0 [OK] — 1.44B downloads, updated 2026-07 (crates.io, verified this session).
- `serde` derive [OK] — same package.
- YAML backend: **deferred** (decision: "decide at implementation"); confined behind the config seam — see §12.
- `thiserror` 2.0 [OK] — updated 2026-09-23; typed error enums.

## 12. Contracts / interface
- `config::load(path: &Path) -> Result<ClusterConfig, ConfigError>`
- All raw-YAML access is confined to one internal seam.
  **Reason for Depth:** the seam isolates the `[SUS]` YAML backend so the SLOPCHECK resolution cannot ripple into loaders, tests, or callers.

## 13. Steps
1. Scaffold the cargo bin project with module layout (`config`, later: `ssh`, `facts`, `phases`, `templates`) → verify: `cargo build`
2. Define `ClusterConfig` structs with serde derives; use `#[serde(deny_unknown_fields)]` so unknown keys error (serde docs: *"Always error during deserialization when encountering unknown fields"*) → verify: `cargo test --lib config`
3. Implement validation returning typed `ConfigError`s for missing fields, invalid CIDR, duplicate hosts, unknown roles → verify: `cargo test --test config_validation`
4. Add `examples/cluster.yaml` and a lossless serialize/deserialize round-trip test → verify: `cargo test --test config_roundtrip`
5. Preflight the whole scaffold → verify: `cargo test && cargo clippy -- -D warnings && cargo build`

## 14. Verification Script (Step-by-Step)
1. Run `cargo test` — all config tests green.
2. Run `cargo clippy -- -D warnings` — zero warnings.
3. Intentionally break `examples/cluster.yaml` (bad CIDR) and run the loader test fixture — expect `ConfigError::InvalidCidr` naming the field.

## 15. Out of scope
- Parsing legacy inventory (e01s02)
- Template rendering (e01s03)
- Any SSH or remote behavior

## 16. Risks
- YAML backend choice slips (mitigated by the §12 seam; decision deferred by user).
- Validation rules drift from what later phases assume → keep error enum exhaustive.

## 17. Acceptance criteria
- [ ] `cargo test && cargo clippy -- -D warnings && cargo build` green
- [ ] Unknown field in YAML → typed error
- [ ] Round-trip test green
- [ ] No module outside `config` reads raw YAML (grep-enforced in review)

## 18. Open questions
- YAML backend crate — deferred to implementation (user decision, this session).

## 19. References
- ADR-002 (single-file config), SCOPE S01, tasks `specs/epics/e01-foundation/e01s01-tasks.yaml`

## 20. Change history
- 2026-09-25: created by plan-work (spine step 3).
