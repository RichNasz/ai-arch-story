# Agent Skill

## What

How the AI agent (e.g., Claude Code) interprets user conversations and generates valid diagram definitions. This spec defines the agent's responsibilities, the conversational workflow, and the contract it must uphold.

## Why

Users never write JSON directly. The agent is the sole producer of diagram definitions. It must:

- Understand architectural descriptions in natural language
- Generate schema-valid `diagram.json` files
- Manage the workspace structure (projects, shared components, multiple diagrams)
- Support iterative refinement through conversation
- Invoke the Rust toolchain to produce HTML output

## Agent Responsibilities

### 1. Workspace Management

- **Initialize projects** — Guide the user through `ai-arch-story start` at a user-specified path. The command confirms the resolved path and derived project name before safely creating `project.json`, `shared/`, and `diagrams/`.
- **Discover existing projects** — Detect `project.json` to load project context
- **Create diagrams** — Create new diagram folders with `diagram.json` under the project
- **Navigate diagrams** — List and switch between diagrams in a project

### 2. Definition Generation

- **Translate conversation to JSON** — Map architectural descriptions to nodes, edges, flows, and groups per `diagram-schema.md`
- **Generate stable IDs** — Use kebab-case slugs derived from labels (e.g., "Order API" → `order-api`)
- **Infer structure** — When a user says "the API talks to the database", create both nodes, the edge, and appropriate types
- **Apply defaults** — Omit optional fields; let the visual design system handle styling unless the user requests specifics
- **Reuse shared components** — When inside a project, check `shared/components.json` before creating new nodes

### 3. Iterative Refinement

- **Read existing definitions** — Load the current `diagram.json` before making changes
- **Apply targeted edits** — When the user says "add a cache between the API and database", modify the existing definition rather than regenerating from scratch
- **Preserve user intent** — Don't remove or restyle elements the user hasn't mentioned
- **Explain changes** — Briefly describe what was added, removed, or changed

### 4. Build Invocation via HTTP API

For a new project, the agent first directs the user to run
`ai-arch-story start`; it does not bypass the command's confirmation and repair
rules by creating workspace files itself. Once `ai-arch-story serve` is
running against that valid workspace, the agent uses the HTTP API for all
operations. This ensures consistency with the web editor — both paths share
the same validation and rendering pipeline.

**API base**: `http://localhost:8080/api/v1` (default port; may vary)

**Read operations** (via curl):
```bash
# List diagrams
curl -s http://localhost:8080/api/v1/diagrams

# Read a diagram
curl -s http://localhost:8080/api/v1/diagrams/{name}

# List nodes
curl -s http://localhost:8080/api/v1/diagrams/{name}/nodes
```

**Write operations**:
```bash
# Add a node
curl -s -X POST http://localhost:8080/api/v1/diagrams/{name}/nodes \
  -H 'Content-Type: application/json' \
  -d '{"id":"redis-cache","label":"Redis Cache","type":"datastore","metadata":{}}'

# Update a node
curl -s -X PUT http://localhost:8080/api/v1/diagrams/{name}/nodes/{id} \
  -H 'Content-Type: application/json' \
  -d '{"id":"redis-cache","label":"Redis Cache (Primary)","type":"datastore","metadata":{}}'

# Delete a node (cascades to edges, flow steps, group membership)
curl -s -X DELETE http://localhost:8080/api/v1/diagrams/{name}/nodes/{id}

# Add an edge
curl -s -X POST http://localhost:8080/api/v1/diagrams/{name}/edges \
  -H 'Content-Type: application/json' \
  -d '{"id":"api-to-cache","from":"order-api","to":"redis-cache","label":"cache lookup","metadata":{}}'
```

**Validate and render**:
```bash
# Validate without rendering
curl -s -X POST http://localhost:8080/api/v1/diagrams/{name}/validate

# Render to HTML
curl -s -X POST http://localhost:8080/api/v1/diagrams/{name}/render
```

**Error handling**: All mutation endpoints validate before persisting. A 422 response means validation failed — the error message describes what's wrong. The agent should parse the error and fix the issue.

**Coexistence with web editor**: The agent and web editor use the same API. The server reads from disk on every request, so both see the same state. If a user edits a node in the web editor and then asks the agent to add a flow, the agent's next API call will see the web editor's changes.

## Conversational Patterns

### Initial Diagram Creation

```
User: "I need a diagram of our order processing system. We have a React
       frontend that calls a Node.js API. The API writes to Postgres and
       publishes events to Kafka. A fulfillment service consumes those events
       and calls a warehouse API."

Agent actions:
1. Identify nodes: frontend (React), api (Node.js), db (Postgres),
   queue (Kafka), fulfillment-service, warehouse-api (external)
2. Identify edges: frontend→api, api→db, api→queue, queue→fulfillment,
   fulfillment→warehouse-api
3. Infer a flow: order lifecycle from frontend through to warehouse
4. Infer a group: backend services
5. Generate diagram.json
6. Run toolchain
7. Report: "Created diagram with 6 components and an order flow.
   HTML output at: .../output/order-processing.html"
```

### Iterative Refinement

```
User: "Add a Redis cache between the API and Postgres"

Agent actions:
1. Read current diagram.json
2. Add node: redis-cache (type: datastore, technology: Redis)
3. Update edge: api→db becomes api→redis-cache and redis-cache→db
4. Update any flows that traversed the old api→db edge
5. Write updated diagram.json
6. Run toolchain
7. Report changes
```

### Style Requests

```
User: "Make the Kafka topic stand out more, maybe in orange"

Agent actions:
1. Read current diagram.json
2. Add style override to the Kafka node: {"color": "#F97316"}
3. Write updated diagram.json
4. Run toolchain
```

### Multi-Diagram

```
User: "Create a second diagram showing just the infrastructure view"

Agent actions:
1. Create diagrams/infrastructure/ folder
2. Reuse relevant shared components from shared/components.json
3. Generate infrastructure-focused diagram.json with deployment groups
4. Run toolchain
```

## Generation Rules

The agent follows these rules when producing `diagram.json`:

1. **Valid schema** — Output must pass all validation rules in `diagram-schema.md`
2. **Minimal output** — Omit optional fields unless the user specified them
3. **Stable IDs** — Derive from labels, kebab-case, unique within the diagram
4. **Infer node types** — "database" → `datastore`, "API" → `service`, "user" → `user`, etc.
5. **Infer edges from verbs** — "calls", "reads from", "publishes to", "consumes" → edges with appropriate direction
6. **Infer flows from narratives** — When the user describes a sequence ("first X, then Y, then Z"), create a flow
7. **Group logically** — Group related backend services together unless the user specifies grouping
8. **Don't over-infer** — If the user doesn't mention flows, don't invent them. If they don't mention groups, a flat diagram is fine.
9. **Preserve on edit** — When updating, only change what the user asked for. Don't reposition, restyle, or remove untouched elements.

## Type Inference with Custom Types

Before inferring a node type, the agent should check the project's resolved type registry by calling `GET /api/v1/types`. If the project defines a custom type whose label or description matches the user's language, use that custom type instead of a built-in.

For example, if a project defines `"kafka-topic"` with label `"Kafka Topic"`, and the user says "add a Kafka topic for order events", the agent should use `type: "kafka-topic"` rather than falling back to `type: "queue"`.

When no custom type matches, fall back to the built-in type inference table below.

### Type Inference Table

| User Language | Inferred Node Type |
|--------------|-------------------|
| database, DB, Postgres, MySQL, Mongo, DynamoDB | `datastore` |
| cache, Redis, Memcached, ElastiCache | `datastore` |
| queue, Kafka, RabbitMQ, SQS, event bus, topic | `queue` |
| API, service, microservice, backend, server | `service` |
| gateway, API gateway, load balancer, proxy, ALB | `gateway` |
| frontend, UI, web app, mobile app, browser, client | `frontend` |
| user, customer, admin, actor, person | `user` |
| external, third-party, vendor, partner API | `external` |
| lambda, function, serverless, worker, cron job | `function` |
| S3, blob storage, CDN, file storage, bucket | `storage` |

### Suggesting Custom Types

When the user repeatedly describes a concept that doesn't map well to a built-in type (e.g., "ML model", "Kubernetes pod", "message broker cluster"), the agent should suggest creating a custom type:

```
Agent: "You've mentioned several ML models. Would you like me to create a
       custom 'ml-model' type with a distinct shape and color? This would
       make them visually consistent across your diagrams."
```

If the user agrees, the agent creates the type via `PUT /api/v1/project/types`.

## Error Handling

| Error | Agent Response |
|-------|---------------|
| Schema validation failure | Show the validation error, offer to fix the definition |
| Referenced node doesn't exist | Create the missing node or ask the user which node they meant |
| Flow path is disconnected | Show the gap, ask user how the steps connect |
| Toolchain build failure | Show the error output, diagnose, and retry |

## Skill Integration

The agent skill should be implementable as a Claude Code skill or similar agent framework capability. The spec files in `.ai/specs/` serve as the agent's reference for:

- `diagram-schema.md` — What valid JSON looks like
- `workspace-structure.md` — Where to create files
- `visual-design.md` — What styling options exist
- `flow-visualization.md` — What flow animation options exist
- `web-api.md` — HTTP API endpoints for reading, writing, validating, and rendering
- `custom-types.md` — Custom type definitions, SVG shape imports, type libraries
- `container-modes.md` — How to start the container in serve mode
- `workspace-bootstrap.md` — How a user initializes, confirms, or repairs a workspace before serve mode
