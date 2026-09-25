#!/usr/bin/env bash
# verify-tdd-red-commit.sh — mechanical RED-commit isolation check (develop-tdd hard gate e45s08)
#
# Verifies the last two commits form a valid TDD pair by EXECUTION (authoritative),
# not by diff-shape heuristics:
#   * HEAD~1 = test-only commit: the suite FAILS at that revision
#   * HEAD   = implementation commit: the suite PASSES at that revision
#
# Usage:
#   bash scripts/verify-tdd-red-commit.sh                # check the current branch pair
#   bash scripts/verify-tdd-red-commit.sh --self-test    # prove the detector on synthetic repos
#
# Env:
#   VERIFY_TEST_CMD    command executed inside each revision (default: cargo test)
set -euo pipefail

TEST_CMD="${VERIFY_TEST_CMD:-cargo test}"
SELF="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"

fail() { echo "FAIL: $*" >&2; exit 1; }
info() { echo "  $*"; }

# --- helpers -----------------------------------------------------------------

# run_suite <root> <rev> <logfile>; echoes exit code (never aborts the script)
#
# NOTE: never share CARGO_TARGET_DIR across worktrees — the temporary
# revision's build would overwrite the current worktree's lib artifacts
# while fingerprints still say "fresh" (stale-rlib link errors).
run_suite() {
  local root="$1" rev="$2" log="$3" rc=0
  if [ "$rev" = "WORKTREE" ]; then
    (cd "$root" && bash -c "$TEST_CMD") >"$log" 2>&1 || rc=$?
  else
    local wt
    wt="$(mktemp -d "${TMPDIR:-/tmp}/tdd-red.XXXXXX")"
    if ! git -C "$root" worktree add --detach "$wt" "$rev" >/dev/null 2>&1; then
      echo "999" >"$log"; echo "worktree-add failed for $rev" >>"$log"
      return 999
    fi
    (cd "$wt" && bash -c "$TEST_CMD") >"$log" 2>&1 || rc=$?
    git -C "$root" worktree remove --force "$wt" >/dev/null 2>&1 || true
    rm -rf "$wt" 2>/dev/null || true
  fi
  return "$rc"
}

# --- main check --------------------------------------------------------------

verify_pair() {
  local root branch
  root="$(git rev-parse --show-toplevel 2>/dev/null)" || fail "not inside a git repository"
  branch="$(git -C "$root" rev-parse --abbrev-ref HEAD)"
  case "$branch" in
    main|master) fail "on '$branch' — develop-tdd hard gate: run kickoff-branch first" ;;
  esac
  git -C "$root" diff --quiet 2>/dev/null || fail "working tree dirty — commit or stash first"

  # Locate the most recent RED commit (subject starts with "test(") and the
  # commit that answered it (its child = GREEN). This keeps the gate re-runnable
  # after later chore/refactor commits land on top of the pair.
  local red_rev="" red_subj="" green_rev="" green_subj=""
  local hash subj
  while read -r hash subj; do
    case "$subj" in
      test\(*) red_rev="$hash"; red_subj="$subj"; break ;;
    esac
  done < <(git -C "$root" log --format='%H %s')
  [ -n "$red_rev" ] || fail "no test(<scope>): commit found on this branch"

  green_rev="$(git -C "$root" rev-list --ancestry-path --reverse "${red_rev}..HEAD" | head -1)"
  if [ -n "$green_rev" ]; then
    green_subj="$(git -C "$root" log -1 --format='%s' "$green_rev")"
  fi

  local tmp red_rc=0 green_rc=0 base_rc=0 head_rc=0
  tmp="$(mktemp -d "${TMPDIR:-/tmp}/tdd-red-check.XXXXXX")"
  trap "rm -rf '$tmp'" EXIT

  echo "== verify-tdd-red-commit: branch '$branch' =="
  info "RED   ${red_rev:0:7} $red_subj"
  if [ -n "$green_rev" ]; then
    info "GREEN ${green_rev:0:7} $green_subj"
    info "HEAD  $(git -C "$root" log -1 --format='%h %s')"
  else
    info "GREEN (none yet — RED is at HEAD, awaiting implementation)"
  fi

  info "running [$TEST_CMD] at RED..."
  run_suite "$root" "$red_rev" "$tmp/red.log" || red_rc=$?

  if [ -z "$green_rev" ]; then
    # Awaiting GREEN: RED must fail here; the pre-RED baseline must pass.
    local base_rc=0
    run_suite "$root" "${red_rev}^" "$tmp/base.log" || base_rc=$?
    info "RED exit=$red_rc | baseline(${red_rev:0:7}^) exit=$base_rc"
    if [ "$red_rc" -eq 0 ]; then
      fail "RED gate violated: the test-only commit PASSES in isolation (exit=0)"
    fi
    if [ "$base_rc" -ne 0 ]; then
      fail "baseline before RED already fails (exit=$base_rc) — nothing to compare against"
    fi
    echo "PASS: test-only commit fails in isolation (RED confirmed) — commit the GREEN implementation next"
    return 0
  fi

  # Pair present: RED must fail, GREEN must pass, and HEAD must still pass.
  info "running [$TEST_CMD] at GREEN..."
  run_suite "$root" "$green_rev" "$tmp/green.log" || green_rc=$?
  if [ "$green_rev" != "$(git -C "$root" rev-parse HEAD)" ]; then
    info "running [$TEST_CMD] at HEAD..."
    run_suite "$root" WORKTREE "$tmp/head.log" || head_rc=$?
  else
    head_rc=$green_rc
  fi
  info "RED exit=$red_rc | GREEN exit=$green_rc | HEAD exit=$head_rc"

  if [ "$red_rc" -eq 0 ]; then
    fail "RED gate violated: the test-only commit PASSES in isolation (RED exit=0)"
  fi
  if [ "$green_rc" -ne 0 ]; then
    fail "GREEN commit fails its own suite (exit=$green_rc)"
  fi
  if [ "$head_rc" -ne 0 ]; then
    fail "HEAD is red (exit=$head_rc) — current tree does not pass"
  fi
  echo "PASS: RED confirmed at ${red_rev:0:7} (exit $red_rc); GREEN at ${green_rev:0:7}; HEAD green (pair valid)"
  return 0
}

# --- self-test: prove all four verdicts on synthetic repos -------------------

self_test() {
  local base root rc
  base="$(mktemp -d "${TMPDIR:-/tmp}/tdd-red-self.XXXXXX")"
  trap "rm -rf '$base'" EXIT
  echo "== self-test: verify-tdd-red-commit.sh =="

  mk_repo() { # $1 = dir
    mkdir -p "$1" && git -C "$1" init -q -b work
  }
  commit() { # $1 = dir, $2 = message
    git -C "$1" add -A
    git -C "$1" -c user.email=self@test -c user.name=self commit -qm "$2"
  }

  # Case 1: valid pair (RED then GREEN) -> expect exit 0
  mk_repo "$base/good"
  echo pending >"$base/good/test.marker"
  commit "$base/good" "test(config): add failing expectation"
  echo ok >"$base/good/impl.txt"
  commit "$base/good" "feat(config): make it pass"
  rc=0
  (cd "$base/good" && VERIFY_TEST_CMD='test -f impl.txt' bash "$SELF") >/dev/null 2>&1 || rc=$?
  [ "$rc" -eq 0 ] && echo "  case 1 valid pair: correctly PASSed" || { echo "  case 1 valid pair: WRONGLY failed (rc=$rc)"; return 1; }

  # Case 2: violation — impl predates the test commit -> expect exit 1
  mk_repo "$base/bad"
  echo ok >"$base/bad/impl.txt"
  commit "$base/bad" "feat(config): impl already exists"
  echo pending >"$base/bad/test.marker"
  commit "$base/bad" "test(config): test that was never red"
  rc=0
  (cd "$base/bad" && VERIFY_TEST_CMD='test -f impl.txt' bash "$SELF") >/dev/null 2>&1 || rc=$?
  [ "$rc" -eq 1 ] && echo "  case 2 never-red test: correctly FAILed" || { echo "  case 2 never-red test: WRONGLY passed (rc=$rc)"; return 1; }

  # Case 3: RED committed, awaiting GREEN (HEAD fails, baseline passes) -> expect exit 0
  mk_repo "$base/awaiting"
  echo ok >"$base/awaiting/impl.txt"
  commit "$base/awaiting" "chore: baseline"
  echo pending >"$base/awaiting/test.marker"
  commit "$base/awaiting" "test(config): red awaiting green"
  rc=0
  (cd "$base/awaiting" && VERIFY_TEST_CMD='test ! -f test.marker' bash "$SELF") >/dev/null 2>&1 || rc=$?
  [ "$rc" -eq 0 ] && echo "  case 3 awaiting-green: correctly PASSed" || { echo "  case 3 awaiting-green: WRONGLY failed (rc=$rc)"; return 1; }

  # Case 4: HEAD still failing (broken GREEN) -> expect exit 1
  mk_repo "$base/broken"
  echo pending >"$base/broken/test.marker"
  commit "$base/broken" "test(config): red"
  echo nope >"$base/broken/wrong.txt"
  commit "$base/broken" "feat(config): wrong impl"
  rc=0
  (cd "$base/broken" && VERIFY_TEST_CMD='test -f impl.txt' bash "$SELF") >/dev/null 2>&1 || rc=$?
  [ "$rc" -eq 1 ] && echo "  case 4 broken green: correctly FAILed" || { echo "  case 4 broken green: WRONGLY passed (rc=$rc)"; return 1; }

  echo "self-test: OK (4/4 verdicts correct)"
  return 0
}

# --- dispatch ----------------------------------------------------------------

case "${1:-}" in
  --self-test) self_test ;;
  "") verify_pair ;;
  *) echo "usage: $0 [--self-test]" >&2; exit 2 ;;
esac
