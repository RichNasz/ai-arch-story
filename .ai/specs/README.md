# Specs Roadmap

This directory contains the source-of-truth specifications for the AI Arch Story project. Specs answer **What**, **How**, and **Why** for each component of the system. All code is generated from these specs — never the reverse.

## How to Use This Roadmap

1. **Start here** — Read this file first in every session to understand what specs exist and their status
2. **Read relevant specs** before making any design or implementation decisions — they are the authority
3. **Update specs first** when a decision changes — code follows specs, not the other way around
4. **Check dependencies** — some specs build on others; the dependency column shows what to read first

## Spec Index

| Spec | Status | Purpose | Dependencies |
|------|--------|---------|-------------|
| [project-vision.md](project-vision.md) | **Done** | Overall vision, design principles, interaction model, open questions | None |
| [tech-stack.md](tech-stack.md) | **Done** | Technology choices (Rust toolchain, JS/CSS in HTML, hybrid layout) | project-vision |
| [project-documentation.md](project-documentation.md) | **Draft** | Root README requirements, truthful badges, and future-documentation map | project-vision, tech-stack, container-modes, workspace-bootstrap, agent-skill, open-source-distribution |
| [diagram-schema.md](diagram-schema.md) | **Done** | JSON definition format — the contract between agent and renderer | project-vision, tech-stack |
| [workspace-structure.md](workspace-structure.md) | **Done** | Folder hierarchy for projects (thematic collections) and diagrams | project-vision, diagram-schema |
| [test-plan.md](test-plan.md) | **Done** | Validation plan using "CloudBrew" thematic test project (5 diagrams) | workspace-structure, diagram-schema |
| [rendering-engine.md](rendering-engine.md) | **Done** | How the JS inside the HTML draws nodes, edges, groups as SVG | tech-stack, diagram-schema |
| [flow-visualization.md](flow-visualization.md) | **Done** | Visual language for animated data/work flows | rendering-engine, diagram-schema |
| [visual-design.md](visual-design.md) | **Done** | Aesthetic system — colors, typography, shapes, themes | rendering-engine |
| [export-format.md](export-format.md) | **Done** | Self-contained HTML assembly, asset inlining, structure | tech-stack, rendering-engine |
| [agent-skill.md](agent-skill.md) | **Done** | How the AI agent generates and iterates on diagram definitions | diagram-schema |
| [branding.md](branding.md) | **Done** | Logo, organization name, footer, favicon, brand colors | visual-design, workspace-structure, export-format |
| [web-api.md](web-api.md) | **Draft** | HTTP API endpoints, request/response shapes, validation-on-write | diagram-schema, workspace-structure, workspace-bootstrap, tech-stack |
| [web-editor.md](web-editor.md) | **Done** | React + PatternFly form-based editor UI with live SVG preview | web-api, rendering-engine, visual-design |
| [container-modes.md](container-modes.md) | **Draft** | Render, start, and serve container commands; Containerfile structure | tech-stack, web-api, workspace-bootstrap |
| [open-source-distribution.md](open-source-distribution.md) | **Draft** | Apache-2.0 licensing, GitHub repository baseline, and pre-release CI boundary | project-documentation, container-modes |
| [public-release-remediation.md](public-release-remediation.md) | **Draft** | Security, provenance, build-context, and API-contract remediation for the public repository | web-api, workspace-structure, export-format, container-modes, open-source-distribution |
| [workspace-bootstrap.md](workspace-bootstrap.md) | **Draft** | `start` initialization and `serve` workspace validation contract | workspace-structure, container-modes, agent-skill |
| [custom-types.md](custom-types.md) | **Done** | User-definable node types, SVG shape imports, type libraries, inheritance | diagram-schema, workspace-structure, web-api, web-editor, visual-design |
| [patternfly-mcp-integration.md](patternfly-mcp-integration.md) | **Draft** | Project-scoped Codex launcher for a local HTTP PatternFly MCP server | container-modes |

## Status Definitions

- **Done** — Spec is written and approved as current source of truth
- **Draft** — Spec exists but is under active revision
- **Planned** — Spec is needed but not yet written

## Dependency Graph

```
project-vision
  ├── tech-stack
  │     ├── project-documentation (also depends on agent-skill, container-modes)
  │     ├── diagram-schema
  │     │     ├── workspace-structure
  │     │     │     ├── test-plan
  │     │     │     └── web-api
  │     │     │           ├── web-editor
  │     │     │           └── container-modes
  │     │     ├── agent-skill
  │     │     ├── rendering-engine
  │     │     │     ├── flow-visualization
  │     │     │     ├── visual-design
  │     │     │     │     ├── branding
  │     │     │     │     └── web-editor
  │     │     │     └── export-format
  │     │     │           └── branding
  │     │     └── custom-types
  │     │           ├── workspace-structure
  │     │           ├── web-api
  │     │           ├── web-editor
  │     │           └── visual-design
```

## Rules for Spec Maintenance

- A spec's status moves to **Draft** if any of its dependencies change in a way that affects it
- New specs should be added to this index before they are written
- When a spec is completed, update its status here and verify downstream specs are still consistent
- Specs should cross-reference each other by filename (e.g., "see `tech-stack.md`") rather than duplicating content
