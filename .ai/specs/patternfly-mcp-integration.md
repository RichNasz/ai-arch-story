# Project-Scoped PatternFly MCP Integration

## What

AI Arch Story provides `scripts/codex-with-patternfly`, the supported entry
point for opening Codex with PatternFly MCP available for this repository. The
script runs PatternFly MCP over streamable HTTP and configures that HTTP URL for
the Codex process it launches only.

## Why

PatternFly guidance is useful only while editing this project's PatternFly web
editor. A global Codex MCP entry exposes the tool in unrelated projects and an
HTTP MCP URL alone cannot create its Podman container. The project launcher
owns that short-lived lifecycle boundary.

## Invocation

```sh
scripts/codex-with-patternfly [codex arguments...]
```

The command runs from any directory, resolves the repository root relative to
itself, and launches Codex with that root as its working directory. It forwards
all supplied arguments unchanged.

## Container contract

The launcher manages a container named `ai-arch-story-patternfly-mcp` with:

- image: `localhost/patternfly-mcp:latest`
- transport: HTTP on container port `8080`
- host mapping: `127.0.0.1:3030:8080` only
- server arguments: `--http --host 0.0.0.0 --port 8080`, with allowed host and
  origin values for `127.0.0.1:3030`
- security settings: `--security-opt=no-new-privileges --cap-drop=ALL`

The launcher must inspect a same-named existing container before acting. If it
is running with the required image and localhost port mapping, it reuses it. If
it is absent or stopped, it starts a new conforming container. It must fail
without replacing a running container whose image or published port differs;
the error must name the container and tell the user to resolve it manually.

## Readiness and Codex configuration

Before Codex starts, the launcher polls `http://127.0.0.1:3030/mcp` for up to
30 seconds. It treats any HTTP response from that path as proof that the HTTP
MCP listener is reachable; an MCP initialization response is not required
because a server may already have an active session. On timeout, it prints the
container logs and exits non-zero without starting Codex.

After readiness, it launches Codex with this process-only override:

```toml
[mcp_servers.patternfly-mcp]
url = "http://127.0.0.1:3030/mcp"
```

The override is passed using Codex's `-c` option; the launcher must not write
to `~/.codex/config.toml` or any other user-global configuration. A user must
remove any pre-existing global `patternfly-mcp` entry separately, because that
entry remains global by definition.

## Lifecycle

The launcher records whether it created the container. When the child Codex
process exits for any reason, it stops only a container it created, allowing
Podman's `--rm` behavior to remove it. It never stops a pre-existing running
container. Cleanup runs for normal exit and `INT`/`TERM` signals, and the
script returns Codex's exit status.

## Failure behavior

The command exits non-zero with an actionable message when Podman is absent or
unreachable, the image is unavailable, port `3030` is occupied, startup fails,
or readiness times out. It must never silently fall back to a stdio MCP server
or a global configuration entry.

## Verification

A repository smoke test verifies, against a temporary uniquely named test
container and localhost port, that the launch parameters yield a reachable
`/mcp` endpoint. It must clean up only the container it created. Unit-level
shell tests cover argument forwarding, rejection of conflicting containers,
and ownership-aware cleanup using mocked `podman`, `curl`, and `codex`
commands.

## Non-goals

- Configuring MCP globally for Codex.
- Starting PatternFly MCP when Codex is opened directly in a desktop UI.
- Exposing PatternFly MCP to the network beyond localhost.
- Changing the PatternFly MCP image or its tool content.
