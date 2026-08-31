# Container Modes

## What

The `ai-arch-story` container supports two operating modes via CLI subcommands: **render** (batch, one-shot) and **serve** (long-running HTTP server with web editor).

## Why

The container already bundles the Rust binary and Graphviz for rendering. Extending it with a `serve` mode adds the web editor and API with zero additional installs for the user. Both modes share the same binary, validation logic, and rendering pipeline.

## Subcommands

## Choosing a Mode

Use `render` when a diagram definition already exists and the only task is to
produce an HTML file. The command runs once, writes its output, and exits; it
does not start the web editor or API.

Use `serve` when creating or editing diagrams interactively. It is a
long-running process that serves both the browser editor and the HTTP API. A
coding agent uses that same API, so the agent and web editor work against the
same mounted workspace through one running container. Do not start a separate
container for the agent; point it at the API exposed by the `serve` container.

### `render` (existing behavior)

One-shot rendering of a single diagram to HTML.

```
ai-arch-story render <input> [-o <output>]
```

| Argument | Required | Description |
|----------|----------|-------------|
| `<input>` | Yes | Path to `diagram.json` |
| `-o, --output` | No | Output HTML path (defaults to `<input-parent>/output/<name>.html`) |

Container usage:
```bash
podman run --rm \
  -v ./my-project:/workspace:Z \
  ai-arch-story render diagrams/overview/diagram.json
```

### `serve` (new)

Starts an HTTP server that serves the web editor UI and API.

```
ai-arch-story serve [--workspace <path>] [--port <port>] [--host <host>]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--workspace` | `/workspace` | Project or diagram root directory |
| `--port` | `8080` | HTTP port |
| `--host` | `0.0.0.0` | Bind address |
| `--static-dir` | (compiled-in) | Override path to webapp static files (dev only) |

Container usage:
```bash
podman run --rm \
  -v ./my-project:/workspace:Z \
  -p 8080:8080 \
  ai-arch-story serve
```

The user opens `http://localhost:8080` in their browser to access the web editor.

## Containerfile Structure

Three-stage build:

```dockerfile
# Stage 1: Build the React + PatternFly web editor
FROM node:24.15-slim AS webapp-builder
WORKDIR /build
COPY webapp/package.json webapp/package-lock.json ./
RUN npm ci
COPY webapp/ ./
RUN npm run build

# Stage 2: Build the Rust binary with a glibc version compatible with UBI 9
FROM rust:1.93-bullseye AS rust-builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY templates/ templates/
RUN cargo build --release

# Stage 3: Runtime image
FROM registry.access.redhat.com/ubi9/ubi-minimal
RUN microdnf install -y graphviz && \
    microdnf clean all

COPY --from=rust-builder /build/target/release/ai-arch-story /usr/local/bin/ai-arch-story
COPY --from=webapp-builder /build/dist /usr/share/ai-arch-story/webapp
COPY templates/ /usr/share/ai-arch-story/templates/

WORKDIR /workspace
EXPOSE 8080
ENTRYPOINT ["ai-arch-story"]
```

The runtime image contains only: Red Hat Universal Base Image 9 Minimal,
Graphviz, the Rust binary, the pre-built webapp static files, and the renderer
templates. No Node.js, no npm, no Rust toolchain.

## Runtime Base and Builder Compatibility

The distribution runtime is fixed to the UBI 9 major-version baseline:
`registry.access.redhat.com/ubi9/ubi-minimal`. It installs Graphviz through
`microdnf` from enabled UBI repositories and cleans package metadata in the
same layer. Graphviz's dependencies are accepted as part of the runtime image.

The Rust builder is `rust:1.93-bullseye`, not a newer Debian release. Its glibc
2.31 ABI is compatible with UBI 9's glibc 2.34. A builder image that produces a
binary requiring a newer glibc cannot be copied into this runtime image. Any
change to this pairing requires a compatibility check or a new decision to use
a static binary or a different runtime baseline.

The Node.js web-editor build stage is intentionally outside this decision: it
is a build-only dependency and is absent from the distribution image. UBI for
every build stage is not required.

Use the UBI major-version reference rather than an untracked immutable digest
until a release and update policy is specified. Rebuilds consume current UBI
package content; image publication, digest pinning, and vulnerability-response
policy are out of scope.

## Container Change Verification

Any change to the container base image, builder image, package installation, or
runtime dependencies must verify all of the following:

1. `podman build -t ai-arch-story .` succeeds.
2. The resulting image reports a RHEL/UBI 9 runtime.
3. Graphviz `dot` is available in the runtime image.
4. The image renders a CloudBrew diagram successfully.

The render smoke test may write to a temporary path inside the container to
avoid changing the mounted example workspace.

## Volume Mounting

The workspace directory is mounted from the host into `/workspace` in the container. Both `render` and `serve` modes operate on this directory.

- **Read/write mount** (`-v ./project:/workspace:Z`) — required for `render` (writes output HTML) and `serve` (API writes `diagram.json`)
- A volume value uses `HOST_PATH:CONTAINER_PATH[:OPTIONS]`; `/workspace` is
  the container destination and `Z` is a trailing mount option, not part of
  the path.
- **Access mode** — `rw` is the default and is required by the normal render
  and serve workflows. `ro` is suitable only for an input-only render whose
  output is redirected outside the mounted workspace.
- **SELinux label** — `Z` gives the mount a private label for one container;
  `z` gives it a shared label when multiple containers must access it. Use
  `Z` for the documented single-container commands on RHEL/Fedora. Omit the
  label option on non-SELinux hosts when it is not applicable.
- Host-ownership (`U`) and temporary overlay (`O`) options are outside the
  supported workflow because they can change host ownership or make writes
  non-persistent.

## Trust Model

The server binds to `0.0.0.0` by default (accessible from the host via port mapping). There is no authentication or authorization — the server is intended for:

- Local development (single user on their machine)
- Trusted network use (team members on a shared network)

Authentication is out of scope for the initial implementation.

## Claude Code Integration

Claude Code accesses the API via `curl` commands through its Bash tool:

```bash
# Read a diagram
curl -s http://localhost:8080/api/v1/diagrams/overview

# Add a node
curl -s -X POST http://localhost:8080/api/v1/diagrams/overview/nodes \
  -H 'Content-Type: application/json' \
  -d '{"id":"cache","label":"Redis Cache","type":"datastore"}'

# Validate
curl -s -X POST http://localhost:8080/api/v1/diagrams/overview/validate

# Render
curl -s -X POST http://localhost:8080/api/v1/diagrams/overview/render
```

This requires the container to be running, which is the same prerequisite as rendering (Graphviz is in the container). No additional local installs are needed.

## What This Spec Does NOT Cover

- API endpoint details (see `web-api.md`)
- Editor UI design (see `web-editor.md`)
- Container registry / image distribution
- TLS termination or reverse proxy configuration
- Multi-user access control
