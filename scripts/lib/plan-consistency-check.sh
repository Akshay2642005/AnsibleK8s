#!/usr/bin/env bash
# plan-consistency-check.sh — plan-work consistency gate
# Usage: EPIC=specs/epics/eNN-slug bash scripts/lib/plan-consistency-check.sh "$EPIC"
set -euo pipefail

EPIC_DIR="${1:?usage: plan-consistency-check.sh <epic-dir>}"
fail=0
err() { echo "CRITICAL: $*" >&2; fail=1; }

[ -d "$EPIC_DIR" ] || { echo "CRITICAL: no such epic dir: $EPIC_DIR" >&2; exit 1; }
[ -f "$EPIC_DIR/epic.yaml" ] || err "missing epic.yaml in $EPIC_DIR"

# Collect story IDs declared by epic.yaml (portable: no bash4isms)
DECLARED=$(grep -Eo 'id: e[0-9]+s[0-9]+' "$EPIC_DIR/epic.yaml" | awk '{print $2}')
[ -n "$DECLARED" ] || err "epic.yaml declares no stories"

# Every declared story has a tasks file
for sid in $DECLARED; do
  [ -f "$EPIC_DIR/${sid}-tasks.yaml" ] || err "declared story $sid has no ${sid}-tasks.yaml"
done

# Every tasks file corresponds to a declared story; planned stories (with .md spec) fully gated
for f in "$EPIC_DIR"/*-tasks.yaml; do
  [ -e "$f" ] || continue
  sid=$(basename "$f" -tasks.yaml)
  grep -q "id: $sid" "$EPIC_DIR/epic.yaml" || err "$f not declared in epic.yaml"

  # every task must carry a runnable verify command
  n_tasks=$(grep -cE '^\s+- id:' "$f" || true)
  n_verify=$(grep -cE '^\s+verify:' "$f" || true)
  [ "${n_tasks:-0}" -gt 0 ] || err "$f: declares zero tasks"
  [ "$n_tasks" = "$n_verify" ] || err "$f: $n_tasks tasks but only $n_verify verify commands"

  # passing tasks must carry recorded verify evidence (failing-ledger rule:
  # a plan starts failing; flipping to passing is legal only with evidence)
  n_pass=$(grep -cE '^\s+status: passing' "$f" || true)
  n_evidence=$(grep -cE '^\s+evidence:' "$f" || true)
  if [ "${n_pass:-0}" -gt "${n_evidence:-0}" ]; then
    err "$f: $n_pass passing task(s) but only $n_evidence recorded evidence line(s)"
  fi

  # A story with a narrative .md spec is PLANNED: risk + allure required
  spec_count=$({ ls "$EPIC_DIR"/${sid}-*.md 2>/dev/null || true; } | wc -l | tr -d ' ')
  if [ "$spec_count" -gt 0 ]; then
    grep -qE '^\s+risk:' "$f" || err "$f: planned story missing risk"
    grep -q 'severity:' "$f" || err "$f: planned story missing allure.severity"
    # story status: failing (planned/in-progress) or passing (ledger flipped
    # — only legal once no task is still failing)
    story_status=$(grep -E '^status:' "$f" | head -1 | awk '{print $2}')
    case "$story_status" in
      failing) : ;;
      passing)
        n_open=$(grep -cE '^\s+status: failing' "$f" || true)
        [ "${n_open:-0}" -eq 0 ] || err "$f: story marked passing with $n_open task(s) still failing"
        ;;
      *) err "$f: story status must be failing or passing (got: ${story_status:-none})" ;;
    esac
  fi
done

# Scope coverage: every in_scope S-id claimed by some epic's scope_ref
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCOPE="$REPO_ROOT/specs/product/SCOPE_LATEST.yaml"
if [ -f "$SCOPE" ]; then
  SCOPE_IDS=$(grep -Eo '^\s+- id: S[0-9]+' "$SCOPE" | awk '{print $3}')
  for sid in $SCOPE_IDS; do
    grep -rq "scope_ref.*$sid" "$REPO_ROOT/specs/epics/" || err "scope item $sid maps to no epic/story"
  done
fi

if [ "$fail" -ne 0 ]; then
  echo "plan-consistency-check: FAIL ($EPIC_DIR)" >&2
  exit 1
fi
echo "plan-consistency-check: PASS ($EPIC_DIR)"
