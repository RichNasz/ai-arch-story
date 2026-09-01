# Diagram Schema

## What

The internal JSON definition format that serves as the contract between the AI agent (producer) and the Rust rendering toolchain (consumer). This is the intermediate representation at the center of the system pipeline.

## Why

This format must satisfy two masters:

1. **LLM-generability** — An AI agent must be able to produce valid, complete definitions from natural language descriptions. The schema must be predictable, unambiguous, and forgiving of ordering — an LLM should never need to "guess" at structure.
2. **Rendering precision** — The Rust toolchain must be able to unambiguously transform a definition into a visual diagram. Every visual element must be derivable from the definition without inference.

## Design Principles

- **JSON format** — Chosen over YAML/TOML for unambiguous parsing, strong LLM familiarity, and native Rust serde support
- **Flat references over nesting** — Components reference each other by ID rather than deep nesting. This makes partial updates easier for the agent during conversational iteration.
- **Sensible defaults** — Most visual properties (colors, sizes, positions) are optional. The toolchain applies defaults from the visual design system. The agent only specifies what the user explicitly requested.
- **Explicit flows** — Flows are first-class objects, not inferred from edges. A connection between two nodes can exist without a flow, and a flow can traverse multiple edges.
- **Extensible metadata** — Each element can carry a `metadata` object for tooltips, descriptions, or custom data that the rendered HTML can display on interaction.

## Schema Structure

### Top Level

```json
{
  "version": "1.0",
  "title": "My Architecture",
  "description": "Optional description shown in the rendered diagram",
  "theme": "default",
  "viewport": { "width": 2560, "height": 1600 },
  "custom_types": {},
  "nodes": [...],
  "edges": [...],
  "flows": [...],
  "groups": [...],
  "metadata": {}
}
```

### Viewport

Optional target display size for layout computation. The diagram is laid out to fill this viewport optimally, then scales adaptively to the actual viewing device.

```json
{
  "width": 2560,
  "height": 1600
}
```

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `width` | number | no | 1920 | Target viewport width in pixels |
| `height` | number | no | 1080 | Target viewport height in pixels |

Common presets the agent can use based on user intent:

| Device / Intent | Width | Height | Aspect Ratio |
|----------------|-------|--------|-------------|
| MacBook Pro 14" | 3024 | 1964 | 3:2 |
| MacBook Pro 16" | 3456 | 2234 | 3:2 |
| MacBook Air 13" | 2560 | 1664 | 3:2 |
| 1080p display | 1920 | 1080 | 16:9 |
| 4K display | 3840 | 2160 | 16:9 |
| iPad Pro 12.9" | 2048 | 2732 | 3:4 (portrait) |
| Presentation (16:9) | 1920 | 1080 | 16:9 |

The viewport does not constrain viewing — diagrams remain zoomable and pannable. It determines how the layout engine distributes elements across space for optimal first-impression rendering on the target display.

### Nodes

A node represents a component, service, system, datastore, or any architectural element.

```json
{
  "id": "api-gateway",
  "label": "API Gateway",
  "type": "service",
  "icon": "gateway",
  "style": {},
  "position": null,
  "metadata": {
    "description": "Kong-based API gateway handling auth and rate limiting",
    "technology": "Kong",
    "owner": "Platform Team"
  }
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Unique identifier, referenced by edges and flows |
| `label` | string | yes | Display name rendered on the diagram |
| `type` | string | no | Semantic type — a built-in type name or a custom type key (see `custom-types.md`). See Node Types below |
| `icon` | string | no | Icon identifier from the built-in icon set |
| `style` | object | no | Visual overrides (color, shape, size). See Style Object |
| `position` | object | no | Layout hint `{x, y}`. If null, Rust auto-layout positions the node |
| `metadata` | object | no | Arbitrary key-value data for tooltips and inspection |

#### Node Types

Semantic types that influence default shape, color, and icon:

| Type | Default Shape | Typical Use |
|------|--------------|-------------|
| `service` | Rounded rectangle | Application services, APIs, microservices |
| `datastore` | Cylinder | Databases, caches, object stores |
| `queue` | Parallelogram | Message queues, event streams, brokers |
| `user` | Person shape | End users, actors, personas |
| `external` | Dashed rectangle | External systems, third-party services |
| `function` | Hexagon | Serverless functions, lambdas, workers |
| `gateway` | Diamond | API gateways, load balancers, proxies |
| `frontend` | Rectangle with browser chrome | Web UIs, mobile apps |
| `storage` | Folder/bucket shape | File storage, blob storage, CDNs |
| `generic` | Rectangle | Anything else |

Custom types can be defined at the project or diagram level to extend this list. See `custom-types.md` for the full type definition schema, SVG shape imports, and inheritance model.

### Edges

An edge represents a connection between two nodes. Edges are structural — they show that a relationship exists. Flow animation is defined separately.

```json
{
  "id": "api-to-db",
  "from": "api-service",
  "to": "postgres-db",
  "label": "SQL queries",
  "direction": "one-way",
  "style": {},
  "metadata": {}
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Unique identifier, referenced by flows |
| `from` | string | yes | Source node ID |
| `to` | string | yes | Target node ID |
| `label` | string | no | Text displayed along the edge |
| `direction` | enum | no | `"one-way"` (default), `"two-way"`, `"none"` |
| `style` | object | no | Visual overrides (line style, color, thickness) |
| `metadata` | object | no | Arbitrary key-value data |

### Flows

A flow represents the movement of data or work through a sequence of edges. Flows are the animated, narrative element of the diagram — they tell the story of how something moves through the architecture.

Use this canonical complete flow shape. `steps` is an array of objects, not an
array of edge-ID strings. Animation configuration belongs inside `style`, not
at the flow's top level.

```json
{
  "id": "flow-id",
  "label": "Flow label",
  "description": "Optional narrative",
  "steps": [
    { "edge": "edge-id", "label": null, "description": null, "parallel": null }
  ],
  "style": { "color": null, "speed": null, "animation": "pulse" },
  "metadata": {}
}
```

Do not invent fields or move fields to a more convenient level. When unsure,
validate with the current AI Arch Story renderer or copy the corresponding
shape from a known-good `diagram.json` generated by that same version.

```json
{
  "id": "user-request-flow",
  "label": "User Request",
  "description": "A user request flows from the browser through the API to the database and back",
  "steps": [
    { "edge": "browser-to-gateway", "label": "HTTPS request", "description": "Browser sends a REST request to the gateway over TLS." },
    { "edge": "gateway-to-api", "label": "Routed request", "description": "Gateway authenticates the token and routes to the API." },
    { "edge": "api-to-db", "label": "SELECT query" },
    { "edge": "db-to-api", "label": "Result set" },
    { "edge": "api-to-gateway", "label": "JSON response" },
    { "edge": "gateway-to-browser", "label": "HTTPS response" }
  ],
  "style": {
    "color": "#3B82F6",
    "speed": "normal",
    "animation": "pulse"
  },
  "metadata": {}
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Unique identifier |
| `label` | string | yes | Display name, shown in flow legend/controls |
| `description` | string | no | Narrative description of what this flow represents |
| `steps` | array | yes | Ordered sequence of edge traversals |
| `steps[].edge` | string | yes | Edge ID to traverse |
| `steps[].label` | string | no | Label for this step (overrides edge label during flow animation) |
| `steps[].description` | string | no | Narrative text shown in the stepper panel when this step is active |
| `style` | object | no | Flow-specific visual properties |
| `style.color` | string | no | Flow color (distinguishes multiple flows) |
| `style.speed` | enum | no | `"slow"`, `"normal"` (default), `"fast"` |
| `style.animation` | enum | no | `"pulse"`, `"particle"`, `"highlight"` (default) |
| `metadata` | object | no | Arbitrary key-value data |

### Groups

A group visually contains a set of nodes, representing a logical boundary (e.g., a VPC, a Kubernetes cluster, a team boundary).

```json
{
  "id": "aws-vpc",
  "label": "AWS VPC (us-east-1)",
  "nodes": ["api-service", "postgres-db", "redis-cache"],
  "style": {
    "color": "#F59E0B",
    "border": "dashed"
  },
  "groups": ["private-subnet"],
  "metadata": {}
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Unique identifier |
| `label` | string | yes | Display name rendered as the group header |
| `nodes` | array | yes | Node IDs contained in this group |
| `groups` | array | no | Nested sub-group IDs (for groups within groups) |
| `style` | object | no | Visual overrides (background color, border style) |
| `metadata` | object | no | Arbitrary key-value data |

### Style Object

Shared style overrides available on nodes, edges, flows, and groups. All fields optional — the visual design system provides defaults.

```json
{
  "color": "#3B82F6",
  "background": "#EFF6FF",
  "border": "solid",
  "opacity": 1.0,
  "size": "medium",
  "shape": "rounded-rect"
}
```

Specific element types may support additional style fields documented in their section above.

## Validation Rules

The Rust toolchain validates definitions before rendering:

1. All `id` fields must be unique within their element type
2. Edge `from` and `to` must reference existing node IDs
3. Flow step `edge` must reference existing edge IDs
4. Group `nodes` must reference existing node IDs
5. Group `groups` must reference existing group IDs (no circular references)
6. Flow steps must form a traversable path (each step's edge must connect from where the previous step ended)
7. `version` must be a supported schema version

## Example: Complete Diagram Definition

```json
{
  "version": "1.0",
  "title": "E-Commerce Order Processing",
  "description": "How a customer order flows from the web frontend through processing to fulfillment",
  "theme": "default",
  "nodes": [
    { "id": "browser", "label": "Customer Browser", "type": "frontend" },
    { "id": "api", "label": "Order API", "type": "service", "metadata": { "technology": "Node.js" } },
    { "id": "orders-db", "label": "Orders DB", "type": "datastore", "metadata": { "technology": "PostgreSQL" } },
    { "id": "queue", "label": "Order Events", "type": "queue", "metadata": { "technology": "Kafka" } },
    { "id": "fulfillment", "label": "Fulfillment Service", "type": "service" },
    { "id": "warehouse", "label": "Warehouse System", "type": "external" }
  ],
  "edges": [
    { "id": "e1", "from": "browser", "to": "api", "label": "REST API" },
    { "id": "e2", "from": "api", "to": "orders-db", "label": "Write order" },
    { "id": "e3", "from": "api", "to": "queue", "label": "Publish event" },
    { "id": "e4", "from": "queue", "to": "fulfillment", "label": "Consume event" },
    { "id": "e5", "from": "fulfillment", "to": "warehouse", "label": "Fulfillment request" }
  ],
  "flows": [
    {
      "id": "order-flow",
      "label": "New Order",
      "description": "Customer places an order, which is persisted, published as an event, and sent to fulfillment",
      "steps": [
        { "edge": "e1", "label": "Submit order" },
        { "edge": "e2", "label": "Persist order" },
        { "edge": "e3", "label": "OrderCreated event" },
        { "edge": "e4", "label": "Process order" },
        { "edge": "e5", "label": "Ship items" }
      ],
      "style": { "color": "#10B981", "animation": "particle" }
    }
  ],
  "groups": [
    {
      "id": "backend",
      "label": "Backend Services",
      "nodes": ["api", "orders-db", "queue", "fulfillment"]
    }
  ],
  "metadata": {
    "author": "Architecture Team",
    "created": "2026-08-21"
  }
}
```

## Agent Generation Notes

When the AI agent generates this format from conversation:

- Generate stable, descriptive IDs (kebab-case derived from the label)
- Omit optional fields rather than including empty/null values
- Only set `position` if the user explicitly requests placement ("put the database on the right")
- Only set `style` overrides if the user requests specific visuals ("make the Kafka topic red")
- Always generate at least one flow if the user describes how data or work moves through the system
- Use `metadata.description` to capture contextual details the user mentions that aren't structural
- Treat the executable schema as authoritative: never introduce unknown or
  guessed JSON fields. Validate before reporting a diagram complete; if a
  validator is unavailable, copy the field shape from a known-good
  `diagram.json` produced by the same AI Arch Story version.
