#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TEST_ROOT=$(mktemp -d)
TEST_BIN="$TEST_ROOT/bin"
TEST_LOG="$TEST_ROOT/commands.log"
mkdir -p "$TEST_BIN"
trap 'rm -rf "$TEST_ROOT"' EXIT

cat >"$TEST_BIN/podman" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'podman %q\n' "$*" >>"$TEST_LOG"
case "${1:-}" in
  container)
    if [[ "${2:-}" == inspect ]]; then
      case "${PODMAN_SCENARIO:-absent}" in
        absent) exit 125 ;;
        compatible) printf 'localhost/patternfly-mcp:latest|true|127.0.0.1:3030\n' ;;
        conflict) printf 'another/image:latest|true|0.0.0.0:3030\n' ;;
      esac
    fi
    ;;
  run) exit 0 ;;
  stop|logs) exit 0 ;;
esac
EOF

cat >"$TEST_BIN/curl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'curl %q\n' "$*" >>"$TEST_LOG"
exit 0
EOF

cat >"$TEST_BIN/codex" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'codex %q\n' "$*" >>"$TEST_LOG"
exit "${CODEX_EXIT:-0}"
EOF
chmod +x "$TEST_BIN/podman" "$TEST_BIN/curl" "$TEST_BIN/codex"

assert_contains() {
  local expected=$1
  if ! rg -F --quiet -- "$expected" "$TEST_LOG"; then
    printf 'expected log to contain: %s\n' "$expected" >&2
    cat "$TEST_LOG" >&2
    exit 1
  fi
}

assert_not_contains() {
  local unexpected=$1
  if rg -F --quiet -- "$unexpected" "$TEST_LOG"; then
    printf 'expected log not to contain: %s\n' "$unexpected" >&2
    cat "$TEST_LOG" >&2
    exit 1
  fi
}

run_launcher() {
  : >"$TEST_LOG"
  PATH="$TEST_BIN:$PATH" TEST_LOG="$TEST_LOG" "$REPO_ROOT/scripts/codex-with-patternfly" "$@"
}

test_starts_owned_container_and_passes_http_override() {
  echo 'test_starts_owned_container_and_passes_http_override'
  PODMAN_SCENARIO=absent run_launcher prompt
  assert_contains 'podman run\ --rm\ -d\ --name\ ai-arch-story-patternfly-mcp'
  assert_contains '-p\ 127.0.0.1:3030:8080'
  assert_contains '--security-opt=no-new-privileges\ --cap-drop=ALL'
  assert_contains '--http\ --host\ 0.0.0.0\ --port\ 8080'
  assert_contains 'curl -fsS\ -o\ /dev/null\ http://127.0.0.1:3030/mcp'
  assert_contains 'codex -C\ '
  assert_contains 'mcp_servers.patternfly-mcp.url=\"http://127.0.0.1:3030/mcp\"'
  assert_contains 'podman stop\ ai-arch-story-patternfly-mcp'
}

test_reuses_conforming_container_without_stopping_it() {
  echo 'test_reuses_conforming_container_without_stopping_it'
  PODMAN_SCENARIO=compatible run_launcher prompt
  assert_not_contains 'podman run\ '
  assert_not_contains 'podman stop\ '
  assert_contains 'codex -C\ '
}

test_rejects_conflicting_running_container_before_codex() {
  echo 'test_rejects_conflicting_running_container_before_codex'
  : >"$TEST_LOG"
  if PODMAN_SCENARIO=conflict PATH="$TEST_BIN:$PATH" TEST_LOG="$TEST_LOG" "$REPO_ROOT/scripts/codex-with-patternfly" prompt; then
    echo 'conflicting container unexpectedly succeeded' >&2
    exit 1
  fi
  assert_not_contains 'codex '
  assert_not_contains 'podman run\ '
}

test_preserves_codex_arguments_and_exit_status() {
  echo 'test_preserves_codex_arguments_and_exit_status'
  : >"$TEST_LOG"
  if PODMAN_SCENARIO=compatible CODEX_EXIT=17 PATH="$TEST_BIN:$PATH" TEST_LOG="$TEST_LOG" "$REPO_ROOT/scripts/codex-with-patternfly" --model gpt-5.6-sol 'draw PatternFly cards'; then
    echo 'Codex exit status was unexpectedly replaced' >&2
    exit 1
  else
    status=$?
  fi
  [[ "$status" -eq 17 ]]
  assert_contains 'codex -C\ '
  assert_contains '--model\ gpt-5.6-sol\ draw\ PatternFly\ cards'
}

test_starts_owned_container_and_passes_http_override
test_reuses_conforming_container_without_stopping_it
test_rejects_conflicting_running_container_before_codex
test_preserves_codex_arguments_and_exit_status
echo 'PASS'
