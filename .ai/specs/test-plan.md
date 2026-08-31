# Test Plan

## What

A validation plan using a set of thematically related diagrams that exercise all capabilities of the system. These diagrams form a project (thematic collection) and are used to prove out each spec as it is implemented.

## Why

Abstract specs need concrete validation. A single test diagram can't cover grouping, shared components, multiple flows, or cross-diagram consistency. A thematically related set exercises the full workspace structure, shared assets, and a realistic range of architectural patterns.

## Test Project: "CloudBrew" — Cloud-Native Coffee Ordering Platform

A fictional cloud-native platform for a coffee chain. The theme is familiar enough that anyone can understand the architecture, but complex enough to exercise every feature of the system.

The project covers: microservices, event-driven architecture, external integrations, data stores of multiple types, user-facing frontends, background workers, and infrastructure boundaries.

### Project Root

```
test/cloudbrew/
├── project.json
├── shared/
│   ├── theme.json
│   ├── components.json
│   └── glossary.json
├── diagrams/
│   ├── system-overview/
│   ├── order-flow/
│   ├── inventory-sync/
│   ├── infrastructure/
│   └── payment-processing/
```

## Test Diagrams

### 1. System Overview

**File:** `diagrams/system-overview/diagram.json`

**Purpose:** High-level view of all major components and their relationships. Tests:

- Many node types (frontend, service, datastore, queue, external, gateway)
- Groups (backend services, data tier, external systems)
- Simple edges showing structural relationships (no animated flows)
- Shared component reuse from `shared/components.json`
- Adaptive layout with 15+ nodes across screen sizes

**Architectural elements:**
- Mobile app and web app (frontend)
- API gateway (gateway)
- Order service, menu service, inventory service, notification service (services)
- Orders DB, menu DB, inventory DB (datastores)
- Event bus (queue)
- Payment provider, loyalty program (external)
- CDN (storage)

---

### 2. Order Flow

**File:** `diagrams/order-flow/diagram.json`

**Purpose:** Detailed view of the order lifecycle with multiple animated flows. Tests:

- **Multiple flows on the same diagram** with distinct colors and animations
- Flow steps that diverge (one event triggers multiple consumers)
- Two-way edges (request/response patterns)
- Step labels that override edge labels during animation
- Flow legend/controls in rendered output

**Flows:**
1. **Happy path order** (green) — Customer places order → API gateway → order service → persist to DB → publish OrderCreated event → notification service sends confirmation
2. **Order preparation** (blue) — OrderCreated event → kitchen display service → order status updates back to customer via WebSocket
3. **Order failure** (red) — Payment decline → order service rolls back → publish OrderFailed event → notification service sends failure notice

---

### 3. Inventory Sync

**File:** `diagrams/inventory-sync/diagram.json`

**Purpose:** Background async architecture showing event-driven inventory management. Tests:

- Event-driven patterns (pub/sub, event sourcing)
- Multiple queues/topics
- External system integration (supplier API)
- Scheduled/cron-triggered flows
- Nodes with rich metadata (SLAs, throughput numbers)

**Flows:**
1. **Real-time stock update** — POS sale → inventory event → inventory service → update DB → check threshold → reorder event
2. **Supplier sync** — Scheduled job → fetch from supplier API → reconcile with inventory DB → publish discrepancy events

---

### 4. Infrastructure

**File:** `diagrams/infrastructure/diagram.json`

**Purpose:** Infrastructure-level view showing deployment topology. Tests:

- **Nested groups** (cloud region → VPC → subnet → cluster → namespace)
- Nodes representing infrastructure (load balancers, DNS, container instances)
- Edges representing network paths rather than data flow
- Group styling (different colors/borders for public vs. private subnets)
- Dense layout with many nested boundaries

**Groups:**
- AWS us-east-1
  - Public subnet (ALB, NAT gateway)
  - Private subnet
    - EKS cluster
      - Orders namespace (order pods, order DB sidecar)
      - Menu namespace (menu pods)
      - Platform namespace (gateway, observability)
  - Data subnet (RDS, ElastiCache, MSK)

---

### 5. Payment Processing

**File:** `diagrams/payment-processing/diagram.json`

**Purpose:** Security-sensitive flow with compliance boundaries. Tests:

- **Compliance/security boundary groups** (PCI scope, encrypted transit)
- Detailed flow with step-level annotations
- External system callouts (payment gateway, fraud detection, bank)
- Conditional flow paths (fraud check pass/fail)
- Minimal nodes but high detail per node (metadata-heavy)

**Flows:**
1. **Successful payment** — Tokenize card → fraud check → authorize with bank → capture → record transaction → emit PaymentCompleted
2. **Fraud rejection** — Tokenize card → fraud check fails → reject → emit PaymentRejected → alert ops

## Shared Assets Validation

### shared/components.json

Components reused across multiple diagrams:

| Component | Used In |
|-----------|---------|
| API Gateway | system-overview, order-flow, payment-processing |
| Order Service | system-overview, order-flow, inventory-sync |
| Orders DB | system-overview, order-flow, infrastructure |
| Event Bus | system-overview, order-flow, inventory-sync |
| Notification Service | system-overview, order-flow |

Validates that shared components resolve correctly and that diagram-level overrides work (e.g., order-flow may add style overrides to highlight the order service).

### shared/theme.json

A custom "CloudBrew" theme with brand colors. Validates that project-level theming applies consistently across all five diagrams.

### shared/glossary.json

Terms like "OrderCreated event", "PCI scope", "stock threshold" that the agent uses for consistent descriptions and metadata across diagrams.

## Validation Criteria

Each spec is validated against the test diagrams as it's implemented:

| Spec | Validated By | What To Check |
|------|-------------|---------------|
| `diagram-schema.md` | All diagrams | Schema validates all five definitions without error |
| `workspace-structure.md` | Project structure | project.json, shared/, diagrams/ resolve correctly; standalone mode works with any single diagram extracted |
| `rendering-engine.md` | All diagrams | All node types, edge types, groups, and nested groups render correctly |
| `flow-visualization.md` | order-flow, inventory-sync, payment-processing | Animations run, multiple flows distinguish visually, flow controls work |
| `visual-design.md` | All diagrams | Theme applies consistently, node types have distinct shapes, readable at all sizes |
| `export-format.md` | All diagrams | Each HTML file is fully self-contained, opens offline, renders identically |
| `agent-skill.md` | All diagrams | Agent can generate each diagram from a natural language description |
| `tech-stack.md` | Build process | Rust toolchain builds all five diagrams from JSON to HTML |

## Incremental Validation

As specs and implementation progress, diagrams are built in this order (each adds complexity):

1. **System overview** — proves basic rendering (nodes, edges, groups)
2. **Infrastructure** — proves nested groups and dense layout
3. **Order flow** — proves flow animation and multiple flows
4. **Inventory sync** — proves event-driven patterns and scheduled flows
5. **Payment processing** — proves compliance boundaries and conditional flows

Each diagram must pass before moving to the next. Issues found feed back into spec updates.
