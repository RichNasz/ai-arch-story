# Web API

## What

An HTTP API served by the `ai-arch-story` Rust binary (`serve` subcommand) that provides structured access to diagram workspaces. Both the web editor UI and Claude Code use this API as the single path for reading, writing, validating, and rendering diagrams.

## Why

The system needs two input paths — conversational AI (Claude Code) and direct manual editing (web editor) — that must stay in sync. By routing both through the same HTTP API with validation-on-write, drift is eliminated by construction. The API runs inside the same container that already bundles the Rust binary and Graphviz, requiring no additional local installs.

## API Design

### Base URL

`/api/v1/`

All endpoints return JSON (`Content-Type: application/json`) unless otherwise noted. Error responses use a consistent envelope:

```json
{
  "error": {
    "code": "VALIDATION_FAILED",
    "message": "Edge e1 references unknown node 'missing-node'",
    "details": [
      { "field": "edges[0].to", "message": "Node 'missing-node' does not exist" }
    ]
  }
}
```

### Project Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/project` | Returns `project.json` metadata (or 404 in standalone mode) |
| GET | `/api/v1/diagrams` | Lists all diagram folders in the workspace |

`GET /diagrams` response:
```json
{
  "diagrams": [
    { "name": "system-overview", "title": "System Overview", "hasOutput": true },
    { "name": "data-pipeline", "title": "Data Pipeline", "hasOutput": false }
  ]
}
```

### Diagram CRUD

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/diagrams/{name}` | Returns the full `diagram.json` |
| PUT | `/api/v1/diagrams/{name}` | Replaces the entire `diagram.json` (validates before writing) |
| POST | `/api/v1/diagrams` | Creates a new diagram (body: `{ "name": "...", "title": "..." }`) |
| DELETE | `/api/v1/diagrams/{name}` | Deletes the diagram folder |

PUT validates the full diagram before persisting. Returns 422 with structured errors on failure.

### Element CRUD

All four element types (nodes, edges, flows, groups) follow the same pattern:

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/diagrams/{name}/{elements}` | List all elements of this type |
| GET | `/api/v1/diagrams/{name}/{elements}/{id}` | Get a single element by ID |
| POST | `/api/v1/diagrams/{name}/{elements}` | Add a new element |
| PUT | `/api/v1/diagrams/{name}/{elements}/{id}` | Update an element |
| DELETE | `/api/v1/diagrams/{name}/{elements}/{id}` | Remove an element |

Where `{elements}` is one of: `nodes`, `edges`, `flows`, `groups`.

**Request/response types** are the existing `schema::types` structs (`Node`, `Edge`, `Flow`, `Group`). They already derive `Serialize` and `Deserialize`.

**Cascade deletion rules:**
- Deleting a **node** also removes all edges referencing it (as `from` or `to`) and removes those edges from any flow steps
- Deleting an **edge** also removes it from any flow steps that reference it
- Deleting a **group** does not delete its member nodes (they remain ungrouped)

**Validation on mutation:** Every POST, PUT, and DELETE re-validates the full diagram after applying the change. If validation fails, the change is rejected (422) and the file is not written.

### Action Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/diagrams/{name}/validate` | Validate diagram without side effects |
| POST | `/api/v1/diagrams/{name}/render` | Validate, run Graphviz layout, assemble HTML, write output file |
| GET | `/api/v1/diagrams/{name}/render-data` | Return the `DiagramRenderData` JSON (for live preview without writing HTML) |
| GET | `/api/v1/diagrams/{name}/preview` | Serve the most recently rendered HTML file |

`POST /validate` response:
```json
{ "valid": true }
```
or 422 with the error envelope above.

`POST /render` response:
```json
{ "outputPath": "diagrams/system-overview/output/system-overview.html" }
```

`GET /render-data` returns the same JSON structure that gets embedded in the generated HTML — the web editor's preview pane uses this with the existing `renderer.js` to draw live SVG.

### Shared Assets

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/shared/branding` | Returns `shared/branding.json` (or 404) |
| PUT | `/api/v1/shared/branding` | Updates `shared/branding.json` |
| GET | `/api/v1/shared/theme` | Returns `shared/theme.json` (or 404) |
| PUT | `/api/v1/shared/theme` | Updates `shared/theme.json` |

### Custom Types (see `custom-types.md`)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/types` | Returns the fully resolved type registry (built-in + libraries + project + diagram types merged) |
| GET | `/api/v1/diagrams/{name}/types` | Returns the fully resolved type registry including the named diagram's inline overrides; the editor uses this endpoint. |
| GET | `/api/v1/project/types` | Returns `shared/types.json` (project-level definitions only, or 404) |
| PUT | `/api/v1/project/types` | Updates `shared/types.json` |
| GET | `/api/v1/diagrams/{name}/custom-types` | Returns diagram-level `custom_types` from `diagram.json` |
| PUT | `/api/v1/diagrams/{name}/custom-types` | Updates diagram-level `custom_types` in `diagram.json` |
| POST | `/api/v1/project/shapes` | Upload an SVG file to `shared/shapes/` (multipart form data) |
| GET | `/api/v1/project/shapes` | List available shape SVG files |
| DELETE | `/api/v1/project/shapes/{name}` | Remove a shape SVG file |

## Server Configuration

The `serve` subcommand accepts:

| Flag | Default | Description |
|------|---------|-------------|
| `--workspace` | `/workspace` | Path to the project or standalone diagram directory |
| `--port` | `8080` | HTTP port to bind |
| `--host` | `0.0.0.0` | Bind address |
| `--static-dir` | (built-in) | Path to the webapp static files (overridable for development) |

## Concurrency

The server reads `diagram.json` from disk on every request — no in-memory cache. This ensures that if the file is modified externally (e.g., by a script or manual edit), the API always reflects the current state.

Write operations use filesystem-level atomicity: write to a temp file, then rename. This prevents partial writes from corrupting `diagram.json`.

## CORS

Enabled via `tower-http` for local development. Allows requests from any origin (the editor UI may be served on a different port during development).

## What This Spec Does NOT Cover

- Authentication or authorization (see `container-modes.md` for trust model)
- WebSocket for real-time change notifications (future enhancement)
- Batch operations across multiple diagrams
- The web editor UI itself (see `web-editor.md`)
