# Web Editor UX Corrections Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the web editor's preview, diagram lifecycle, element forms, custom types, branding, and status feedback conform to `.ai/specs/web-editor.md`.

**Architecture:** Keep the existing React/PatternFly editor and Axum API. Add narrowly scoped UI helpers and API endpoints, use server rendering as the preview source, and model type origin explicitly so mutations affect only the selected scope.

**Tech Stack:** React 19, PatternFly 6, TypeScript, Vitest/Testing Library, Rust, Axum.

**Spec:** `.ai/specs/web-editor.md`

## Global Constraints

- All reads and writes go through the HTTP API.
- Preview output uses the existing renderer and must match exported HTML.
- Nodes and edges trigger re-layout; other saved changes refresh rendered output.
- Existing diagram schema fields and JSON naming remain backward compatible.

---

### Task 1: Regression Test Foundation and Preview State

**Files:**
- Modify: `webapp/package.json`
- Modify: `webapp/vite.config.ts`
- Create: `webapp/src/test/setup.ts`
- Create: `webapp/src/components/PreviewPane.test.tsx`
- Modify: `webapp/src/App.tsx`
- Modify: `webapp/src/components/PreviewPane.tsx`

- [ ] Add a test proving existing rendered output opens immediately and diagram changes reset preview state.
- [ ] Run the focused test and confirm it fails because `hasOutput` is ignored.
- [ ] Pass output availability into `PreviewPane`, key refresh state by diagram, and expose loading/error states.
- [ ] Run the focused test and confirm it passes.

### Task 2: Diagram Lifecycle and Status Feedback

**Files:**
- Create: `webapp/src/components/CreateDiagramModal.tsx`
- Create: `webapp/src/components/StatusBar.tsx`
- Modify: `webapp/src/App.tsx`
- Modify: `webapp/src/api.ts`
- Test: `webapp/src/App.test.tsx`

- [ ] Add failing tests for creating a diagram, render/re-layout feedback, and last-saved status.
- [ ] Implement the create modal, explicit Re-layout action, output link, validation state, and save timestamp.
- [ ] Run the focused tests and confirm they pass.

### Task 3: Complete Element Editing Forms

**Files:**
- Create: `webapp/src/components/StyleFields.tsx`
- Create: `webapp/src/components/MetadataFields.tsx`
- Modify: `webapp/src/components/NodesTab.tsx`
- Modify: `webapp/src/components/EdgesTab.tsx`
- Modify: `webapp/src/components/FlowsTab.tsx`
- Modify: `webapp/src/components/GroupsTab.tsx`
- Test: `webapp/src/components/EditorForms.test.tsx`

- [ ] Add failing tests for style/metadata persistence, flow descriptions/parallel/reordering, and nested groups.
- [ ] Implement reusable style and metadata fields and the missing flow/group controls.
- [ ] Run the focused tests and confirm they pass.

### Task 4: Type Scope and Shape Workflows

**Files:**
- Modify: `src/schema/types.rs`
- Modify: `src/server/routes.rs`
- Modify: `webapp/src/types.ts`
- Modify: `webapp/src/api.ts`
- Modify: `webapp/src/components/TypesTab.tsx`
- Test: `src/server/routes.rs`
- Test: `webapp/src/components/TypesTab.test.tsx`

- [ ] Add failing tests proving diagram-scoped types are returned with origin and edited only in that scope.
- [ ] Add a diagram-aware resolved-types endpoint and origin metadata without changing render serialization.
- [ ] Implement scoped edit/delete, reset-to-default, imported-shape listing/upload, and a visual shape preview.
- [ ] Run Rust and web focused tests and confirm they pass.

### Task 5: Branding Editor

**Files:**
- Modify: `src/server/routes.rs`
- Modify: `webapp/src/api.ts`
- Create: `webapp/src/components/BrandingTab.tsx`
- Modify: `webapp/src/components/EditorSidebar.tsx`
- Test: `src/server/routes.rs`
- Test: `webapp/src/components/BrandingTab.test.tsx`

- [ ] Add failing API and component tests for reading/saving branding and converting uploaded assets to data URIs.
- [ ] Implement shared branding GET/PUT endpoints and the Branding tab fields specified by the editor spec.
- [ ] Run focused tests and confirm they pass.

### Task 6: Full Verification

**Files:** No production changes expected.

- [ ] Run `cargo test`.
- [ ] Run `npm test -- --run` and `npm run build` in `webapp/`.
- [ ] Start the server against CloudBrew and verify startup preview, create modal, all tabs, save refresh, and status feedback in the browser.
- [ ] Inspect browser console errors and confirm none remain.
