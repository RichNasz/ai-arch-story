# Public Release Remediation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the public AI Arch Story repository safe, provenance-clean, and contract-accurate before future releases.

**Architecture:** Centralize diagram-name validation and atomic persistence in the Rust server; remove corporate fixture material; constrain the container build context; then bring the API implementation to its published contract. Treat public-history rewrite as an independently approved publication operation after the normal remediation passes.

**Tech Stack:** Rust/Axum, filesystem atomic rename, multipart HTTP, GitHub Actions, Podman, Git filter-repo.

**Spec:** `.ai/specs/public-release-remediation.md`

## Global Constraints

- Valid diagram names are lowercase kebab-case slugs matching `^[a-z0-9]+(?:-[a-z0-9]+)*$`; invalid names return HTTP 400 `INVALID_DIAGRAM_NAME`.
- No unvalidated diagram name may reach a filesystem join, read, write, render, preview, or recursive delete.
- All diagram, branding, type, and theme JSON writes use one same-directory atomic writer.
- Public fixtures contain no Red Hat trademark, corporate metadata, employee address, or internal marker.
- No container image is published by this remediation.
- Rewriting public history is prohibited until its dedicated approval checkpoint.

---

### Task 1: Codify the remediation contract and test surface

**Files:**
- Modify: `.ai/specs/README.md`
- Create: `.ai/specs/public-release-remediation.md`
- Create: `src/server/routes_test.rs` or an equivalent focused server-test module

- [ ] Write failing tests for `../escape`, `/tmp/x`, `a/b`, `a\\b`, `.`, `..`, uppercase names, and a valid `system-overview` name.
- [ ] Exercise create, read, render, preview, and delete paths through the router; assert invalid names return 400 `INVALID_DIAGRAM_NAME` and the temporary workspace has no escaped files.
- [ ] Run the focused test module and record the expected failure before implementation.

### Task 2: Centralize safe diagram paths and atomic JSON persistence

**Files:**
- Modify: `src/server/state.rs`
- Modify: `src/server/routes.rs`
- Modify: `src/server/mod.rs` only if error wiring needs it
- Modify: `Cargo.toml` and `Cargo.lock` only for test/runtime dependencies required by the chosen atomic-write implementation
- Test: `src/server/routes_test.rs`

- [ ] Implement one parsed diagram-name type/resolver in `state.rs`; it validates the exact slug grammar before returning a diagram directory or `diagram.json` path.
- [ ] Route every diagram handler and output/preview path through that resolver; preserve valid route responses and map invalid-name errors to the defined 400 envelope.
- [ ] Implement one atomic JSON write helper used by `write_diagram`, branding writes, type writes, and theme writes: temp sibling → flush → rename; remove a temp file on pre-rename failure.
- [ ] Add red/green tests for no filesystem escape and atomic-write failure cleanup; run `cargo test --locked`.

### Task 3: Replace public fixture identity and prevent container-context leaks

**Files:**
- Replace: `test/cloudbrew/shared/logo.svg`
- Modify: `test/cloudbrew/shared/branding.json`
- Create: `.containerignore`
- Modify: `.gitignore` only if a public-source ignore rule is also needed
- Test: fixture rendering and container build commands

- [ ] Replace the logo with a minimal project-owned CloudBrew SVG containing no `<metadata>`, names, email addresses, internal markers, or Red Hat references.
- [ ] Change the branding organization, footer, colors, and logo alt text so the fixture is neutral and truthful.
- [ ] Add `.containerignore` entries for `.git`, `.github`, `.ai`, `.codex`, `.mcp.json`, `.superpowers`, `.env`, `.env.*`, `target`, `webapp/node_modules`, `webapp/dist`, `.DS_Store`, and `output`; retain all Containerfile COPY inputs.
- [ ] Verify with a context-inspection test or build log that excluded paths are unavailable to the build, then build the image and render the CloudBrew diagram.

### Task 4: Bring shared theme and SVG upload routes into the documented API contract

**Files:**
- Modify: `src/server/routes.rs`
- Modify: `src/server/mod.rs` and `Cargo.toml` only if multipart extraction requires it
- Modify: `webapp/src/api.ts` and `webapp/src/components/TypesTab.tsx` if the client currently sends the incompatible JSON shape payload
- Test: Rust server route tests and relevant webapp tests

- [ ] Write failing route tests for `GET`/`PUT /shared/theme` and multipart `POST /project/shapes` (valid SVG, invalid content, unsafe name).
- [ ] Add the theme routes using atomic JSON persistence.
- [ ] Implement multipart SVG upload with explicit name/file fields and strict SVG/name validation; update the web editor client to use `FormData` if necessary.
- [ ] Run focused Rust and web tests, then full `cargo test --locked` and `(cd webapp && npm test -- --run && npm run build)`.

### Task 5: Review, publish normal remediation, and gate history rewrite

**Files:**
- Modify: `.ai/specs/README.md` and remediation spec status only after every criterion is verified
- Modify: Git history only with the separate approval described below

- [ ] Run `git diff --check`, secret/metadata scans, `cargo test --locked`, web tests/build, and `podman build` plus CloudBrew render smoke test.
- [ ] Request an independent review of the remediation diff and resolve every Critical/Important finding.
- [ ] Push the normal remediation commit; confirm GitHub CI is green.
- [ ] **Approval checkpoint:** ask the repository owner to explicitly authorize rewriting public history and force-pushing `main`. Do not run `git filter-repo`, `git push --force-with-lease`, or tag rewrites before that response.
- [ ] After approval, create a local backup ref, use `git filter-repo --force --path test/cloudbrew/shared/logo.svg --path test/cloudbrew/diagrams/system-overview/output/system-overview.html --path test/cloudbrew/shared/.DS_Store --invert-paths`, verify those paths are absent from every reachable commit, then force-push with lease and verify remote `main` and Actions.
- [ ] If history rewrite is declined, leave the normal remediation commit in place; record that prior public history remains accessible and retain the spec as Draft.
