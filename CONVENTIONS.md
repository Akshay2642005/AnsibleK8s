# CONVENTIONS — ansiblek8s-rs

Read this before any GitHub or git operation.

## Commit Messages

Use Conventional Commits: `type(scope): subject`.
Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`, `build`, `ci`.
Use `!` or a `BREAKING CHANGE:` footer for breaking changes.
Subject: imperative mood, lowercase, no trailing period.
Put discovered defect fixes in separate commits from feature work.

## Preflight

Preflight MUST pass before every forward step:

```
cargo test && cargo clippy -- -D warnings && cargo build
```

## Always Green / Shift Left

Fix defects at the stage they appear: 1× locally, 10× in review, 100× in production.
Preflight green means tests pass, clippy reports zero warnings, build succeeds.
CI green means all required checks pass on the PR.
NEVER proceed on red Preflight or red CI.

## Discovered Defects

Apply the fix-or-log ladder:

1. Trivial fix (under ~20 lines, root cause obvious) → `quick-fix` now, separate commit.
2. Anything else → log to `specs/bugs/BUG-*.md`, run `fix-bug` later.
3. NEVER bundle an unrelated defect fix into feature work.

## Banned Dismissive Phrases

NEVER use these phrases to justify a red gate:

| Phrase | Why banned |
|--------|------------|
| "pre-existing" | A red gate blocks everyone regardless of origin |
| "unrelated to this session" | Defects found during work are in scope to log |
| "not introduced by my changes" | Prove it or log it; never assume |
| "out of scope" (ignoring a red gate) | Logging is the minimum obligation |

## Defensive Code

Apply **Retry** and **Timeout** categories:

- Retry: bound every retry with exponential backoff and a maximum attempt count.
- Timeout: set an explicit timeout on every SSH command, connection, and poll loop.
- NEVER retry without a bound. NEVER leave a remote operation unbounded.

## Specs Convention

All planning output goes to `specs/`:

- `specs/product/SCOPE_LATEST.yaml` — scope of the active initiative
- `specs/adr/` — architecture decision records
- `specs/epics/` — stories and tasks
- `specs/state.yaml` — session and workflow state (`workflow_mode`)

## Stack Conventions (Rust)

- Edition: stable Rust. `cargo clippy` with `-D warnings` is the lint gate.
- Name modules and files in `snake_case`; types in `UpperCamelCase`.
- Return `Result` from fallible operations; use `?` and typed errors.
- Put integration tests in `tests/`; unit tests live next to the code.
- Keep templates under `templates/`; keep golden files under `tests/golden/`.

## Verification Convention

Every epic task MUST carry a runnable `verify:` command. Evidence over claims.
Prove idempotency: re-run every mutating step and show zero changes.
Template output MUST pass the golden-file diff against the Python Jinja2 oracle.
