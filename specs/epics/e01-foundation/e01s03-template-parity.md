# e01s03 — Template port and golden-file parity harness

## 1. Story
As a maintainer, I prove every ported template renders byte-identically to the Jinja2 oracle, so template fidelity is evidence, not hope.

## 2. Type
feat

## 3. Risk
P0 (BCP 8) — retires grill risk F1, the largest unknown in the conversion.

## 4. Context
domain

## 5. Context
The legacy roles render 10 Jinja2 templates (systemd units, kube-vip/MetalLB/calico/cilium manifests, proxy config, log content). Tera deliberately deviates from Jinja2 — the maintainer states *"Tera requires keyword arguments for calls"* (github.com/Keats/tera/pull/448). This story builds a Python-Jinja2 oracle (development-only), ports the templates, implements the missing filters, and gates `cargo test` on byte-for-byte golden parity. Coverage of all 10 templates is a fidelity claim, not a feature claim: calico/cilium/proxy features remain out of scope.

## 6. Requirements
#### ADDED: Oracle renders all 10 templates
A committed script renders every legacy `.j2` with shared sample contexts into `tests/golden/*.golden`.

#### ADDED: Tera output matches goldens byte-for-byte
Any whitespace, quoting, or value drift fails `cargo test`.

#### ADDED: Missing Jinja filters exist as tested shims
`ipwrap`, `ipsubnet`, `ipaddr_prefix`, `bool`, `join_by_name` (Jinja `map('join', …)` equivalent) with unit tests against known inputs.

## 7. Actors
- CI / developer running `cargo test`
- Later phases that render systemd units and manifests

## 8. Preconditions
- e01s01 complete (project scaffold).
- Local `python3` + `jinja2` available as dev-only oracle (allowed by SCOPE `runtime_dependencies`).

## 9. Inputs
`reference/roles/**/templates/*.j2` (read-only), shared sample contexts in `tests/oracle/contexts/`.

## 10. Outputs
`tests/golden/*.golden` (10 files), `templates/` (ported Tera set), filter shim module.

## 11. Dependencies (SLOPCHECK)
- `tera` 2.4 [OK] — updated 2026-09-11, 38.7M downloads (crates.io, verified this session).
- Python `jinja2` [OK] — dev-only, never shipped.
- Custom filters: in-repo code, no extra crates.

## 12. Contracts / interface
- Filter trait signature per Tera docs: `pub fn upper(val: &str, _: Kwargs, _: &State) -> String`; keyword extraction via `Kwargs::must_get::<&str>("…")` which errors when missing (docs.rs/tera Kwargs).
- `templates::render(name, ctx) -> Result<String, RenderError>` — single entry point for all later rendering.
  **Reason for Depth:** Jinja filter semantics are a cross-template contract verified against the oracle; one shim module keeps parity failures localized and reviewable.

## 13. Steps
1. Write the oracle script rendering all 10 `.j2` files into goldens → verify: `python3 tests/oracle/render_golden.py && test "$(ls tests/golden | wc -l)" -eq 10`
2. Implement the five filter shims with unit tests (IPv6 wrapping, subnet prefix math, truthiness rules matching Jinja) → verify: `cargo test --lib filters`
3. Port the 10 templates to Tera (positional → keyword arguments; replace `map('join')` chains with `join_by_name`) → verify: `cargo test --test golden_templates`
4. Golden parity test compares Tera render vs committed goldens for every template × context → verify: `cargo test --test golden_templates -- --nocapture`
5. Document regeneration workflow in README → verify: `grep -q render_golden README.md`
6. Preflight → verify: `cargo test && cargo clippy -- -D warnings && cargo build`

## 14. Verification Script (Step-by-Step)
1. `python3 tests/oracle/render_golden.py` — regenerates 10 goldens (dev machine only).
2. `git diff --stat tests/golden` — only intentional template changes appear.
3. `cargo test --test golden_templates` — all 10 pass byte-for-byte.
4. Mutate one ported template (delete a space), rerun — test MUST fail.

## 15. Out of scope
- Applying templates to hosts (e03)
- Shipping Python or Jinja2 with the tool
- Feature work for calico/cilium/proxy

## 16. Risks
- Jinja whitespace behavior (trim_blocks/lstrip_blocks) differs from Tera defaults → configure Tera whitespace control explicitly; goldens catch drift.
- Contexts that under-represent template branches (e.g. kube-vip BGP block) → contexts must cover every conditional branch; assert branch coverage per template.

## 17. Acceptance criteria
- [ ] 10 goldens committed; `cargo test` green on parity
- [ ] Deliberate template mutation fails the golden test (proved once, recorded)
- [ ] All five filters unit-tested with Jinja-matching outputs
- [ ] Preflight green

## 18. Open questions
- None — filter behavior questions are answered by the oracle, not opinion.

## 19. References
- Grill F1/F2 (session record), github.com/Keats/tera README + PR #448, keats.github.io/tera (filter list), docs.rs/tera Kwargs, SCOPE S03, tasks `e01s03-tasks.yaml`

## 20. Change history
- 2026-09-25: created by plan-work (spine step 3).
