#!/usr/bin/env bash
set -euo pipefail

container_name="ai-arch-story-patternfly-mcp-check-$$"
image=localhost/patternfly-mcp:latest
created_container=0

cleanup() {
  if [[ "$created_container" -eq 1 ]]; then
    podman stop "$container_name" >/dev/null 2>&1 || true
  fi
}

trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

podman run --rm -d --name "$container_name" \
  -p 127.0.0.1::8080 \
  --security-opt=no-new-privileges --cap-drop=ALL \
  "$image" \
  --http --host 0.0.0.0 --port 8080 \
  --allowed-origins http://127.0.0.1:3030 \
  --allowed-hosts 127.0.0.1:3030 \
  --log-stderr >/dev/null
created_container=1

port=$(podman inspect --format '{{(index (index .NetworkSettings.Ports "8080/tcp") 0).HostPort}}' "$container_name")
mcp_url="http://127.0.0.1:$port/mcp"

for _ in $(seq 1 30); do
  if curl -sS -o /dev/null \
    -H 'Host: 127.0.0.1:3030' \
    -H 'Accept: application/json, text/event-stream' \
    "$mcp_url"; then
    printf 'PatternFly MCP HTTP smoke check passed at %s\n' "$mcp_url"
    exit 0
  fi
  sleep 1
done

printf 'PatternFly MCP did not become reachable at %s. Container logs follow:\n' "$mcp_url" >&2
podman logs "$container_name" >&2 || true
exit 1
