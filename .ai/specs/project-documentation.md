# Project Documentation

## What

This spec defines the user-facing documentation for AI Arch Story. Its primary
deliverable is the repository-root `README.md`: a concise, accurate entry point
for people evaluating, using, or contributing to the project. It also defines
how future documentation is introduced and kept discoverable without turning
the README into a duplicate of every reference document.

## Why

AI Arch Story combines a Rust toolchain, a TypeScript web editor, a
self-contained JavaScript renderer, and agent-assisted workflows. New users
need a trustworthy route from the project overview to a first rendered diagram;
contributors need an equally clear path to the local development workflow.

The README must make those paths visible while preserving the focused specs in
`.ai/specs/` as the technical source of truth. It must also follow open-source
documentation conventions without implying that unavailable policies, releases,
or automation already exist.

## Documentation Principles

- **Accurate and runnable** — Commands, prerequisites, links, and compatibility
  claims must reflect the current project. Do not publish placeholders as if
  they were working instructions.
- **Progressive disclosure** — Give newcomers the shortest useful path in the
  README; link to focused documents for detailed behavior.
- **User and contributor friendly** — Explain both the Podman-based usage path
  and the local development path.
- **Accessible and durable** — Use descriptive link text and image alt text,
  semantic heading order, language-tagged code fences, and relative links for
  repository content.
- **Truthful project signals** — Add only badges and status statements backed by
  the repository or configured public services.

## Root README Requirements

The root `README.md` is required and follows this order.

### 1. Project Identity

- Project name: **AI Arch Story**.
- A one-sentence description: AI-assisted creation of visually rich,
  self-contained HTML architecture diagrams with animated data and work flows.
- The approved badge row described in [Badge Policy](#badge-policy).

### 2. Overview and Core Capabilities

Explain the user value before implementation details. Cover these capabilities
without promising unimplemented features:

- Conversational generation and refinement of diagram definitions.
- Interactive, animated SVG diagrams exported as portable HTML with no runtime
  dependencies.
- Graphviz layout at build time.
- A local web editor and HTTP API that share the rendering and validation
  pipeline.

### 3. Quick Start: Podman

Provide the shortest supported path for a user to render a diagram and the
separate command to run the editor. State Podman and an available project or
diagram workspace as prerequisites. Commands must follow `container-modes.md`:

```bash
podman run --rm -v ./my-project:/workspace:Z ai-arch-story render diagrams/overview/diagram.json

podman run --rm -v ./my-project:/workspace:Z -p 8080:8080 ai-arch-story serve
```

Before the commands, state the mode-selection rule: use `render` for a
one-shot conversion of an existing, finished `diagram.json`; use `serve` for
interactive creation or editing. Explain that `serve` keeps the HTTP API and
web editor running and that a coding agent and the browser editor can use that
same running service and mounted workspace concurrently. The browser editor is
available at `http://localhost:8080`.

Do not claim a published image name, registry location, or version tag until it
exists; use a clearly identified local image name or link to an
install/release guide once one is published. Link readers to
`docs/container-usage.md` immediately after the quick-start examples for the
complete, project-supported Podman parameter reference and application command
options.

### 4. Local Development

Describe the developer prerequisites: stable Rust/Cargo, Graphviz, Node.js/npm
for the web editor, and Podman for container builds. Include only commands
verified against the repository's actual package scripts and CLI. Link to the
relevant technical specs for detailed runtime behavior rather than repeating
their contracts.

### 5. Agent-Assisted Workflows

Present **Codex** and **Claude Code** as supported conversational interfaces.
Explain that either agent translates a user description into diagram operations
through the local HTTP API, validates changes, and renders the output. Link to
`agent-skill.md` and `container-modes.md` for the authoritative workflow and
API examples. Do not imply sponsorship, certification, or exclusive support by
either provider.

### 6. Architecture Overview and Repository Navigation

Include a concise architecture overview that explains the system's primary
runtime boundary and the two supported authoring paths. It must include a
compact boundary diagram or prose equivalent showing:

```
Codex or Claude Code / Web Editor -> HTTP API -> Rust + Graphviz -> self-contained HTML
```

The overview must make these boundaries explicit without duplicating focused
specifications:

- Coding agents and the TypeScript/React web editor are separate authoring
  clients that use the same local HTTP API.
- The Rust application validates definitions, resolves workspace data, calls
  Graphviz for layout, prepares render data, and assembles the HTML export.
- The exported HTML embeds the renderer, styles, SVG, and diagram data and has
  no runtime dependency on the API, Rust, Graphviz, or the editor.

Link from this overview to the architecture deep dive at
`docs/architecture.md`. Follow it with a short repository map for `src/`,
`webapp/`, `templates/`, `test/`, and `.ai/specs/`. Each description must
remain high-level and point to the focused spec or source directory for detail.

### 7. Architecture Deep Dive

`docs/architecture.md` is the required, reader-facing architecture reference.
It is a guided system overview for contributors and technical evaluators, not a
second source of truth for detailed contracts. It must:

- State the system purpose and distinguish authoring-time components from the
  standalone exported artifact.
- Describe the responsibilities and interfaces of the authoring clients, HTTP
  API, workspace/schema layer, Rust render pipeline, Graphviz layout engine,
  and inlined HTML renderer.
- Trace the main flows: agent or editor mutation, validation and persistence,
  live preview/render-data retrieval, and final HTML export.
- Explain the container/runtime boundary: the distribution container bundles
  the Rust binary, Graphviz, templates, and built editor assets; generated HTML
  is portable and offline-capable.
- State the key consistency properties: validation-on-write, disk-backed shared
  workspace state, and shared render data/templates for preview and export.
- Link to the authoritative specs for API routes, schema, rendering, flows,
  export format, custom types, web editor behavior, and container modes rather
  than restating their field-level or algorithm-level rules.

The deep dive may use a compact text diagram. Claims must match the current
implementation and approved specs, and it must use repository-relative links.

### 8. Branding Guide

`docs/branding.md` is the required user-facing guide for applying organization
identity to diagrams. It complements `branding.md`, which remains the
source-of-truth design specification. The guide must:

- Explain that branding is optional, workspace-first, and embedded into the
  exported HTML rather than loaded from an external service.
- Direct interactive users to the web editor's Branding tab or a coding-agent
  workflow; present `shared/branding.json` as the version-controlled workspace
  representation, not as the only authoring path.
- Document the project-level file location and relative asset resolution, then
  give a complete project-branding example.
- Explain diagram-level overrides and `branding.enabled: false`, including a
  focused example for each.
- Cover the organization name, logo, colors, footer, and favicon fields,
  including logo formats, accessibility alt text, the 16KB encoded-logo limit,
  and the recommended 32x32 PNG favicon.
- Explain the resolution order and that explicit theme colors override branding
  colors for diagram content.
- Describe the rendered header, corner logo, footer, and favicon at a
  high level and link to the branding, schema, workspace, web editor, and API
  specs for detailed contracts.

The README documentation navigation must link to this guide.

### 9. Container Usage Reference

`docs/container-usage.md` is the required operational reference for running
the project through Podman. It complements the concise Quick Start rather than
duplicating it. It must document every Podman parameter used in the project's
supported build and run commands and every application parameter accepted by a
container-started command.

The reference must cover:

- The local image build command and its `-t` image-tag parameter and build
  context.
- The shared `podman run` parameters: `--rm`, `-v` volume mounts, the
  `/workspace` container destination, the `:Z` SELinux label, and `-p` port
  publishing. Volume-mount documentation must explain the
  `HOST_PATH:CONTAINER_PATH[:OPTIONS]` structure; distinguish the container
  destination from trailing mount options; and give selection guidance for
  `ro`/`rw` and `z`/`Z`, including the non-SELinux-host case.
- The `render` command's required input plus `-o` / `--output`, including how
  relative paths are resolved against the mounted workspace and where the
  default output is written.
- The `serve` command's `--workspace`, `--port`, `--host`, and `--static-dir`
  parameters, their defaults, intended use, and host-to-container port mapping.
- The distinction between the application's `.` workspace default and the
  container's `WORKDIR /workspace`, which makes the effective default workspace
  `/workspace` inside the distribution image.
- The UBI 9 Minimal runtime baseline, its use of `microdnf` for Graphviz, and
  the separate ABI-compatible Rust builder. The reference must clarify that
  build-stage images do not ship in the distribution image.
- At least one complete render example and one complete editor/API-server
  example using the local image name.

It must state that `--static-dir` is a development-only override and that the
server has no authentication, so published ports are appropriate only for the
local machine or a trusted network. It documents project-supported parameters,
not the full Podman command-line interface. It may name unsupported mount
options only to direct readers away from host-file ownership or overlay changes
that are not required by the project's commands.

### 10. Documentation and Project Participation

- Link to `.ai/specs/README.md` as the documentation/specification roadmap.
- Link to `docs/branding.md` from the README's documentation navigation.
- Link to `docs/container-usage.md` from the Podman quick-start section and
  the README's documentation navigation.
- Link to any published contributor, community, security, release, and license
  documents as described in [Future Documentation Map](#future-documentation-map).
- State how users should request help or report issues only after the project
  has an authoritative issue tracker or support channel.

## Badge Policy

The README starts with exactly these five static badges, in this order:

| Badge | Meaning | Destination |
| --- | --- | --- |
| Rust | The CLI, API server, validation, layout integration, and export tooling are Rust. | Rust's official site or documentation |
| TypeScript | The React web editor is implemented in TypeScript. | TypeScript's official site or documentation |
| JavaScript | The exported HTML renderer and interaction code are JavaScript. | MDN JavaScript documentation |
| Codex | Codex can be used for the documented conversational workflow. | Official Codex documentation |
| Claude Code | Claude Code can be used for the documented conversational workflow. | Official Claude Code documentation |

Badges use a stable static badge service or committed image assets. Every badge
has meaningful alt text and a destination link. The implementation may add
brand marks only when their use conforms to the respective provider's brand
guidelines.

Do not add build, test, coverage, package, download, release, license, or
community badges until the underlying service or project artifact exists and
the badge's claim is verified. Badge labels must describe the project fact, not
claim endorsement by a language, tool, or provider.

## Future Documentation Map

The following documents are reserved for future project needs. Their absence
must not be masked by broken README links or unsupported claims.

| Document | Canonical location | Audience and purpose | README rule |
| --- | --- | --- | --- |
| Contribution guide | `CONTRIBUTING.md` | Contributors; setup, submission, review, and style expectations | Link when published |
| Code of conduct | `CODE_OF_CONDUCT.md` | Community; participation expectations and reporting route | Link when published |
| Security policy | `SECURITY.md` | Security researchers and maintainers; reporting process and supported versions | Link when published |
| License | `LICENSE` | All users and contributors; usage and redistribution terms | Link when selected and published |
| Change log | `CHANGELOG.md` | Users upgrading between releases | Link when release process begins |
| Installation or release guide | `docs/installation.md` or `docs/releases.md` | Users installing published artifacts or upgrading | Link when artifact distribution exists |
| Architecture deep dive | `docs/architecture.md` | Contributors and technical evaluators; component boundaries and end-to-end rendering flow | Link from the README architecture overview |
| Branding guide | `docs/branding.md` | Diagram authors; workspace branding, overrides, assets, and visual precedence | Link from the README documentation navigation |
| Container usage reference | `docs/container-usage.md` | Users and operators; project-supported Podman and container-started command parameters | Link from the Podman quick start and documentation navigation |
| User guides | `docs/` | Users who need topic-focused workflows beyond quick start | Link from the README documentation section when it aids discovery |
| Development workflow | `docs/development-workflow.md` | Contributors and agents; branch, verification, and publish gates | Link from the README documentation section |

When introducing any future document:

1. Choose its canonical path and identify its audience and source of truth.
2. Add it to this map or to its own documentation spec if it establishes a new
   documentation subsystem.
3. Add a README link only when it helps a newcomer navigate the project.
4. Update `.ai/specs/README.md` if a new documentation spec is created.

## Maintenance and Acceptance Criteria

Update the README when a change affects user-visible setup, supported workflows,
compatibility, or navigation. Keep detailed schemas, API behavior, rendering
rules, and agent instructions in their focused specs.

An implementation of this spec is acceptable only when:

- The README contains every required section in the specified order.
- The README's architecture overview accurately links to `docs/architecture.md`.
- `docs/architecture.md` describes the current component boundaries and flows
  while delegating detailed contracts to the focused specs.
- `docs/branding.md` documents project branding, diagram-level control, asset
  constraints, and theme precedence without duplicating the branding contract.
- `docs/container-usage.md` accounts for every Podman and container-started
  application parameter used by the supported commands, with examples that
  match the image's runtime behavior.
- The README and container usage reference clearly distinguish one-shot
  `render` conversion from the long-running `serve` mode used for the web
  editor and agent-driven interactive editing.
- The container usage reference explains the three parts of a `-v` value and
  gives actionable selection guidance for `ro`/`rw`, `z`/`Z`, and non-SELinux
  hosts; it warns readers before suggesting ownership or overlay options.
- Container documentation identifies the UBI runtime and does not imply that
  the Node.js or Rust build stages ship in the distribution image.
- The five approved badges have accurate labels, accessible alt text, and
  authoritative destinations.
- All commands have been executed or otherwise validated against the current
  CLI, container, and package configuration.
- Relative links and local anchors resolve, and external links target their
  intended authoritative documentation.
- No section claims a published release, CI status, security process, license,
  or community policy before the underlying artifact or service exists.

## What This Spec Does Not Cover

- The contents of individual contributor, security, licensing, or release
  documents once they are introduced.
- CLI, API, renderer, or workspace contracts; see their corresponding specs.
- Publishing container images, managing a package registry, or configuring CI.
