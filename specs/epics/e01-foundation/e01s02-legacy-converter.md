# e01s02 — Legacy inventory converter

## 1. Story
As an operator with an existing Ansible inventory, I run `convert` once to obtain a valid `cluster.yaml`.

## 2. Type
feat

## 3. Risk
P0 (BCP 5) — migration path; quoted verify runs against the real oracle inventory.

## 4. Context
domain

## 5. Context
Implements the migration half of ADR-002: read `reference/inventory/*` (hosts.ini groups + group_vars YAML), map known keys to the new schema, resolve derivable Jinja expressions (`apiserver_endpoint`, `k3s_node_ip`, MetalLB range) into computed defaults, and emit documented placeholders where a value cannot be derived without live facts. Conversion never modifies the oracle directory.

## 6. Requirements
#### ADDED: Convert hosts.ini groups to roles
`[master]` → `role: server`, `[node]` → `role: agent`, `[k3s_cluster:children]` members honored; plain-IP lines and `user@host:port` forms accepted.

#### ADDED: Map legacy group_vars keys
Known keys map to new fields; derivable expressions resolve to evaluated defaults; underivable expressions emit placeholder values with warnings listing each unresolved key.

#### ADDED: Converter output validates
`convert` output always passes `config::load` validation; conversion failure names the source line or key.

## 7. Actors
- Operator running `ansiblek8s-rs convert <inventory-dir> -o cluster.yaml`

## 8. Preconditions
- e01s01 story complete (loader + validator exist).

## 9. Inputs
Legacy inventory directory (`hosts.ini`, `group_vars/*.yaml`).

## 10. Outputs
`cluster.yaml` + conversion warnings on stderr.

## 11. Dependencies (SLOPCHECK)
- ~~`rust-ini` 0.21~~ **REJECTED 2026-09-26** (user-approved mid-C1): rust-ini's `key = value` grammar cannot parse Ansible inventory — bare host lines (`192.168.30.38`) slurp to EOF (`13:1 expecting [= or :] but found EOF`), `ParseOption` has no valueless-key mode, and space-separated `host k=v` vars are not INI at all. Replaced by the hand parser in `src/convert.rs` (~40 lines, zero new deps) covering exactly the §16 documented forms; errors name the offending line (§6).
- `serde` for the group_vars YAML values [OK].

## 12. Contracts / interface
- `convert::inventory(dir: &Path) -> Result<(ClusterConfig, Vec<Warning>), ConvertError>`
- Pure function; no filesystem writes beyond the `-o` target.

## 13. Steps
1. Parse hosts.ini groups via `convert::parse_hosts` (hand parser, no INI crate) into host entries with roles → verify: `cargo test --test inventory_parse`
2. Map legacy group_vars keys; resolve derivable expressions; collect underivable keys as warnings → verify: `cargo test --test group_vars_convert`
3. Add the `convert` subcommand writing `-o` target → verify: `cargo run -- convert reference/inventory/sample -o /tmp/converted.yaml && cargo run -- validate /tmp/converted.yaml`
4. End-to-end on the sample inventory (3 masters, 2 agents) → verify: `bash -c 'cargo run -- convert reference/inventory/sample -o /tmp/c.yaml && grep -q "role: server" /tmp/c.yaml && grep -q "role: agent" /tmp/c.yaml'`
5. Preflight → verify: `cargo test && cargo clippy -- -D warnings && cargo build`

## 14. Verification Script (Step-by-Step)
1. `cargo run -- convert reference/inventory/sample -o /tmp/converted.yaml`
2. Confirm warnings list any underivable keys (e.g. facts-dependent node IP).
3. `cargo run -- validate /tmp/converted.yaml` → exit 0.
4. Diff host sets: sample has 3 `server` + 2 `agent` entries.

## 15. Out of scope
- Converting group_vars Jinja *evaluation* (rejected by ADR-002)
- Writing back into `reference/` (never)

## 16. Risks
- Legacy configs in the wild use keys we do not map → warnings must never be silent (test asserts warning output).
- hosts.ini edge syntax (vars in lines, aliases) → parser tests cover the documented forms only.

## 17. Acceptance criteria
- [ ] Sample inventory converts and validates (task e01s02t3/t4 green)
- [ ] Underivable keys produce warnings, not silent drops
- [ ] `reference/` unmodified (assert via `git -C reference status` unchanged or file mtimes)
- [ ] Preflight green

## 18. Open questions
- None.

## 19. References
- ADR-002, SCOPE S02, `reference/inventory/sample/`, tasks `e01s02-tasks.yaml`

## 20. Change history
- 2026-09-25: created by plan-work (spine step 3).
- 2026-09-26: §11 amended during cycle C1 — rust-ini SLOPCHECK [OK] was maintenance-only; fitness against the oracle failed (EOF error). User approved dropping it for the hand parser; no other dep change.
