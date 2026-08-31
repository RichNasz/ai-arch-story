# Tech Stack

## What

Technology choices for the AI Arch Story system, split by runtime boundary.

## Decisions

### Rust — All toolchain code

Any code that runs outside the generated HTML file is written in Rust. This includes:

- **CLI / build tool** — Reads a diagram definition (JSON), renders it into a self-contained HTML file
- **Schema validation** — Validates diagram definitions against the schema before rendering
- **Template engine** — Assembles the final HTML by inlining JS, CSS, and SVG assets
- **HTTP API server** — axum + tokio serve the web editor UI and diagram CRUD API (see `web-api.md`)

**Why Rust:**
- Single binary distribution — no runtime dependencies for end users
- Performance for potentially complex layout calculations
- Strong type system aligns with strict schema validation
- Can compile to WASM if in-browser toolchain is ever needed

### JavaScript/CSS — Inside the generated HTML

The self-contained HTML file includes inlined JavaScript and CSS for:

- **SVG rendering** — Drawing nodes, edges, groups, and labels
- **Flow animation** — Animating data/work movement along paths
- **Interactivity** — Zoom, pan, hover tooltips, click-to-inspect (if specified)
- **Layout** — Client-side positioning of elements (unless pre-computed by Rust)

**Why JS/CSS inside the HTML:**
- Browser-native — no plugins or viewers required
- Self-contained requirement demands everything inlined
- CSS animations/transitions are performant for flow visualization
- SVG + JS is the standard for interactive diagrams in browsers

### Open: JS libraries inside the HTML

The JavaScript inlined in the generated HTML may use libraries (bundled inline). Candidates to evaluate in the `rendering-engine.md` spec:

| Option | Tradeoff |
|--------|----------|
| Vanilla JS + SVG | Smallest output, full control, more code to write |
| D3.js (subset) | Powerful SVG manipulation, well-known, adds ~80KB inlined |
| Dagre / ELK.js | Auto-layout algorithms, adds size but solves positioning |
| Custom minimal | Purpose-built for this use case, optimal size |

Decision deferred to `rendering-engine.md` spec.

### Decided: Graphviz for Layout

Graph layout is delegated to **Graphviz** (`dot`), an industry-standard open-source graph visualization tool (EPL-2.0 license).

- **Rust toolchain** generates a `.dot` graph with `cluster_*` subgraphs from the diagram definition
- **Graphviz `dot -Tjson0`** computes node positions, edge spline control points, and cluster bounding boxes, returning them as JSON
- **Rust toolchain** parses the Graphviz JSON output and maps coordinates into the rendering pipeline
- **JavaScript (view time)** handles adaptive scaling, zoom/pan, and interaction from the pre-computed layout

**Why Graphviz:**
- Decades of refinement on hierarchical layout (Sugiyama algorithm), crossing minimization, and edge routing
- Native support for clusters (groups) — exactly what architecture diagrams need
- Used as a build-time tool only — no Graphviz code or artifacts in the generated HTML
- EPL-2.0 license with no obligations when used as an external tool (not linked)

**Integration crate:** `graphviz-rust` (MIT license) for programmatic `.dot` generation from Rust.

### Decided: Hybrid Layout Strategy

- **Graphviz + Rust (build time)** — Graphviz computes positions, Rust maps them to the viewport and embeds as coordinate data in the HTML
- **JavaScript (view time)** — Handles adaptive reflow for different viewport sizes, zoom/pan, and interaction

### React + PatternFly — Web Editor UI

The web editor is a React application using the PatternFly design system, built at container image build time and served as static files.

- **React** — Component model for form-based editing UI
- **PatternFly** — Red Hat's standard design system for internal web applications
- **Build-time only** — Node.js and npm exist only in the container build stage; the runtime image contains only the static JS/CSS/HTML output
- **No impact on diagram output** — The zero-dependency philosophy applies to generated HTML diagrams, not the editor UI

The editor's live preview pane reuses `templates/renderer.js` and `templates/styles.css` directly, ensuring pixel-identical preview.

### axum + tokio — HTTP Server

The `serve` subcommand starts an axum HTTP server (see `web-api.md` and `container-modes.md`):

- **axum** — Rust HTTP framework built on tokio and tower
- **tokio** — Async runtime
- **tower-http** — Static file serving (ServeDir) and CORS middleware

### Decided: Podman for Distribution

Graphviz is a runtime dependency of the build toolchain. To avoid requiring users to install Graphviz on their system, the toolchain is distributed as a **Podman container image**.

- **Container image** bundles the Rust binary + Graphviz in a minimal image
- **Runtime base image** is Red Hat Universal Base Image 9 Minimal
  (`registry.access.redhat.com/ubi9/ubi-minimal`), a freely redistributable
  RHEL-derived image with `microdnf` for installing Graphviz. UBI 9 is the
  supported major-version baseline for the distribution runtime.
- **Rust builder image** is `rust:1.93-bullseye`; its glibc 2.31 runtime ABI is
  compatible with UBI 9's glibc 2.34, unlike a Trixie-based builder
- **Podman** (not Docker) is the container runtime — rootless, daemonless, compatible with OCI standards
- Users mount their diagram workspace into the container, the toolchain generates HTML output
- Local development can still use a system-installed Graphviz; the container is for distribution

#### UBI Runtime Decision and Constraints

UBI applies to the **distribution runtime stage**, not every build stage. The
Node.js image used to compile the web editor does not ship in the final image,
so choosing a Red Hat Node builder is out of scope for this decision. The Rust
builder is selected for ABI compatibility with the runtime rather than for its
own distribution lineage.

The runtime uses `microdnf` and the enabled UBI 9 BaseOS and AppStream
repositories. Graphviz and its transitive dependencies are accepted runtime
contents because Graphviz is required to compute layout. The image must clean
package-manager metadata after installation.

Native Rust binaries copied into the UBI 9 runtime must require glibc 2.34 or
older. A builder or toolchain upgrade that would require a newer glibc is not
compatible with this runtime contract. Such a change requires an explicit
architecture decision to either retain a compatible builder, produce a
statically linked binary, or change the runtime baseline.

The project tracks the UBI major version in the `FROM` reference and rebuilds
the image to consume updated base-image and RPM content. Digest pinning,
published-image tagging, and a vulnerability-response process are deferred
until the project defines a release process.

**Why Podman:**
- Rootless by default — no daemon, no root privileges needed
- CLI-compatible with Docker (users familiar with Docker can use the same commands)
- OCI-compliant — images work with any OCI runtime

## Architecture Boundary

```
┌─ Podman Container ──────────────────┐     ┌──────────────────────────────┐
│  ┌─────────────────────────────┐    │     │     GENERATED HTML FILE      │
│  │       RUST TOOLCHAIN        │    │     │                              │
│  │                             │    │     │  Inlined CSS (styles)        │
│  │  CLI entrypoint             │    │     │  Inlined JS (rendering,     │
│  │  JSON schema validation     │    │────>│    animation, interaction)   │
│  │  Dot generation             │    │     │  Inlined SVG (diagram)       │
│  │  Graphviz JSON parsing      │    │     │  Diagram data (JSON blob)    │
│  │  HTML template assembly     │    │     │                              │
│  └──────────┬──────────────────┘    │     └──────────────────────────────┘
│             │ shell out             │         Zero-dependency HTML file
│  ┌──────────▼──────────────────┐    │
│  │     GRAPHVIZ (dot)          │    │
│  │  Hierarchical layout        │    │
│  │  Edge routing               │    │
│  │  Cluster positioning        │    │
│  └─────────────────────────────┘    │
└─────────────────────────────────────┘
```

## Build & Development

- **Rust toolchain**: Cargo, stable Rust edition
- **Graphviz**: System install for development, bundled in container for distribution
- **Container**: `Containerfile` at project root, built with `podman build`
- **Project structure**: Cargo workspace if complexity warrants it, single crate to start
- **Testing**: Rust unit/integration tests for schema validation and HTML assembly; visual snapshot tests for rendered output (approach TBD)
- **Output verification**: Generated HTML files can be opened directly in a browser for manual review
- **Container verification**: Changes to the container base, builder, package
  installation, or runtime dependencies must build the full image, confirm
  Graphviz's `dot` command is available, confirm the runtime is UBI/RHEL 9,
  and render a CloudBrew diagram successfully

## What This Spec Does NOT Cover

- Internal definition format → see `diagram-schema.md`
- Rendering approach and JS library choice → see `rendering-engine.md`
- Visual design language → see `visual-design.md`
- Agent interaction model → see `agent-skill.md` (planned)
