# AI Arch Story

A system for creating visually stunning, self-contained HTML technical architecture diagrams with animated data/work flow visualization. Users describe diagrams conversationally to an AI agent; the agent generates a JSON definition; a Rust toolchain renders it into a portable HTML file.

## Spec-Driven Development

**All design and implementation decisions are governed by specs in `.ai/specs/`.**

### Every Session

1. Read `.ai/specs/README.md` first — it is the roadmap of all specs, their status, and dependencies
2. Before writing or modifying code or documentation, read the relevant spec(s) for that area
3. If a decision isn't covered by an existing spec, write or update a spec before writing code or documentation
4. Code and documentation follow specs. If either disagrees with a spec, the spec wins — update the artifact or propose a spec change

### Spec Authority

- `.ai/specs/README.md` — Index, status, and dependency graph for all specs
- `.ai/specs/project-vision.md` — Vision, principles, interaction model
- `.ai/specs/tech-stack.md` — Technology choices and rationale
- `.ai/specs/diagram-schema.md` — JSON definition format (agent ↔ renderer contract)
- `.ai/specs/workspace-structure.md` — Folder hierarchy for projects and diagrams
- `.ai/specs/test-plan.md` — Validation plan using "CloudBrew" test project (5 thematic diagrams)
- `.ai/specs/rendering-engine.md` — SVG rendering, adaptive layout, interaction model
- `.ai/specs/flow-visualization.md` — Flow animation types, timing, controls
- `.ai/specs/visual-design.md` — Colors, typography, shapes, themes, spacing
- `.ai/specs/export-format.md` — HTML assembly pipeline, inlining, file structure
- `.ai/specs/agent-skill.md` — Agent generation rules, conversational patterns, type inference
- `.ai/specs/web-api.md` — HTTP API endpoints, request/response shapes, validation-on-write
- `.ai/specs/web-editor.md` — React + PatternFly form-based editor UI with live SVG preview
- `.ai/specs/container-modes.md` — Render vs. serve container modes, Containerfile structure
- `.ai/specs/custom-types.md` — User-definable node types, SVG shape imports, type libraries, inheritance

## Tech Stack

- **Rust** — All code outside the generated HTML (CLI, validation, dot generation, HTML assembly, HTTP API server via axum)
- **Graphviz** — Layout engine (`dot -Tjson0`) for hierarchical graph layout with cluster support (EPL-2.0)
- **JavaScript/CSS** — Inlined inside generated HTML files (rendering, animation, interaction)
- **React + PatternFly** — Web editor UI (form-based diagram editing, built at container image build time)
- **JSON** — Diagram definition format
- **Podman** — Container distribution (bundles Rust binary + Graphviz + web editor, no system install needed)

## Key Principles

- Specs are the source of truth — code and documentation are generated from specs
- Flows are first-class — not just arrows, but animated stories of data/work movement
- Zero-dependency HTML output — works offline, no CDN, no viewer software
- Adaptive output — diagrams render well across desktop, tablet, phone, projector
- Graphviz layout — Graphviz handles node positioning, edge routing, and cluster boundaries at build time; JS handles zoom/pan/interaction at view time
- Conversational interaction — users never write JSON directly; an AI agent generates it
- Containerized distribution — Podman container bundles all dependencies; local dev can use system Graphviz
