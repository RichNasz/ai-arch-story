# Rendering Engine

## What

How the JavaScript inside the generated HTML file draws, lays out, and manages the interactive diagram. This spec covers the rendering approach, SVG structure, adaptive layout behavior, and interaction model.

## Why

The rendering engine is the most visible part of the system — it determines what the user actually sees. It must produce beautiful diagrams that work across devices, animate flows smoothly, and fit entirely within a single self-contained HTML file.

## Decision: Vanilla JS + SVG

No external JS libraries are inlined. The rendering engine is a purpose-built, minimal JavaScript module embedded in every generated HTML file.

**Why vanilla:**
- **Size** — Every byte is inlined into every HTML file. D3 adds ~80KB, ELK.js adds ~140KB. A purpose-built renderer can be under 15KB minified.
- **Control** — Architecture diagrams have a constrained visual vocabulary (rectangles, cylinders, paths, text). We don't need a general-purpose visualization framework.
- **Predictability** — No library version conflicts, no API surface to learn, no behaviors to override.
- **LLM-generability** — The Rust toolchain generates the JS; simpler code is easier to template and debug.

**What we give up:**
- No auto-layout in the browser. The Rust toolchain pre-computes all positions. The JS only scales and adapts — it does not run graph layout algorithms.
- More upfront work to build the renderer. Acceptable because the visual vocabulary is finite and well-defined.

## SVG Structure

The rendered diagram is a single `<svg>` element with layered `<g>` groups:

```
<svg viewBox="0 0 {width} {height}" class="arch-diagram">
  <defs>
    <!-- Reusable definitions: markers, gradients, filters, patterns -->
  </defs>
  <g class="layer-groups">
    <!-- Group boundaries (rendered first, behind everything) -->
  </g>
  <g class="layer-edges">
    <!-- Connection paths between nodes -->
  </g>
  <g class="layer-nodes">
    <!-- Architectural components -->
  </g>
  <g class="layer-labels">
    <!-- Edge labels, positioned along paths -->
  </g>
  <g class="layer-flow">
    <!-- Flow animation elements (particles, highlights) -->
  </g>
</svg>
```

**Layer order matters:** Groups render behind edges, edges behind nodes, nodes behind labels, and flow animations on top of everything.

## Node Rendering

Each node type from `diagram-schema.md` maps to an SVG shape:

| Node Type | SVG Shape | Construction |
|-----------|-----------|-------------|
| `service` | Rounded rectangle | `<rect rx="8">` |
| `datastore` | Cylinder | `<path>` with elliptical top/bottom |
| `queue` | Parallelogram | `<polygon>` with skewed vertices |
| `user` | Person silhouette | `<path>` icon + label below |
| `external` | Dashed rectangle | `<rect stroke-dasharray="...">` |
| `function` | Hexagon | `<polygon>` with 6 vertices |
| `gateway` | Diamond | `<polygon>` with 4 vertices |
| `frontend` | Rectangle with top bar | `<rect>` + `<rect>` (browser chrome hint) |
| `storage` | Rounded rectangle with fold | `<path>` (folder/document shape) |
| `generic` | Rectangle | `<rect>` |

Each node is a `<g class="node" data-id="{id}">` containing:
- The shape element
- A `<text>` label (centered, auto-wrapped if needed)
- An optional icon (inline SVG symbol from `<defs>`)

### Node Sizing

Node dimensions are computed by the Rust toolchain based on:
- Label text length and font metrics
- Icon presence
- Minimum size per node type
- A configurable padding value

The JS renderer uses the pre-computed dimensions directly.

## Edge Rendering

Edges are SVG `<path>` elements using cubic Bezier curves for smooth routing.

```svg
<g class="edge" data-id="{id}" data-from="{from}" data-to="{to}">
  <path d="M... C..." class="edge-path" />
  <text class="edge-label"><textPath href="#edge-{id}-path">{label}</textPath></text>
</g>
```

- **One-way edges** have an arrowhead marker at the target end
- **Two-way edges** have arrowhead markers at both ends
- **No-direction edges** have no markers
- Edge paths are pre-computed by the Rust toolchain to avoid node overlap
- Labels ride along the path using `<textPath>`, positioned at the midpoint

## Group Rendering

Groups are rendered as rounded rectangles behind their contained nodes.

```svg
<g class="group" data-id="{id}">
  <rect class="group-boundary" rx="12" />
  <text class="group-label">{label}</text>
</g>
```

- Group bounds are computed by the Rust toolchain as the bounding box of contained nodes plus padding
- **Nested groups** render inside their parent group, with slightly different styling (lighter background, thinner border) at each nesting level
- Group labels appear at the top-left inside the boundary
- The nesting depth is encoded as `data-depth="{n}"` for CSS styling

## Adaptive Layout

The Rust toolchain pre-computes layout at a **reference viewport** (default: 1920x1080). The JS renderer adapts this to the actual device.

### Scaling Strategy

```javascript
// Conceptual approach
const referenceWidth = diagramData.layout.width;
const referenceHeight = diagramData.layout.height;
const containerRect = container.getBoundingClientRect();
const scale = Math.min(
  containerRect.width / referenceWidth,
  containerRect.height / referenceHeight
);
```

The SVG `viewBox` is set to the reference dimensions. The SVG element scales to fit the container while preserving aspect ratio. This gives us device adaptation through SVG's native scaling rather than re-computing positions.

### Breakpoint Adjustments

Beyond simple scaling, the renderer makes targeted adjustments at small viewports:

| Viewport Width | Adjustment |
|---------------|------------|
| > 1200px | Full rendering, all labels, all metadata |
| 800–1200px | Reduce label font size, hide edge labels, simplify group borders |
| 400–800px | Hide node icons, abbreviate long labels, collapse nested group styling |
| < 400px | Minimal mode — shapes and primary labels only |

These are CSS-driven using media queries embedded in the HTML's `<style>` block, not JS re-layout.

## Interaction Model

The rendered HTML supports these interactions:

### Always Available
- **Pan** — Click/touch and drag to move the viewport
- **Zoom** — Scroll wheel or pinch to zoom in/out
- **Hover tooltips** — Hovering a node shows its `metadata.description` and `metadata.technology` in a tooltip

### Flow Controls (when flows are present)
- **Flow legend** — A small panel listing all flows with their color and label
- **Toggle flows** — Click a flow in the legend to show/hide its animation
- **Play/pause** — Global control to pause all flow animations

### Optional (enabled by diagram config)
- **Click to inspect** — Clicking a node opens a detail panel showing all metadata
- **Highlight path** — Clicking a node highlights all edges connected to it

## Data Embedding

The diagram definition data is embedded in the HTML as a JSON blob in a `<script>` tag:

```html
<script type="application/json" id="diagram-data">
{
  "layout": {
    "width": 1920,
    "height": 1080,
    "nodes": [
      { "id": "api", "x": 400, "y": 200, "width": 160, "height": 80 }
    ],
    "edges": [
      { "id": "e1", "path": "M400,240 C400,350 600,350 600,200" }
    ],
    "groups": [
      { "id": "backend", "x": 350, "y": 150, "width": 500, "height": 400 }
    ]
  },
  "diagram": { /* original diagram.json content for metadata/tooltips */ },
  "flows": [ /* flow definitions with resolved step coordinates */ ]
}
</script>
```

The Rust toolchain enriches the original `diagram.json` with computed layout data (positions, dimensions, edge paths) and writes this combined structure into the HTML. The JS renderer reads this blob and draws from it — no computation needed beyond scaling.

## HTML Structure

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{diagram title}</title>
  <style>/* all CSS inlined */</style>
</head>
<body>
  <div id="diagram-container">
    <!-- SVG injected by JS -->
  </div>
  <div id="flow-controls">
    <!-- Flow legend and play/pause, built by JS if flows exist -->
  </div>
  <div id="tooltip" hidden>
    <!-- Hover tooltip, positioned by JS -->
  </div>
  <script type="application/json" id="diagram-data">
    /* pre-computed layout + diagram definition */
  </script>
  <script>/* all JS inlined — renderer, interaction handlers, flow animation */</script>
</body>
</html>
```

## Performance Targets

| Metric | Target | Rationale |
|--------|--------|-----------|
| HTML file size | < 500KB for typical diagrams (< 50 nodes) | Must be email-attachable, fast to load |
| Initial render | < 200ms | No layout computation needed; just SVG construction from pre-computed data |
| Flow animation | 60fps on modern browsers | CSS animations and `requestAnimationFrame` |
| JS bundle size | < 15KB minified | Purpose-built renderer, no library overhead |
