# Custom Types

## What

User-definable node types that extend or override the 10 built-in types (service, datastore, queue, user, external, function, gateway, frontend, storage, generic). Custom types map to SVG shapes (built-in or imported) with custom accent colors, labels, and metadata. They live in workspace editing files and resolve through an inheritance chain at publish time.

## Why

The 10 built-in types cover common architecture patterns, but real-world diagrams need domain-specific types (Kafka topics, Kubernetes pods, ML models, CDN edges). Users need to:

- Define types that carry semantic meaning beyond "generic with a custom color"
- Share type palettes across related projects (e.g., a "cloud infrastructure" type library)
- Override built-in type visuals to match organizational branding
- Import custom SVG shapes from external vector editors for types that don't map to the 10 built-in shapes
- Have all of this resolve cleanly into self-contained HTML at publish time

## Design Principles

- **Editing-time structure, publish-time flatness** — Type definitions live in separate files during editing; at publish time, the Rust pipeline resolves and inlines everything into the HTML
- **Inheritance with override** — Built-in → library → project → diagram. Each level can add new types or override fields of existing ones
- **SVG shapes are importable** — New shape primitives come from SVG files created in external editors (Inkscape, Figma, Illustrator). The system consumes SVGs, it does not edit them
- **Built-in shapes are overridable** — An SVG file named `cylinder.svg` in the shapes directory replaces the programmatic cylinder renderer. No hard boundary between built-in and custom
- **Backward compatible** — Existing diagrams with only built-in types work unchanged

## Custom Type Definition Schema

A type definition maps a kebab-case key to a shape, accent color, and metadata.

### types.json

```json
{
  "types": {
    "kafka-topic": {
      "label": "Kafka Topic",
      "shape": "parallelogram",
      "gvShape": "parallelogram",
      "accentColor": "#F59E0B",
      "description": "An Apache Kafka event topic"
    },
    "kubernetes-pod": {
      "label": "K8s Pod",
      "shape": "hexagon",
      "accentColor": "#326CE5",
      "description": "A Kubernetes pod or deployment"
    },
    "cdn-edge": {
      "label": "CDN Edge",
      "shape": "cloud-native",
      "accentColor": "#FF9900",
      "description": "Content delivery network edge node"
    }
  }
}
```

### Type Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `label` | string | yes | — | Human-readable name shown in editor dropdowns and legends |
| `shape` | string | no | `"rounded-rect"` | Shape name — one of the 10 built-in names, or the filename (without `.svg`) of an imported SVG shape |
| `gvShape` | string | no | derived from `shape` | Graphviz shape for layout geometry approximation. If omitted, derived using the same mapping as built-in types |
| `accentColor` | string | no | `"#94A3B8"` | CSS color for the type's accent bar and UI badges |
| `description` | string | no | — | Tooltip text and agent hint for when to use this type |

### gvShape Derivation

When `gvShape` is omitted, it is derived from `shape` using this mapping (same as built-in types):

| shape | gvShape |
|-------|---------|
| `rounded-rect` | `box` |
| `cylinder` | `cylinder` |
| `parallelogram` | `parallelogram` |
| `person` | `house` |
| `dashed-rect` | `box` |
| `hexagon` | `hexagon` |
| `diamond` | `diamond` |
| `browser` | `box` |
| `folder` | `folder` |
| `rect` | `box` |
| (unknown/custom SVG) | `box` |

## Custom SVG Shape Imports

New shape primitives are SVG files stored in the workspace. The system imports them — editing happens in external vector tools.

### Shape File Location

```
<project-root>/
  shared/
    shapes/                    # Project-wide custom shape SVGs
      cloud-native.svg
      database-cluster.svg
    types.json                 # References shapes by filename (without .svg)
```

A custom type references an imported shape by its filename (without `.svg` extension):

```json
{
  "types": {
    "cloud-native": {
      "label": "Cloud Native App",
      "shape": "cloud-native",
      "accentColor": "#3B82F6"
    }
  }
}
```

### SVG Requirements

| Requirement | Detail |
|-------------|--------|
| `viewBox` attribute | Required. The renderer scales the SVG to fit node dimensions using the viewBox |
| `currentColor` for themed elements | Strokes and fills that should pick up the accent color must use `currentColor` |
| Simple outlines | The renderer adds labels, icons, and accent bars on top of the shape. Keep shapes as outlines/containers |
| Recommended canvas | 100×100 viewBox for square shapes, 150×100 for wide shapes |
| No external references | No `<use href="...">`, no linked stylesheets. Everything inline |

### Shape Override Resolution

Every shape — built-in or custom — follows the same resolution:

1. Check for an SVG file override matching the shape name (e.g., `cylinder.svg` overrides the built-in cylinder)
2. Fall back to the programmatic renderer in `renderer.js`
3. Final fallback to `rounded-rect`

SVG file search order:

1. Diagram-level: `diagrams/<name>/assets/shapes/`
2. Project-level: `shared/shapes/`
3. Type library shapes (see Type Libraries below)
4. Built-in programmatic shapes in `renderer.js`

This means a workspace with `shared/shapes/cylinder.svg` gets a custom database shape across all diagrams, while workspaces without that file get the default programmatic cylinder.

## Where Type Definitions Live

### Hierarchy

| Level | Location | Scope |
|-------|----------|-------|
| Built-in | Compiled into the Rust binary | Always available — the 10 default types |
| Type library | External `types.json` + `shapes/` directory referenced from `project.json` | Shareable across projects |
| Project | `shared/types.json` + `shared/shapes/` | All diagrams in the project |
| Diagram | `custom_types` field in `diagram.json` + `assets/shapes/` | Single diagram only |

### project.json: Type Libraries

For cross-workspace sharing, `project.json` gains a `type_libraries` field:

```json
{
  "name": "E-Commerce Platform",
  "version": "1.0",
  "type_libraries": [
    { "path": "../shared-libs/cloud-provider-types/" },
    { "path": "../shared-libs/k8s-types/" }
  ]
}
```

A type library directory contains:

```
cloud-provider-types/
  types.json
  shapes/
    load-balancer.svg
    cdn-node.svg
```

Libraries are resolved as relative file paths from the project root. No package registry, no versioning, no URL fetching — just files on disk.

### diagram.json: Inline Custom Types

Diagram-level type definitions are inline in `diagram.json`:

```json
{
  "version": "1.0",
  "title": "ML Pipeline",
  "custom_types": {
    "types": {
      "ml-model": {
        "label": "ML Model",
        "shape": "hexagon",
        "accentColor": "#7C3AED",
        "description": "A trained machine learning model"
      }
    }
  },
  "nodes": [
    { "id": "recommender", "label": "Recommender Model", "type": "ml-model" }
  ]
}
```

## Resolution Order

When the toolchain builds a diagram, types are resolved in this order (later wins):

1. **Built-in types** — the 10 hardcoded types, always present
2. **Type libraries** — from `project.json` `type_libraries`, merged in array order (later entries win on key collision)
3. **Project types** — `shared/types.json`
4. **Diagram types** — `custom_types` in `diagram.json`

### Field-Level Merge

Override is field-level, not wholesale replacement. A diagram that sets only `accentColor` for `"service"` keeps the built-in's `rounded-rect` shape and `"Service"` label:

```json
{
  "custom_types": {
    "types": {
      "service": {
        "accentColor": "#CC0000"
      }
    }
  }
}
```

Result: `service` type has shape `rounded-rect`, accent color `#CC0000`, label `Service`.

## Resolved Type Registry

At build time, the Rust pipeline merges all levels into a single `ResolvedTypeRegistry`. This registry is used throughout the pipeline and embedded in the published HTML.

### Built-In Type Definitions

For reference, the 10 built-in types as they appear in the registry:

| Key | Label | Shape | gvShape | Accent Color |
|-----|-------|-------|---------|-------------|
| `service` | Service | `rounded-rect` | `box` | `#3B82F6` |
| `datastore` | Datastore | `cylinder` | `cylinder` | `#8B5CF6` |
| `queue` | Queue | `parallelogram` | `parallelogram` | `#F59E0B` |
| `user` | User | `person` | `house` | `#14B8A6` |
| `external` | External | `dashed-rect` | `box` | `#6B7280` |
| `function` | Function | `hexagon` | `hexagon` | `#EC4899` |
| `gateway` | Gateway | `diamond` | `diamond` | `#06B6D4` |
| `frontend` | Frontend | `browser` | `box` | `#10B981` |
| `storage` | Storage | `folder` | `folder` | `#A78BFA` |
| `generic` | Generic | `rect` | `box` | `#94A3B8` |

### Unknown Type Handling

If a diagram references a type not in the resolved registry (e.g., a type library was removed), the toolchain:

1. Emits a validation warning: `"Node 'foo' uses unknown type 'bar', falling back to 'generic'"`
2. Applies `generic` defaults (rect shape, `#94A3B8` accent)
3. Does **not** fail the build — forward compatibility over strictness

## Published HTML

At publish time:

1. The Rust pipeline resolves all types into the registry
2. Each `LayoutNode` gets its resolved `shape` and `accent_color` as strings
3. Imported SVG shapes are read from disk and inlined as strings
4. The `diagram-data` JSON embedded in the HTML includes:
   - Per-node `shape` and `accent_color` (already present)
   - A `shapeOverrides` map: `{ "cloud-native": "<svg viewBox='...' ...>...</svg>" }` for any SVG shapes used by the diagram
   - A `typeRegistry` object: the full resolved type definitions (for legends, tooltips, filtering)

The HTML remains self-contained — no external file references.

## Renderer.js Shape Dispatch

Shape dispatch becomes a unified two-step lookup for all shapes:

```
1. Check shapeOverrides[node.shape] → use SVG template renderer
2. Fall back to shapes[node.shape] → use programmatic renderer
3. Final fallback: shapes['rounded-rect']
```

The SVG template renderer:
- Parses the SVG string from `shapeOverrides`
- Scales it to the node's bounding box using the SVG's `viewBox`
- Sets CSS `color` to the node's accent color (shapes use `currentColor`)
- Inserts it into the node's `<g>` element

## Web Editor

### Types Tab

A new sidebar tab alongside Nodes, Edges, Flows, Groups, Branding:

- **Type list**: shows all resolved types (built-in marked as such, project and diagram types editable)
- **Add type form**: key (kebab-case, validated for uniqueness), label, shape picker, color picker, description, scope selector (project vs. diagram)
- **Shape picker**: visual grid showing all available shapes (the 10 built-in + any imported SVGs)
- **SVG import**: file upload that saves the SVG to the appropriate `shapes/` directory
- **Edit type**: modify any field; built-in types show overridable fields with "reset to default" option

### NodesTab Changes

The hardcoded `NODE_TYPES` array and `TYPE_COLORS` map are replaced with data fetched from the API. The type dropdown renders all resolved types with their labels and accent-colored badges.

### API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/types` | Returns the fully resolved type registry for the current workspace |
| GET | `/api/v1/diagrams/{name}/types` | Returns the fully resolved registry for a specific diagram, including its inline custom types |
| GET | `/api/v1/project/types` | Returns `shared/types.json` (project-level definitions only) |
| PUT | `/api/v1/project/types` | Updates `shared/types.json` |
| GET | `/api/v1/diagrams/{name}/custom-types` | Returns diagram-level `custom_types` |
| PUT | `/api/v1/diagrams/{name}/custom-types` | Updates diagram-level `custom_types` |
| POST | `/api/v1/project/shapes` | Upload an SVG file to `shared/shapes/` |
| GET | `/api/v1/project/shapes` | List available shape SVG files |
| DELETE | `/api/v1/project/shapes/{name}` | Remove a shape SVG file |

## Agent Skill Integration

The agent checks the resolved type registry before falling back to built-in type inference:

1. Call `GET /api/v1/types` to discover available types
2. When the user says "add a Kafka topic" and a `kafka-topic` type exists, use it instead of guessing `queue`
3. When the user repeatedly mentions a concept with no matching type, suggest creating a custom type
4. When generating `diagram.json`, use custom type keys in the `type` field — the toolchain resolves them

## Example: Complete Workspace with Custom Types

```
ecommerce-platform/
  project.json                          # type_libraries: ["../k8s-types/"]
  shared/
    types.json                          # "redis-cache": { shape: "cylinder", accentColor: "#DC2626" }
    shapes/
      service-mesh.svg                  # Custom SVG for Istio service mesh nodes
    components.json
    theme.json
  diagrams/
    checkout-flow/
      diagram.json                      # custom_types: { "payment-gateway": { ... } }
      assets/
        shapes/
          payment-icon.svg              # Diagram-specific shape
      output/
        checkout-flow.html              # Self-contained, all types/shapes inlined

../k8s-types/                           # Shared type library
  types.json                            # "k8s-pod", "k8s-service", "k8s-ingress"
  shapes/
    pod.svg
    ingress.svg
```
