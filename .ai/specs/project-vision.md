# Project Vision: AI Arch Story

## What

A system that enables users to create visually stunning technical architecture diagrams that illustrate how work and data flow through elements of an architecture. Generated diagrams are self-contained HTML files that can be shared without any external dependencies.

## Why

Technical architecture diagrams are essential for communicating system design, but existing tools either produce static, lifeless diagrams or require proprietary platforms to view. There is a gap for a tool that:

- Produces **beautiful, publication-quality** visual output
- Shows **dynamic flow** — how work and data move through system components
- Generates **portable, self-contained HTML** — no viewer software, no SaaS login, just open in a browser
- Can be **version-controlled** alongside the systems they describe

## How (High Level)

The system will follow a spec-driven design approach:

1. **Specs as source of truth** — All design decisions, component definitions, and behavioral contracts live in `.ai/specs/` and drive all code generation
2. **Conversational input** — Users describe their architecture to an AI agent (e.g., Claude Code) through natural conversation. The agent interprets the request and generates the project's internal diagram definition format. Users never write the definition format directly.
3. **Internal definition format** — A structured format (JSON or similar) that serves as the intermediate representation between conversational intent and rendered output. This format must be unambiguous and complete enough for the rendering engine, and well-structured enough for an LLM to generate reliably.
4. **Rendering engine** — Transforms definitions into interactive, animated SVG/HTML visualizations (engine TBD per tech stack spec)
5. **Export** — Produces a single self-contained HTML file with all styles, scripts, and assets inlined

### Interaction Model

Two complementary input paths share a single API and rendering pipeline:

```
User (natural language) → Claude Code → HTTP API ──┐
                                                     ├→ diagram.json → Graphviz → HTML
User (direct editing)   → Web Editor  → HTTP API ──┘
```

**AI-assisted path** (Claude Code):
1. Describe the architecture conversationally ("I have a React frontend that talks to a Node API, which reads from Postgres and publishes events to Kafka...")
2. The agent generates diagram elements via the HTTP API (`curl` calls)
3. The rendering engine produces the HTML diagram
4. The user reviews and iterates conversationally ("Make the Kafka flow more prominent", "Add a cache layer between the API and database")
5. The agent updates the definition via the API, re-renders

**Direct editing path** (Web Editor):
1. Open the web editor in a browser (served from the same container)
2. Edit node properties, edges, flows, groups, and branding through PatternFly forms
3. See changes in a live SVG preview (pixel-identical to exported HTML)
4. Trigger rendering to produce the HTML output

Both paths go through the same HTTP API with validation-on-write, eliminating drift. The AI path is best for creative work (inferring architecture from descriptions, choosing visualization patterns). The direct path is best for deterministic edits (renaming, styling, connecting) that don't benefit from AI and would waste tokens.

## Key Design Principles

- **Visual quality first** — Diagrams should be beautiful enough to present to executives, detailed enough for engineers
- **Flow is a first-class concept** — Not just boxes and arrows; the movement of data and work through the system should be visually represented
- **Zero-dependency output** — A shared HTML file must work offline, with no CDN calls or external resources
- **Adaptive output** — Diagrams must render well across devices (desktop, laptop, tablet, phone, projector) without requiring separate exports per screen size
- **Spec-driven** — Specs answer What, How, and Why; code is generated from specs, not the other way around

## Spec Organization

Specs live in `.ai/specs/` and are organized by concern:

| Spec | Purpose |
|------|---------|
| `project-vision.md` | This file — overall vision, principles, and goals |
| `tech-stack.md` | Technology choices and rationale (pending) |
| `diagram-schema.md` | The input format users use to define architectures (pending) |
| `rendering-engine.md` | How diagrams are rendered and animated (pending) |
| `flow-visualization.md` | How data/work flow is represented visually (pending) |
| `export-format.md` | Self-contained HTML export specification (pending) |
| `visual-design.md` | Aesthetic guidelines, color, typography, styling (pending) |

## Open Questions

1. What internal definition format best balances LLM-generability with rendering precision? (JSON schema, constrained DSL?)
2. What rendering technology? (SVG via D3, Canvas, WebGL?)
3. What does "flow" look like visually? (Animated particles, pulsing paths, sequential highlighting?)
4. Should diagrams be interactive when viewed? (Zoom, pan, click-to-inspect, toggle flows?)
5. Is there a build step, or does everything happen in-browser?
6. How does the agent skill/prompt reference the definition format spec to ensure valid output?
