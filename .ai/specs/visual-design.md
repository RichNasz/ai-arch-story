# Visual Design

## What

The aesthetic system governing how diagrams look — colors, typography, shapes, spacing, and themes. Defines the defaults that make diagrams visually stunning without requiring the user to specify any styling.

## Why

The system must produce beautiful diagrams out of the box. Users describe architecture conversationally — they don't pick colors or fonts. The visual design system makes every default choice so the output is presentation-ready. Custom themes (project-level or diagram-level) override these defaults.

## Design Philosophy

- **Clean and modern** — Flat design with subtle depth cues (soft shadows, gentle gradients)
- **Information hierarchy** — The eye should flow from groups → nodes → edges → labels → metadata
- **Restrained color** — A muted palette with selective color for emphasis (flows, node types)
- **Whitespace is structure** — Generous padding and margins make dense diagrams breathable
- **Dark and light** — Both modes are first-class, not an afterthought

## Color System

### Default Theme Palette

#### Light Mode

| Role | Color | Usage |
|------|-------|-------|
| Background | `#FAFAFA` | Diagram canvas |
| Surface | `#FFFFFF` | Node fill |
| Border | `#E2E8F0` | Node and group borders |
| Text Primary | `#1E293B` | Node labels |
| Text Secondary | `#64748B` | Edge labels, metadata |
| Group Fill | `#F8FAFC` | Group background (with opacity) |
| Group Border | `#CBD5E1` | Group boundary |
| Edge | `#94A3B8` | Default edge color |

#### Dark Mode

| Role | Color | Usage |
|------|-------|-------|
| Background | `#0F172A` | Diagram canvas |
| Surface | `#1E293B` | Node fill |
| Border | `#334155` | Node and group borders |
| Text Primary | `#F1F5F9` | Node labels |
| Text Secondary | `#94A3B8` | Edge labels, metadata |
| Group Fill | `#1E293B` | Group background (with opacity) |
| Group Border | `#475569` | Group boundary |
| Edge | `#64748B` | Default edge color |

Mode is set by `"theme"` in the diagram definition. Default: `"default"` (light). Options: `"default"`, `"dark"`, `"auto"` (follows `prefers-color-scheme`).

### Node Type Colors

Each semantic node type has a distinct accent color used for a subtle left border or top accent bar:

| Node Type | Accent Color | Rationale |
|-----------|-------------|-----------|
| `service` | `#3B82F6` (blue) | Workhorse of architecture — calm, neutral |
| `datastore` | `#8B5CF6` (purple) | Distinct from services, suggests persistence |
| `queue` | `#F59E0B` (amber) | Warmth suggests activity, messages in motion |
| `gateway` | `#06B6D4` (cyan) | Entry point, cool and inviting |
| `frontend` | `#10B981` (emerald) | User-facing, fresh and approachable |
| `external` | `#6B7280` (gray) | Deliberately muted — outside our control |
| `function` | `#EC4899` (pink) | Small, punchy — stands out for serverless |
| `user` | `#14B8A6` (teal) | Human element, warm but distinct |
| `storage` | `#A78BFA` (light purple) | Related to datastore family |
| `generic` | `#94A3B8` (slate) | No semantic meaning, neutral |

### Flow Colors

Flows use a distinct palette from node accents to avoid confusion:

| Flow Index | Color | Name |
|-----------|-------|------|
| 1st flow | `#10B981` | Emerald |
| 2nd flow | `#3B82F6` | Blue |
| 3rd flow | `#EF4444` | Red |
| 4th flow | `#F59E0B` | Amber |
| 5th flow | `#8B5CF6` | Purple |

Auto-assigned by index unless the flow definition specifies `style.color`.

## Typography

### Font Stack

```css
font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
```

System fonts — no web fonts to load, instant rendering, native feel on every OS.

### Scale

| Element | Size (at reference viewport) | Weight |
|---------|-----|--------|
| Diagram title | 24px | 600 |
| Group label | 14px | 600 |
| Node label | 13px | 500 |
| Edge label | 11px | 400 |
| Tooltip text | 12px | 400 |
| Flow legend | 12px | 400 |

All sizes scale proportionally with the SVG viewBox.

## Node Styling

### Shape Construction

Nodes have:
- A filled shape (type-specific, see `rendering-engine.md`)
- A subtle box shadow via SVG `<filter>` (2px blur, 10% opacity black offset)
- A left accent bar (3px) in the node type's accent color
- Rounded corners where applicable (8px radius)
- Interior padding: 12px horizontal, 8px vertical

### Node States

| State | Visual Change |
|-------|--------------|
| Default | Standard styling |
| Hover | Border brightens, subtle scale (1.02), shadow deepens |
| Active (clicked) | Accent border all sides, connected edges highlight |
| Flow active | Colored ring pulse (see `flow-visualization.md`) |

## Edge Styling

- Stroke width: 1.5px default
- Color: `#94A3B8` (light mode), `#64748B` (dark mode)
- Arrowhead: 8px, filled, matches edge color
- Bezier curves with smooth entry/exit angles
- Edge labels: centered on path, white background pill for readability

## Group Styling

- Background: theme's group fill color at 50% opacity
- Border: 1.5px, theme's group border color
- Border radius: 12px
- Label: positioned top-left inside boundary, semi-bold
- Nested groups: each depth level slightly adjusts opacity and border weight

| Depth | Background Opacity | Border Width |
|-------|-------------------|-------------|
| 0 | 50% | 1.5px |
| 1 | 40% | 1.2px |
| 2 | 30% | 1.0px |
| 3+ | 25% | 0.8px |

## Spacing and Layout Constants

These values guide the Rust layout engine:

| Constant | Value | Purpose |
|----------|-------|---------|
| Node padding (internal) | 12px h, 8px v | Space between label and node border |
| Node margin (external) | 40px | Minimum space between adjacent nodes |
| Group padding | 24px | Space between group boundary and contained nodes |
| Group label height | 28px | Reserved space for the group label above content |
| Edge clearance | 20px | Minimum distance between edge path and unrelated nodes |
| Nested group indent | 16px | Additional padding per nesting level |

## Theme Customization

Project-level `shared/theme.json` can override any default:

```json
{
  "mode": "light",
  "colors": {
    "background": "#FEFCE8",
    "surface": "#FFFBEB",
    "accent": {
      "service": "#B45309"
    }
  },
  "typography": {
    "fontFamily": "\"IBM Plex Sans\", sans-serif"
  },
  "spacing": {
    "nodeMargin": 50
  }
}
```

Only specified fields override defaults — everything else falls through to the built-in theme.
