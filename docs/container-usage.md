# Container Usage Reference

This reference documents the project-supported commands for building and
running AI Arch Story with Podman. It covers the parameters used by this
project, not the complete Podman command-line interface.

The examples use the local image name `ai-arch-story`. No published image
registry or tag is assumed.

## Build the Local Image

From the repository root, build the image with:

```bash
podman build -t ai-arch-story .
```

| Parameter | Meaning |
| --- | --- |
| `build` | Builds an OCI container image from the project's `Containerfile`. |
| `-t ai-arch-story` | Assigns the local image name `ai-arch-story`, which the run examples use. |
| `.` | Uses the repository root as the build context, making the `Containerfile`, Rust source, templates, and web editor available to the build. |

The runtime image is based on Red Hat Universal Base Image 9 Minimal
(`registry.access.redhat.com/ubi9/ubi-minimal`). It contains the Rust
application, Graphviz, the renderer templates, and the pre-built web editor.
It sets `/workspace` as its working directory and uses `ai-arch-story` as its
entrypoint. Graphviz is installed through UBI's `microdnf` package manager.

The Node.js and Rust builder images are build-only stages and are not included
in the distribution image. The Rust builder uses a glibc version compatible
with UBI 9; see the [container modes specification](../.ai/specs/container-modes.md)
for the ABI compatibility contract.

## Choose an Operating Mode

| Goal | Run | What happens |
| --- | --- | --- |
| Convert an existing, finished diagram definition into one HTML file | `render` | Runs once, writes the HTML export, and exits. It does not start the web editor or API. |
| Create or edit diagrams through the browser, a coding agent, or both | `serve` | Keeps the web editor and HTTP API running against the mounted workspace. |

Use `render` for a single HTML conversion. Use `serve` for an interactive
session: the browser editor and a coding agent both use the same HTTP API and
the same mounted workspace. Start one `serve` container, open the editor in a
browser, and point the agent at that API; a second container is not needed for
the agent.

## Shared `podman run` Parameters

Every supported container run uses the local image name followed by an
application command:

```text
podman run [Podman parameters] ai-arch-story <application command> [command parameters]
```

| Parameter | Used for | Meaning |
| --- | --- | --- |
| `--rm` | Render and serve | Removes the stopped container automatically. The workspace contents remain because they are mounted from the host. |
| `-v HOST_PATH:/workspace:Z` | Render and serve | Mounts the host project or diagram workspace read/write at `/workspace` in the container. `HOST_PATH` must contain the diagram or project files. |
| `-p HOST_PORT:CONTAINER_PORT` | Serve only | Publishes a container port to the host. The standard editor/API mapping is `-p 8080:8080`. |

Use a read/write mount: rendering creates an output HTML file and the server
persists diagram changes through the API. The container's working directory is
`/workspace`, so relative application paths refer to the mounted workspace by
default.

### Understand the `-v` Value

The value passed to `-v` has this form:

```text
HOST_PATH:CONTAINER_PATH[:OPTIONS]
```

For the project's standard mount:

```text
-v /my/project:/workspace:Z
```

| Part | Example | Meaning |
| --- | --- | --- |
| Host path | `/my/project` | The directory on your computer containing the diagram workspace. You choose this path. |
| Container path | `/workspace` | The directory where that host folder appears inside the container. This is the image's default working directory. |
| Mount option | `Z` | An optional behavior modifier. It is not part of the `/workspace` path. |

Multiple options follow the last colon and are separated by commas, for
example `:ro,Z`.

### Choose Mount Options

| Option | Use it when | Effect and caution |
| --- | --- | --- |
| `rw` | Normal render or editor/API use | Read/write access. This is the default and is required because rendering writes HTML and the API can persist diagram changes. |
| `ro` | Input-only rendering with output redirected outside the mount | Read-only access. A standard render without a separate output path will fail because it needs to create an output file beside the input. |
| `Z` | One AI Arch Story container accesses the workspace on a SELinux host | Applies a private SELinux label. This is the project default for RHEL/Fedora. Do not use the same `Z`-labeled directory concurrently from another container. |
| `z` | Multiple containers need the same mounted workspace on a SELinux host | Applies a shared SELinux label. Use this only when the containers genuinely share the workspace. |

On macOS and other hosts where SELinux labeling does not apply, omit the label
option:

```bash
podman run --rm \
  -v /my/project:/workspace \
  ai-arch-story render diagrams/overview/diagram.json
```

On RHEL or Fedora, the normal read/write, private-label form is:

```bash
podman run --rm \
  -v /my/project:/workspace:Z \
  ai-arch-story render diagrams/overview/diagram.json
```

Podman also provides options such as `U` (change ownership of host files) and
`O` (temporary overlay writes). They are not part of this project's supported
commands: `U` can change host-file ownership, and `O` prevents writes from
persisting to the workspace.

## Render a Diagram: One-Shot Conversion

Use `render` for a one-shot HTML export:

```text
ai-arch-story render <input> [-o <output>]
```

| Parameter | Required | Meaning |
| --- | --- | --- |
| `<input>` | Yes | Path to a `diagram.json` file. Inside the container, use a path relative to `/workspace` or an absolute path beneath it. |
| `-o <output>` / `--output <output>` | No | Writes the HTML to the specified path in the mounted workspace. |

Without `-o`/`--output`, the command creates an `output/` directory beside the
input and writes `<name>.html`. When the input filename is `diagram.json`, the
output name is the input directory name; otherwise it is the input filename's
stem.

For the bundled CloudBrew example:

```bash
podman run --rm \
  -v "$(pwd)/test/cloudbrew:/workspace:Z" \
  ai-arch-story render diagrams/system-overview/diagram.json
```

This writes
`test/cloudbrew/diagrams/system-overview/output/system-overview.html` on the
host. To choose a different path, add `--output`, for example:

```bash
podman run --rm \
  -v "$(pwd)/test/cloudbrew:/workspace:Z" \
  ai-arch-story render diagrams/system-overview/diagram.json \
  --output output/cloudbrew-overview.html
```

## Start the Web Editor and API: Interactive Editing

Use `serve` to start the local HTTP API and the web editor it serves:

```text
ai-arch-story serve [--workspace <path>] [--port <port>] [--host <host>] [--static-dir <path>]
```

| Parameter | Default | Meaning |
| --- | --- | --- |
| `--workspace <path>` | `.` | Workspace containing the project or diagram files. The binary defaults to the current directory. Because the image sets `WORKDIR /workspace`, its effective default is `/workspace` when run in the container. |
| `--port <port>` | `8080` | Port on which the application listens inside the container. Pair it with a matching `-p` mapping to reach it from the host. |
| `--host <host>` | `0.0.0.0` | Address on which the application listens. The default accepts connections available through the Podman port mapping. |
| `--static-dir <path>` | Built-in editor assets | Development-only override for the location of the web-editor static files. Do not use it for normal container operation. |

Start the editor and API for the bundled workspace:

```bash
podman run --rm \
  -v "$(pwd)/test/cloudbrew:/workspace:Z" \
  -p 8080:8080 \
  ai-arch-story serve
```

Open [http://localhost:8080](http://localhost:8080). The browser connects to
host port `8080`, which Podman forwards to port `8080` in the container.

Keep this container running while using the web editor or asking Codex or
Claude Code to modify diagrams. Both clients call its API and see the same
disk-backed workspace state on their next request.

To use another host port while keeping the application default, map it to
container port `8080`:

```bash
podman run --rm \
  -v "$(pwd)/test/cloudbrew:/workspace:Z" \
  -p 9000:8080 \
  ai-arch-story serve
```

Open [http://localhost:9000](http://localhost:9000). If the application port
also changes, set both values consistently, for example `-p 9000:9000` with
`ai-arch-story serve --port 9000`.

The server has no authentication or authorization. Publish a port only for a
local machine or trusted network.

## Related Documentation

- [README quick start](../README.md#quick-start-podman)
- [Architecture deep dive](architecture.md)
- [Container modes specification](../.ai/specs/container-modes.md)
- [Web API specification](../.ai/specs/web-api.md)
