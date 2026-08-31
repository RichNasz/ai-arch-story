# Branding

## What

A minimal branding system that lets users stamp diagrams with their organization's identity — logo, name, colors, and attribution. Branding is defined at the workspace (project) level and optionally overridden or suppressed per diagram.

## Why

Architecture diagrams are shared artifacts — they appear in presentations, docs, Slack threads, and emails. Users need their diagrams to look like they belong to their organization without manually editing the HTML output. Branding sits above theming: theming controls *how things look*, branding controls *whose they are*.

## Design Principles

- **Optional and non-intrusive** — Diagrams look great with no branding at all. Branding adds identity without cluttering the diagram.
- **Workspace-first** — Define once in the project, inherit everywhere. Per-diagram overrides are the exception.
- **Self-contained** — Brand assets (logos, fonts) are inlined into the HTML output. No external references.
- **Minimal footprint** — Logo files must be small enough to inline without blowing the file size budget (see `export-format.md`).

## Branding Definition

### Project Level: `shared/branding.json`

```json
{
  "organization": "Acme Corp",
  "logo": {
    "src": "logo.svg",
    "alt": "Acme Corp logo",
    "placement": "header",
    "height": 28
  },
  "colors": {
    "primary": "#EE0000",
    "secondary": "#2B2B2B"
  },
  "footer": {
    "text": "Acme Corp — Architecture Team",
    "showGeneratedDate": true
  },
  "favicon": {
    "src": "favicon.png"
  }
}
```

All fields are optional. An empty `branding.json` or its absence means no branding is applied.

### Field Reference

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `organization` | string | — | Organization name; used in header if no logo, and in default footer text |
| `logo.src` | string | — | Path to logo file relative to `shared/`, or inline `data:` URI. SVG recommended. |
| `logo.alt` | string | `organization` value | Alt text for accessibility |
| `logo.placement` | enum | `"header"` | Where the logo appears: `"header"`, `"corner"` |
| `logo.height` | number | 24 | Logo height in pixels (width scales proportionally) |
| `colors.primary` | string | — | Primary brand color; used for header accent and active UI elements |
| `colors.secondary` | string | — | Secondary brand color; available for group borders or edge accents |
| `footer.text` | string | — | Custom footer text. If absent and `organization` is set, defaults to the organization name. |
| `footer.showGeneratedDate` | boolean | `false` | Append generation timestamp to footer |
| `favicon.src` | string | — | Path to favicon file relative to `shared/`, or inline `data:` URI. PNG recommended, 32x32. |

### Logo Constraints

- **Formats:** SVG (preferred), PNG, or `data:` URI
- **Max file size:** 16KB after base64 encoding — keeps the HTML output within the file size budget
- **Rendering:** The logo is inlined as an `<img>` with a `data:` URI or as inline `<svg>` in the HTML output

### Diagram Level Override

A diagram can override or suppress branding via a `branding` field in `diagram.json`:

```json
{
  "version": "1.0",
  "title": "Internal Draft",
  "branding": {
    "enabled": false
  }
}
```

Or selectively override:

```json
{
  "branding": {
    "footer": {
      "text": "Prepared for: Customer Inc."
    }
  }
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `branding.enabled` | boolean | `true` | Set to `false` to suppress all branding for this diagram |
| Other fields | — | — | Same structure as `shared/branding.json`; values merge with and override the project branding |

## Resolution Order

Branding follows the existing resolution cascade (see `workspace-structure.md`), inserted between built-in defaults and theme:

1. **Built-in defaults** — no branding
2. **Project branding** — `shared/branding.json`
3. **Diagram override** — `branding` field in `diagram.json`
4. **Project theme** — `shared/theme.json` (colors from branding feed into theme; explicit theme values win)

### Brand Colors and Theme Interaction

Brand `colors.primary` and `colors.secondary` are available to the theme system but do not automatically override theme colors. They influence:

- Header accent bar (if present)
- Footer text color
- Flow control panel active state

If `shared/theme.json` explicitly sets colors, those win. Brand colors are defaults for identity-related UI elements, not overrides for diagram content.

## Rendering

### Header

When branding is active and a logo or organization name is present, the HTML header is extended:

```html
<header id="diagram-header">
  <div class="brand">
    <img src="data:image/svg+xml;base64,..." alt="Acme Corp logo" height="28">
  </div>
  <h1>{{diagram.title}}</h1>
  <p>{{diagram.description}}</p>
</header>
```

If `placement` is `"corner"`, the logo appears as a fixed-position element in the top-right corner of the diagram canvas instead.

### Footer

When footer text is present:

```html
<footer id="diagram-footer">
  <span>Acme Corp — Architecture Team</span>
  <span>Generated 2026-08-21</span>
</footer>
```

The footer sits below the diagram container. Minimal styling: small text, muted color, no border.

### Favicon

When a favicon is provided, it's inlined in `<head>`:

```html
<link rel="icon" href="data:image/png;base64,...">
```

## Standalone Mode

In standalone mode (no `project.json`), branding can be specified directly in `diagram.json` under the `branding` field using the same structure as `shared/branding.json`. Logo `src` paths are relative to the diagram root.

## Agent Interaction

When the agent detects branding in a project:
- Include branding metadata in the diagram data JSON passed to the renderer
- Mention the organization name when confirming outputs ("Generated the Acme Corp order processing diagram")

When the user asks to set up branding:
- Create `shared/branding.json` with the specified values
- If a logo file is provided, copy it to `shared/` and reference it
- Confirm what will appear in rendered output

## Future Extensions

Not in this spec, but natural next steps:
- **Watermark** — semi-transparent logo or text overlaid on the diagram canvas
- **Multi-brand profiles** — switch between brand sets (e.g., internal vs. customer-facing)
- **Custom icon sets** — brand-specific node icons
- **Cover page** — title slide mode for presentations
