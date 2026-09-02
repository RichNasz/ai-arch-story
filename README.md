# AI Arch Story

AI-assisted creation of visually rich, self-contained HTML architecture diagrams
with animated data and work flows.

[![Rust toolchain](https://img.shields.io/badge/Rust-toolchain-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![TypeScript web editor](https://img.shields.io/badge/TypeScript-web%20editor-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![JavaScript renderer](https://img.shields.io/badge/JavaScript-renderer-F7DF1E?logo=javascript&logoColor=black)](https://developer.mozilla.org/docs/Web/JavaScript)
[![Codex agent workflow](https://img.shields.io/badge/Codex-agent%20workflow-412991)](https://developers.openai.com/codex)
[![Claude Code agent workflow](https://img.shields.io/badge/Claude%20Code-agent%20workflow-D97757)](https://code.claude.com/docs/en/getting-started)

## Overview

AI Arch Story turns an architecture description into an interactive diagram that
can be shared as a single HTML file. The generated output works offline and
includes its styles, SVG, flow animations, and interaction code.

- Describe and refine an architecture conversationally.
- Render portable SVG-based diagrams with animated flows.
- Use Graphviz for build-time layout and routing.
- Edit diagrams through a local web editor and HTTP API that share one
  validation and rendering pipeline.

## Quick Start: Podman

Prerequisite: [Podman](https://podman.io/). You do not need to clone this
repository to begin a diagram project. The public `main` image is a mutable
active-development build, not a release or compatibility promise.

Create and enter a directory for your project, then initialize it. `start`
shows the resolved workspace and the directory-name-derived project name; you
can confirm, edit either value, or quit before anything is written:

```bash
mkdir my-architecture && cd my-architecture
podman run --rm -it \
  -v "$(pwd):/workspace:Z" \
  ghcr.io/richnasz/ai-arch-story:main \
  start --workspace /workspace --name "$(basename "$PWD")"
```

`start` creates `AGENTS.md`, a workspace-local `.ai/specs/` guidance bundle,
`project.json`, `shared/`, and `diagrams/`, then exits. The guidance helps a
coding agent work on the same user-owned workspace mounted by the container; it
is not a copy of this repository. `start` never initializes Git. If the
directory is partially initialized, it lists exactly what it can repair and
asks for approval.

Choose `serve` for ongoing interactive creation or editing. It requires an
initialized project workspace, then keeps the web editor and HTTP API running;
both the browser editor and a coding agent use that same API and mounted
workspace:

```bash
podman run --rm \
  -v "$(pwd):/workspace:Z" \
  -p 8080:8080 \
  ghcr.io/richnasz/ai-arch-story:main serve
```

Open [http://localhost:8080](http://localhost:8080) in a browser.

Choose `render` when you already have a `diagram.json` and only need its HTML
export. The command converts one diagram, writes the file, and exits:

```bash
podman build -t ai-arch-story .
podman run --rm \
  -v "$(pwd)/test/cloudbrew:/workspace:Z" \
  ai-arch-story render diagrams/system-overview/diagram.json
```

The rendered file is written to
`test/cloudbrew/diagrams/system-overview/output/system-overview.html`.

With `serve` running, open the project in Codex or Claude Code and ask it to
create or refine a diagram. The agent sends validated changes to the same API;
you can use the browser editor at the same time. You do not need a second
container for the agent.

For every supported Podman and container-started application parameter, mount
behavior, defaults, and additional examples, see the
[container usage reference](docs/container-usage.md).

## Local Development

Prerequisites: stable Rust/Cargo, Graphviz, and Node.js/npm. Podman is required
only to build or run the container image.

```bash
# Render the bundled example with the local Rust toolchain.
cargo run -- render test/cloudbrew/diagrams/system-overview/diagram.json

# Test the Rust implementation.
cargo test

# Build and test the web editor.
cd webapp
npm ci
npm run build
npm test -- --run
```

For browser-based editor development, run the API server in one terminal and
Vite in another:

```bash
# Terminal 1
cargo run -- serve --workspace test/cloudbrew

# Terminal 2
cd webapp
npm run dev
```

Vite proxies `/api` requests to the server at port 8080.

## Agent-Assisted Workflows

[Codex](https://developers.openai.com/codex) and
[Claude Code](https://code.claude.com/docs/en/getting-started) can both be used
to describe an architecture conversationally, make validated diagram changes
through the local HTTP API, and render the final HTML.

Initialize a new project once with `ai-arch-story start --yes`, then start the
service with `ai-arch-story serve`. Open the repository in your preferred
coding agent and describe the diagram you want to create or refine.
The project instructions in [AGENTS.md](AGENTS.md) and
[CLAUDE.md](CLAUDE.md) direct the agent to the source-of-truth specs and the
shared API workflow. See the [agent workflow spec](.ai/specs/agent-skill.md)
and [container modes spec](.ai/specs/container-modes.md) for details.

### PatternFly MCP for Codex

For this repository's PatternFly web-editor work, start Codex through the
project launcher:

```bash
scripts/codex-with-patternfly
```

It starts (or safely reuses) a local PatternFly MCP HTTP container bound only
to `127.0.0.1`, waits for its `/mcp` endpoint, and configures that endpoint for
the Codex process it launches. It does not modify global Codex configuration,
so the tool is unavailable to other projects. The local
`localhost/patternfly-mcp:latest` image is required.

Verify the HTTP setup independently with:

```bash
bash scripts/check-patternfly-mcp-http.sh
```

## Architecture

```text
Coding agents ─┐
               ├──> Local HTTP API ──> Rust render pipeline ──> Portable HTML
Web editor ────┘                              │
                                               └──> Graphviz layout
```

Coding agents and the TypeScript/React web editor are separate authoring
clients, but both use the same local HTTP API and workspace. The Rust
application validates and persists diagram definitions, resolves workspace
data, asks Graphviz to calculate layout, produces renderer-ready data, and
assembles the final HTML.

The exported file is a different runtime boundary: it embeds the JavaScript
renderer, CSS, SVG, and diagram data, so viewers need only a browser and no
connection to the API, Rust toolchain, Graphviz, or editor. Read the
[architecture deep dive](docs/architecture.md) for the component boundaries,
main request flows, and container model.

## Repository Map

| Path | Purpose |
| --- | --- |
| [`src/`](src/) | Rust CLI, HTTP server, schema validation, layout, and export pipeline |
| [`webapp/`](webapp/) | TypeScript/React + PatternFly web editor |
| [`templates/`](templates/) | Inlined JavaScript renderer and CSS for generated HTML |
| [`test/`](test/) | CloudBrew example workspace and test assets |
| [`.ai/specs/`](.ai/specs/) | Source-of-truth project specifications |

## Documentation and Participation

The [specification roadmap](.ai/specs/README.md) is the entry point for the
project's design documentation. Key references include the
[diagram schema](.ai/specs/diagram-schema.md),
[rendering engine](.ai/specs/rendering-engine.md), and
[project documentation spec](.ai/specs/project-documentation.md). The
[architecture deep dive](docs/architecture.md) explains how the implemented
components work together. The [branding guide](docs/branding.md) explains how
to apply organization identity to a workspace or individual diagram. The
[container usage reference](docs/container-usage.md) documents
project-supported Podman and container-started command parameters.
[development workflow](docs/development-workflow.md) defines contributor and
agent verification gates.

Contribution, community, security, release, and license documents will be
linked here when they are published. Until then, this README and the specs are
the authoritative project documentation.

## License and participation

AI Arch Story is open source under the [Apache License 2.0](LICENSE).
Report bugs and feature requests through the [GitHub issue tracker](https://github.com/RichNasz/ai-arch-story/issues).
