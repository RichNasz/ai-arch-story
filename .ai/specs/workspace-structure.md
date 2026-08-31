# Workspace Structure

## What

Defines how user-created content is organized on disk — the folder hierarchy for projects (thematic collections) and individual diagrams within them.

## Why

Users will create multiple diagrams that are often thematically related (e.g., all diagrams for a single product's architecture, or a set of diagrams for a migration plan). The workspace structure must:

- Allow users to specify where their content lives
- Support multiple diagrams within a thematic collection
- Share context (styles, terminology, shared components) across related diagrams
- Keep each diagram's artifacts self-contained and independently shareable
- Be intuitive enough that an AI agent can navigate and manage it

## Workspace Hierarchy

```
<user-chosen-path>/                     # Project root (thematic collection)
├── project.json                        # Project-level metadata and shared config
├── shared/                             # Shared assets across all diagrams in this project
│   ├── theme.json                      # Shared visual theme overrides
│   ├── components.json                 # Reusable component definitions
│   ├── glossary.json                   # Shared terminology and descriptions
│   ├── types.json                      # Custom node type definitions (see custom-types.md)
│   └── shapes/                         # Imported SVG shape files (see custom-types.md)
│       └── *.svg                       # Shape primitives created in external vector editors
├── diagrams/
│   ├── <diagram-name>/                 # One folder per diagram
│   │   ├── diagram.json               # The diagram definition (see diagram-schema.md)
│   │   ├── output/
│   │   │   └── <diagram-name>.html    # Generated self-contained HTML
│   │   └── assets/                    # Diagram-specific assets (custom icons, images)
│   ├── <another-diagram>/
│   │   ├── diagram.json
│   │   ├── output/
│   │   │   └── <another-diagram>.html
│   │   └── assets/
```

## Project Root

The user specifies the project root path. This is the top-level folder for a thematic collection of related diagrams. Examples:

- `~/architectures/ecommerce-platform/`
- `./docs/architecture/cloud-migration/`
- `/shared/team/diagrams/q3-redesign/`

### project.json

Project-level metadata and configuration shared across all diagrams in the collection.

```json
{
  "name": "E-Commerce Platform Architecture",
  "description": "Architecture diagrams for the e-commerce platform redesign",
  "version": "1.0",
  "defaults": {
    "theme": "default",
    "output_format": "html"
  },
  "type_libraries": [
    { "path": "../shared-libs/cloud-types/" }
  ],
  "metadata": {
    "author": "Platform Architecture Team",
    "created": "2026-08-21"
  }
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Human-readable project name |
| `description` | string | no | What this collection of diagrams represents |
| `version` | string | yes | Schema version for this project file |
| `defaults` | object | no | Default settings inherited by all diagrams unless overridden |
| `type_libraries` | array | no | Paths to shared type library directories (see `custom-types.md`) |
| `metadata` | object | no | Arbitrary project-level metadata |

## Shared Directory

The `shared/` directory contains assets and definitions that apply across all diagrams in the project. Diagrams inherit from these unless they override locally.

### shared/types.json

Custom node type definitions available to all diagrams in the project. Defines new types or overrides built-in type properties (shape, color, label). See `custom-types.md` for the full schema.

### shared/shapes/

Imported SVG shape files for custom shape primitives. Created in external vector editors (Inkscape, Figma, etc.). Referenced by filename (without `.svg`) from type definitions. Can also override built-in shapes (e.g., `cylinder.svg` replaces the programmatic cylinder). See `custom-types.md` for SVG requirements.

### shared/theme.json

Visual theme overrides applied to all diagrams in the project. Allows a team to define a consistent look across related diagrams. Structure follows the style system defined in `visual-design.md` (planned).

### shared/components.json

Reusable node definitions that can be referenced by any diagram in the project. Avoids re-describing the same service across multiple diagrams.

```json
{
  "components": [
    {
      "id": "api-gateway",
      "label": "API Gateway",
      "type": "gateway",
      "metadata": {
        "technology": "Kong",
        "owner": "Platform Team",
        "description": "Central API gateway handling auth, rate limiting, and routing"
      }
    },
    {
      "id": "orders-db",
      "label": "Orders Database",
      "type": "datastore",
      "metadata": {
        "technology": "PostgreSQL 15",
        "owner": "Orders Team"
      }
    }
  ]
}
```

When a diagram references a node ID that matches a shared component, the shared component's properties serve as defaults — the diagram can override any field.

### shared/glossary.json

Shared terminology and descriptions that the agent can use for consistency across diagrams.

```json
{
  "terms": {
    "order-event": "A Kafka event published when an order state changes, consumed by fulfillment and notification services",
    "service-mesh": "Istio-based service mesh handling mTLS, retries, and observability between backend services"
  }
}
```

## Diagram Directory

Each diagram lives in its own folder under `diagrams/`. The folder name is the diagram's slug (kebab-case).

### diagram.json

The diagram definition as specified in `diagram-schema.md`. When inside a project, the diagram can:

- Reference shared components by ID (resolved from `shared/components.json`)
- Inherit the project theme (from `shared/theme.json` or `project.json` defaults)
- Override any inherited value locally

### output/

Generated artifacts. The primary output is the self-contained HTML file. The Rust toolchain writes here; users share files from here.

### assets/

Optional diagram-specific assets such as custom icons or images that get inlined into the HTML during generation.

## Standalone Mode

A single diagram without a project is also valid. In this case the structure is just:

```
<user-chosen-path>/
├── diagram.json
├── output/
│   └── <name>.html
└── assets/
```

The Rust toolchain detects whether it's operating on a project (has `project.json`) or a standalone diagram (has `diagram.json` at root) and adjusts resolution accordingly.

## Resolution Order

When the toolchain builds a diagram, it resolves properties in this order (later wins):

1. **Built-in defaults** — from the visual design system compiled into the Rust binary
2. **Type libraries** — external type definitions referenced in `project.json` `type_libraries` (see `custom-types.md`)
3. **Project theme** — `shared/theme.json` if inside a project
4. **Project types** — `shared/types.json` for custom node type definitions (see `custom-types.md`)
5. **Project defaults** — `project.json` `defaults` section
6. **Shared components** — `shared/components.json` for matching node IDs
7. **Diagram definition** — values in `diagram.json` (including `custom_types`) override everything above

## Agent Interaction

When the user starts a session, the agent should:

1. Ask where to store content (or use a previously established path)
2. Check if a `project.json` exists at that path — if so, load project context
3. List existing diagrams in the project for reference
4. When creating a new diagram, create the folder structure under `diagrams/`
5. When updating shared components or theme, modify the `shared/` files
6. After generating or updating `diagram.json`, invoke the Rust toolchain to produce the HTML output
