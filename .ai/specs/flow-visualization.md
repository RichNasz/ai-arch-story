# Flow Visualization

## What

The visual language for animating data and work flows through architecture diagrams. Defines what flow animations look like, how they behave, and how users control them.

## Why

Flow visualization is the defining feature that sets these diagrams apart from static architecture diagrams. A well-executed flow animation tells the story of how a system works — not just what the components are, but how data and work move between them. This spec ensures flows are beautiful, informative, and not distracting.

## Animation Types

Three animation styles, selectable per flow in the diagram definition:

### 1. Particle (`"animation": "particle"`)

Small glowing dots travel along edge paths from source to target.

- Particles are small circles (4–6px radius) with a subtle glow filter
- Multiple particles in sequence with staggered timing create a "stream" effect
- Particle color matches the flow's `style.color`
- At each node along the path, particles briefly pause (100ms) with a subtle pulse to show processing
- Particle trail: a fading afterglow along the last ~20% of the path traveled

**Best for:** Showing discrete items moving through a system (requests, messages, events)

### 2. Pulse (`"animation": "pulse"`)

The entire edge path illuminates in sequence, step by step.

- Each step's edge brightens from source to target over the step duration
- The edge color transitions from base to the flow's `style.color` and back
- A "wave" effect travels along the path — the leading edge is brightest
- After the wave passes, the edge retains a subtle tint of the flow color
- Steps animate in sequence: step 1 completes, then step 2 begins

**Best for:** Showing sequential processing, request/response patterns, pipelines

### 3. Highlight (`"animation": "highlight"`)

Static visual emphasis — no motion animation. Edges and nodes in the flow are highlighted with the flow's color.

- All edges in the flow path are colored with the flow's `style.color`
- All nodes touched by the flow get a colored border/ring
- No animation frames — this is the "print-friendly" flow style
- When multiple flows use highlight, each shows its distinct color simultaneously

**Best for:** Static contexts, printed output, simple relationship emphasis

## Animation Timing

### Speed Settings

| Speed | Step Duration | Particle Interval | Use Case |
|-------|-------------|-------------------|----------|
| `slow` | 3000ms | 1500ms between particles | Presentations, walkthrough |
| `normal` | 1500ms | 800ms between particles | Default viewing |
| `fast` | 700ms | 350ms between particles | Quick overview, many flows |

### Looping

- Flows loop continuously by default
- A 2-second pause between loop iterations prevents visual fatigue
- When a flow is toggled on, it begins from the first step (not mid-animation)

### Multiple Flows

When multiple flows are active simultaneously:

- Each flow animates independently on its own timing
- Flows that share edges: the edge takes the color of whichever flow is currently animating through it, with a smooth blend transition
- Flow legend clearly distinguishes flows by color and label
- Users can solo a single flow to see it in isolation

## Flow Controls UI — Narration Stepper

A control panel in the bottom-right corner that lets viewers step through a flow's story:

```
┌──────────────────────────────────────┐
│  Flows                               │
│  [● New Order] [● Order Prep]        │
├──────────────────────────────────────┤
│  Customer places a coffee order...   │
│                                      │
│  ① Place order                       │
│     Customer submits their order...  │
│  ② Create order              (active)│
│     API Gateway authenticates...     │
│  ③ Persist order                     │
│  ④ Order confirmed                   │
│  ⑤ Charge card                       │
│  ⑥ OrderCreated event                │
├──────────────────────────────────────┤
│  ⏮ ◀ ▶ ▶   2 / 6     Delay [2s ▾]  │
└──────────────────────────────────────┘
```

### Structure

- **Header** with "Flows" label
- **Flow tabs** — one tab per flow with color swatch and label; click to switch flows
- **Description** — the flow's top-level description (optional)
- **Step list** — numbered steps with label and optional description; clickable to jump directly
- **Transport bar** — reset (⏮), previous (◀), play/pause (▶/⏸), next (▶), step counter, delay selector

### Modes

- **Manual stepping** — use previous/next buttons or click any step to jump directly
- **Autoplay** — press ▶ to advance through steps automatically; press ⏸ to pause
- **Switching** — can switch between autoplay and manual freely at any time; clicking a step during autoplay pauses and jumps to that step

### Delay Configuration

A dropdown selector with presets: 1s, 2s (default), 3s, 5s. Changing delay during autoplay takes effect on the next step.

### Visual Feedback

- Active step is highlighted with a border and bolder text
- Completed steps are dimmed
- All edges up to and including the current step are highlighted with the flow's color
- A particle animates along the current step's edge when stepping

### Panel Behavior

- Semi-transparent with backdrop blur, doesn't obscure the diagram
- Panel position is fixed relative to the viewport (not the SVG), stays accessible during pan/zoom
- Collapses on small viewports (< 800px)

### Display Modes

The panel supports three display modes, switchable via icon buttons in the header (right-aligned next to "Flows"). Mode state is stored in the `data-fc-mode` attribute on `#flow-controls`.

#### Floating (default)

- Panel floats in the bottom-right corner, draggable to any position
- Semi-transparent with backdrop blur and rounded corners
- `data-fc-mode="floating"`

#### Docked

- Panel anchors to the right edge of the viewport as a full-height sidebar
- Left side retains rounded corners (`border-radius: 12px 0 0 12px`); right side is flush
- Drag is disabled; step list scrolls vertically within the panel body
- Overlays the diagram — does not push or resize it; user can pan to see obscured content
- `data-fc-mode="docked"`

#### Minimized

- Panel collapses to a small circular button (40px) in the bottom-right corner (`#fc-restore-btn`)
- Clicking the restore button returns to the previous mode (floating or docked)
- `data-fc-mode="minimized"` on the panel; restore button toggles `.visible` class

#### Mode Toggle UI

Two buttons in the panel header, right-aligned:
- **Dock/Float toggle** — switches between docked and floating modes (icon changes to reflect available action)
- **Minimize** — collapses the panel to the restore button

## SVG Implementation

### Particle Animation

Particles move along edge paths using SVG `<animateMotion>` or JS-driven `requestAnimationFrame` positioning along the pre-computed path.

```svg
<!-- In <defs> -->
<filter id="glow">
  <feGaussianBlur stdDeviation="2" result="blur" />
  <feMerge>
    <feMergeNode in="blur" />
    <feMergeNode in="SourceGraphic" />
  </feMerge>
</filter>

<!-- In layer-flow -->
<circle class="flow-particle" r="5" fill="{flow.color}" filter="url(#glow)">
  <animateMotion dur="{stepDuration}" path="{edgePath}" fill="freeze" />
</circle>
```

### Pulse Animation

Edge paths are duplicated in the flow layer with a clip-path or stroke-dashoffset animation that reveals the colored path progressively.

### Node Processing Pulse

When a particle or pulse wave arrives at a node, the node briefly shows a colored ring:

```css
.node-pulse {
  animation: node-arrival 300ms ease-out;
}
@keyframes node-arrival {
  0% { stroke-width: 2; stroke-opacity: 0.8; }
  100% { stroke-width: 6; stroke-opacity: 0; }
}
```

## Diverging and Converging Flows

Flows can model patterns beyond simple linear paths:

### Fan-out (one step triggers multiple next steps)

When a flow step ends at a node that is the `from` for multiple subsequent steps, those steps animate simultaneously:

```json
"steps": [
  { "edge": "api-to-db", "label": "Persist" },
  { "edge": "api-to-queue", "label": "Publish event", "parallel": true },
  { "edge": "api-to-cache", "label": "Invalidate cache", "parallel": true }
]
```

Steps marked `"parallel": true` begin at the same time as the previous step, rather than sequentially. This visually shows a service doing multiple things after receiving input.

### Fan-in (multiple steps converge on one node)

The next sequential step after a parallel group waits until all parallel animations complete before beginning.

## Accessibility

- Flow animations respect `prefers-reduced-motion` — when enabled, all flows fall back to `highlight` mode
- Flow colors meet WCAG AA contrast against the diagram background
- The flow legend is keyboard-navigable
- Tooltips on flow steps are screen-reader accessible via `aria-label`
