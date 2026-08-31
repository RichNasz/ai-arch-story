# Architecture Deep Dive

AI Arch Story converts a diagram definition into a portable, interactive HTML
artifact. It has two distinct runtime boundaries:

- **Authoring time:** coding agents and the local web editor modify a
  disk-backed workspace through the Rust HTTP API and rendering pipeline.
- **Viewing time:** the exported HTML runs independently in a browser. It
  contains the diagram data, SVG, styles, and interaction/animation code it
  needs; it does not call the API or require Graphviz.

## System at a Glance

```text
Coding agent ─┐
              ├──> Axum HTTP API ──> workspace + schema validation
Web editor ───┘             │                    │
                            │                    └──> diagram definition on disk
                            v
                    Rust render pipeline
                            │
                            ├──> Graphviz `dot` layout
                            ├──> renderer-ready diagram data
                            └──> HTML assembler
                                      │
                                      v
                         self-contained interactive HTML
```

The API and command-line render path share the same Rust workspace, validation,
layout, render-data, and export stages. This keeps an agent-created diagram, an
editor-created diagram, and a command-line render consistent.

## Components and Responsibilities

### Authoring clients

Coding agents translate natural-language requests into targeted diagram
operations. The React + PatternFly web editor provides forms for deterministic
edits such as nodes, edges, flows, groups, branding, and custom types. Neither
client owns a separate model of a diagram: each works against the local API and
the same workspace.

The expected agent behavior and the editor interaction model are specified in
the [agent workflow](../.ai/specs/agent-skill.md) and
[web editor](../.ai/specs/web-editor.md) specs.

### HTTP API and workspace

The Rust application exposes an Axum HTTP API for project data, diagrams,
diagram elements, validation, previews, rendered data, and final rendering.
Requests read from and write to the configured workspace, so changes made by
one authoring client are visible to the other on its next request.

Mutations are validation-on-write: an invalid definition is rejected before it
is persisted. The workspace layer also resolves shared data such as branding
and type definitions before rendering. The authoritative request and response
contracts are in [web-api.md](../.ai/specs/web-api.md), and the on-disk project
layout is defined in [workspace-structure.md](../.ai/specs/workspace-structure.md).

### Rust render pipeline

The Rust pipeline turns a validated diagram definition into data the browser
renderer can draw:

1. Load the diagram and resolved workspace context.
2. Validate the diagram and its references.
3. Generate a Graphviz graph and run `dot` to calculate node positions, edge
   routing, and group boundaries.
4. Convert the layout and resolved diagram information into render data.
5. Assemble a complete HTML document by inlining the renderer and styles with
   that data.

The CLI's `render` command and the API's render endpoint use this common path.
Schema rules, rendering behavior, and HTML assembly details remain governed by
[diagram-schema.md](../.ai/specs/diagram-schema.md),
[rendering-engine.md](../.ai/specs/rendering-engine.md), and
[export-format.md](../.ai/specs/export-format.md).

### Graphviz at build time

Graphviz is a build-time layout dependency, not part of the exported artifact.
The Rust pipeline generates a DOT graph, calls `dot -Tjson0`, and maps its
positions, routed splines, and cluster bounds into the renderer's coordinate
system. The browser receives precomputed layout data and focuses on adaptive
display, zooming, panning, interaction, and flow animation.

This split gives the output mature hierarchical layout while preserving a
zero-dependency viewing experience. See [tech-stack.md](../.ai/specs/tech-stack.md)
for the technology decision and [rendering-engine.md](../.ai/specs/rendering-engine.md)
for rendering details.

### Exported HTML renderer

The final HTML inlines the JavaScript SVG renderer and CSS along with render
data. It draws nodes, edges, groups, labels, and animated flows directly in the
browser. Because these assets are embedded, the document is suitable for
offline sharing as a single file.

The rendered output is also the visual source for editor preview behavior: the
editor requests server-produced render data and uses the same renderer assets,
avoiding a separate browser-only layout model. Flow behavior and visual rules
are specified in [flow-visualization.md](../.ai/specs/flow-visualization.md)
and [visual-design.md](../.ai/specs/visual-design.md).

## Main Flows

### Edit, validate, and persist

1. An agent or web-editor form sends a diagram operation to the API.
2. The API loads the current workspace state and applies the requested change.
3. Schema and reference validation run before the changed definition is
   written to disk.
4. The next request from either client reads that updated disk-backed state.

This validation-on-write model prevents the API and editor from accepting a
definition that the renderer cannot safely consume.

### Preview and rendering

1. The editor requests renderer-ready data or a preview for the selected
   diagram.
2. The server resolves types and branding, then runs the shared layout and
   render-data stages.
3. The preview uses the common renderer assets to display that data.
4. A final render invokes the same pipeline and writes the assembled HTML file
   to the diagram's output location.

The detailed preview and endpoint semantics are deliberately not duplicated
here; consult [web-editor.md](../.ai/specs/web-editor.md) and
[web-api.md](../.ai/specs/web-api.md).

## Container and Distribution Boundary

The Podman image is the supported distribution unit for authoring and
rendering. Its runtime image is based on Red Hat Universal Base Image 9 Minimal
and contains the Rust executable, Graphviz, the renderer templates, and the
pre-built web-editor static assets. In `render` mode it performs a one-shot
export; in `serve` mode it hosts the editor and API against a mounted workspace.

Node.js, npm, and the Rust compiler are build-stage dependencies, not runtime
requirements in the distribution image. The generated HTML is more portable
still: it is independent of the container and can be opened directly in a
browser. See [container-modes.md](../.ai/specs/container-modes.md) for the
commands, mount model, and server constraints.

The Rust builder uses a Bullseye-based image so its binary is compatible with
UBI 9's glibc runtime. This builder/runtime pairing is a build concern; neither
compiler nor build image is included in the distribution image.

For a parameter-by-parameter guide to the supported Podman commands and the
`render` and `serve` application options, see the
[container usage reference](container-usage.md).

## Extensibility and Source of Truth

Diagram definitions are the shared contract between authoring, validation,
layout, and rendering. Projects can define custom node types and imported SVG
shapes; type resolution occurs before the render pipeline constructs the final
render data. The complete inheritance and type-library rules are in
[custom-types.md](../.ai/specs/custom-types.md).

This document is an orientation guide. The source-of-truth specifications are
listed in the [specification roadmap](../.ai/specs/README.md); when a detailed
contract here conflicts with one of those specifications, the specification
wins.
