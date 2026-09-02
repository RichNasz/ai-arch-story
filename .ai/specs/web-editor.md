# Web Editor

## What

A form-based web editor for creating and modifying architecture diagrams, built with React and PatternFly. Served as static files from the same container that runs the API server and Graphviz. Provides a live SVG preview pane that reuses the existing `renderer.js`.

## Why

Many diagram edits are deterministic — renaming nodes, changing types, adjusting styles, adding edges, configuring branding. These don't benefit from AI conversation and waste tokens when done through Claude Code. A web editor handles these edits directly while Claude Code handles the creative, non-deterministic work (inferring architecture from natural language, choosing visualization patterns, flow storytelling).

## Design Principles

- **Form-based, not canvas-based** — Users edit properties through structured PatternFly forms, not by dragging nodes on a canvas. The preview pane shows the result.
- **Live preview** — Changes are reflected in the SVG preview immediately (or on save) using the existing `renderer.js` and `styles.css` that power the generated HTML output. The preview is pixel-identical to the export.
- **API-first** — All reads and writes go through the HTTP API (see `web-api.md`). The editor has no direct filesystem access.
- **PatternFly design system** — Uses React + PatternFly as the standard for Red Hat internal web applications.

## Layout

```
┌────────────────────────────────────────────────────────────┐
│  Masthead: Project name / Diagram selector / Actions       │
├──────────────────────┬─────────────────────────────────────┤
│  Sidebar (320px)     │  Preview Pane                       │
│                      │                                     │
│  [Nodes] [Edges]     │  ┌─────────────────────────────┐   │
│  [Flows] [Groups]    │  │                             │   │
│  [Types] [Branding]  │  │     Live SVG Rendering      │   │
│                      │  │     (renderer.js)           │   │
│  ┌────────────────┐  │  │                             │   │
│  │ Element List   │  │  │                             │   │
│  │ + Add button   │  │  └─────────────────────────────┘   │
│  │                │  │                                     │
│  │ node-1 [edit]  │  │  Flow controls (when flow selected) │
│  │ node-2 [edit]  │  │                                     │
│  │ node-3 [edit]  │  │                                     │
│  └────────────────┘  │                                     │
│                      │                                     │
│  ┌────────────────┐  │                                     │
│  │ Edit Form      │  │                                     │
│  │ (selected el.) │  │                                     │
│  │                │  │                                     │
│  │ Label: [____]  │  │                                     │
│  │ Type:  [____]  │  │                                     │
│  │ Style: [____]  │  │                                     │
│  └────────────────┘  │                                     │
├──────────────────────┴─────────────────────────────────────┤
│  Status bar: Validation status / Last saved / Render link  │
└────────────────────────────────────────────────────────────┘
```

## Sidebar Tabs

### Nodes Tab
- List of all nodes with id, label, and type badge
- Add node form: label (required), type (dropdown of all resolved types — built-in and custom, fetched from `/api/v1/types`), id (auto-generated from label, editable)
- Edit node form: label, type, icon, style overrides (color, background, border, opacity, shape), metadata key-value pairs

### Edges Tab
- List of all edges with from → to labels
- Add edge form: from (node dropdown), to (node dropdown), label, direction (one-way/two-way/none)
- Edit edge form: label, direction, style overrides

### Flows Tab
- List of flows with label and step count
- Add flow form: label, description
- Flow step editor: ordered list of steps, each referencing an edge (dropdown), with optional step label and description, parallel flag
- Flow style: color picker, speed (slow/normal/fast), animation type (pulse/particle/highlight)

### Groups Tab
- List of groups with label and member count
- Add group form: label, node multi-select
- Edit group form: label, nodes (multi-select), sub-groups (multi-select from other groups), style overrides

### Types Tab (see `custom-types.md`)
- List of all resolved types (built-in types marked as such, project and diagram types editable)
- Add type form: key (kebab-case, validated for uniqueness), label, shape picker (visual grid of available shapes), color picker, description, scope selector (project vs. diagram)
- SVG shape import: file upload that saves the SVG to `shared/shapes/` or `assets/shapes/`
- Edit type: modify any field; built-in types show overridable fields with "reset to default" option
- Shape preview: shows the selected shape rendered with the chosen accent color

### Branding Tab
- Organization name
- Logo upload (converts to data URI via API)
- Brand colors (primary, secondary) with color picker
- Footer text, show generated date toggle
- Favicon upload

## Preview Pane

The preview pane renders the diagram using the same JavaScript (`renderer.js`) and CSS (`styles.css`) that are embedded in the generated HTML output. This guarantees the preview is pixel-identical to what the user gets when they export.

**How it works:**
1. Editor calls `GET /api/v1/diagrams/{name}/render-data` to get the `DiagramRenderData` JSON
2. The preview pane passes this data to `renderer.js` functions to draw the SVG
3. Pan/zoom and flow controls work in the preview just as they do in the exported HTML

**When to refresh:**
- On save (after a successful PUT/POST/DELETE that changes the diagram)
- Full re-layout (calls `POST /render` then refreshes render-data) occurs when nodes or edges are added or removed, and when the user exports HTML

## HTML Export

The masthead provides an **Export HTML** action for the selected diagram. It
calls `POST /api/v1/diagrams/{name}/render`, which validates the definition,
computes layout, and writes the portable HTML artifact to its standard output
path. Once that succeeds, the editor initiates a browser download from the
diagram's preview URL using `<diagram-name>.html` as the filename.

The export control is disabled when no diagram is selected and while an export
is in progress. Render or validation errors are shown in the editor's standard
alert area and no download is started. The status bar continues to offer an
"Open rendered diagram" link after a successful export.

## PatternFly Components

Key PatternFly components used:

| Component | Usage |
|-----------|-------|
| `Masthead` | Top navigation with project/diagram selector |
| `Page`, `PageSection`, `PageSidebar` | Layout structure |
| `Tabs` | Sidebar tab switching (Nodes/Edges/Flows/Groups/Branding) |
| `DataList` | Element lists in each tab |
| `Form`, `FormGroup` | Edit forms for element properties |
| `TextInput`, `TextArea` | Text fields |
| `Select`, `SelectOption` | Dropdowns (node type, edge direction, flow speed) |
| `ColorPicker` | Style color selection |
| `Switch` | Boolean toggles (parallel flag, show generated date) |
| `Button` | Add, save, delete, render actions |
| `Alert` | Validation errors, save confirmation |
| `Modal` | Delete confirmation, new diagram creation |
| `DragDrop` | Reordering flow steps |

## Build Toolchain

The React + PatternFly app is built at container image build time:

1. A Node.js build stage in the Containerfile runs `npm install` + `npm run build`
2. The output is a `dist/` directory of static HTML/JS/CSS files
3. These static files are copied into the runtime image
4. The axum server serves them via `tower-http::services::ServeDir`
5. The existing `templates/renderer.js` and `templates/styles.css` are also served as static files for the preview pane

The build toolchain (Node.js, npm, React dev dependencies) exists only in the
build stage and is not present in the runtime image. The UBI 9 runtime decision
therefore does not require a Red Hat Node.js builder image; it constrains the
distribution runtime and the Rust binary ABI copied into it. See
`container-modes.md` for that compatibility contract.

## What This Spec Does NOT Cover

- API endpoint details (see `web-api.md`)
- Container configuration (see `container-modes.md`)
- Visual drag-and-drop node positioning
- Real-time collaborative editing
- Mobile-optimized layout (desktop-first editor)
