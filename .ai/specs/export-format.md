# Export Format

## What

How the Rust toolchain assembles a self-contained HTML file from the diagram definition, pre-computed layout, and rendering assets.

## Why

The core promise of the system is that the output is a single HTML file with zero external dependencies. This spec defines exactly what goes into that file, how it's assembled, and what guarantees it provides.

## Self-Containment Guarantees

The generated HTML file:

1. **Contains no external references** — no `<link>`, `<script src>`, `<img src>` pointing to CDNs, URLs, or local files
2. **Works offline** — opening the file from disk in any modern browser renders the full diagram
3. **Works without a server** — `file://` protocol is sufficient; no localhost needed
4. **Is a single file** — no sidecar files, no asset folders, one `.html` file is the complete artifact

## Assembly Pipeline

The Rust toolchain executes these steps to produce the HTML:

```
diagram.json
    │
    ▼
┌──────────────────────┐
│  1. Schema Validation │  Validate against diagram-schema rules
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│  2. Resolution       │  Merge shared components, theme, project defaults
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│  3. Layout Engine    │  Compute node positions, edge paths, group bounds
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│  4. Flow Resolution  │  Map flow steps to computed edge paths, generate
│                      │  animation coordinates
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│  5. Asset Collection │  Gather icons, inline any diagram-specific assets
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│  6. HTML Assembly    │  Inject CSS, JS, layout data, SVG defs into template
└──────────┬───────────┘
           │
           ▼
      output.html
```

## HTML Template Structure

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{{diagram.title}}</title>
  <style>
    {{INLINED_CSS}}
  </style>
</head>
<body data-theme="{{theme.mode}}">
  <header id="diagram-header">
    <h1>{{diagram.title}}</h1>
    <p>{{diagram.description}}</p>
  </header>

  <main id="diagram-container">
    <!-- SVG built by JS from embedded data -->
  </main>

  <aside id="flow-controls" hidden>
    <!-- Built by JS if flows exist -->
  </aside>

  <div id="tooltip" hidden></div>
  <div id="inspector" hidden></div>

  <script type="application/json" id="diagram-data">
    {{DIAGRAM_DATA_JSON}}
  </script>

  <script>
    {{INLINED_JS}}
  </script>
</body>
</html>
```

## Inlined Sections

### CSS (`{{INLINED_CSS}}`)

Contains all styles for:
- Base layout (body, container, header)
- SVG element styling (nodes, edges, groups, labels)
- Node type accent colors
- Group depth styling
- Flow animation keyframes
- Flow controls panel
- Tooltip and inspector panel
- Dark/light mode variables
- Responsive breakpoint adjustments (media queries)
- `prefers-reduced-motion` overrides

Assembled by the Rust toolchain from a base CSS template with theme variables injected.

### Diagram Data (`{{DIAGRAM_DATA_JSON}}`)

A single JSON object containing everything the JS renderer needs:

```json
{
  "meta": {
    "title": "...",
    "description": "...",
    "theme": "default",
    "generated": "2026-08-21T10:30:00Z",
    "generator": "ai-arch-story v0.1.0"
  },
  "layout": {
    "width": 1920,
    "height": 1080,
    "nodes": [
      {
        "id": "api",
        "label": "Order API",
        "type": "service",
        "x": 400, "y": 200,
        "width": 160, "height": 80,
        "icon": "...",
        "metadata": {}
      }
    ],
    "edges": [
      {
        "id": "e1",
        "from": "api", "to": "db",
        "label": "SQL queries",
        "direction": "one-way",
        "path": "M480,280 C480,350 640,350 640,320",
        "labelPosition": { "x": 560, "y": 310 }
      }
    ],
    "groups": [
      {
        "id": "backend",
        "label": "Backend Services",
        "x": 350, "y": 150,
        "width": 500, "height": 400,
        "depth": 0
      }
    ]
  },
  "flows": [
    {
      "id": "order-flow",
      "label": "New Order",
      "description": "...",
      "color": "#10B981",
      "animation": "particle",
      "speed": "normal",
      "steps": [
        {
          "edge": "e1",
          "label": "Submit order",
          "path": "M480,280 C480,350 640,350 640,320",
          "fromNode": "api",
          "toNode": "db"
        }
      ]
    }
  ],
  "icons": {
    "service": "<svg>...</svg>",
    "datastore": "<svg>...</svg>"
  }
}
```

### JavaScript (`{{INLINED_JS}}`)

The complete rendering engine as a single minified script. Responsibilities:

1. Parse the `#diagram-data` JSON blob
2. Build the SVG DOM from layout data
3. Set up pan/zoom handlers
4. Initialize flow animations (if flows present)
5. Build flow controls panel (if flows present)
6. Attach tooltip handlers

The JS is generated from source files in the Rust project, minified at build time, and injected as a string.

## File Size Budget

| Component | Target Size | Notes |
|-----------|------------|-------|
| HTML shell | < 1KB | Template boilerplate |
| CSS | < 8KB | All styles including themes and animations |
| JS (minified) | < 15KB | Renderer, interactions, flow animations |
| Diagram data | Varies | ~200 bytes per node, ~150 per edge, ~300 per flow step |
| Icons | < 5KB | Minimal inline SVG icon set |
| **Total (50-node diagram)** | **< 50KB** | Well under email attachment limits |

## Output Path

The generated HTML is written to:
- **Project mode:** `<project-root>/diagrams/<diagram-name>/output/<diagram-name>.html`
- **Standalone mode:** `<diagram-root>/output/<diagram-name>.html`

The `output/` directory is created if it doesn't exist. Existing files are overwritten (the definition is the source of truth, not the generated output).

## Browser Compatibility

The generated HTML targets:
- Chrome/Edge 90+
- Firefox 90+
- Safari 15+
- Mobile Chrome and Safari (iOS 15+, Android Chrome 90+)

No IE11 support. No polyfills.
