# Workspace Bootstrap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add safe `start` workspace initialization and make `serve` reject invalid project workspaces.

**Architecture:** Extract a shared workspace inspection/initialization module used by both CLI commands. Keep `start` interactive but automation-safe through `--yes`; keep `serve` mutation-free and fail before binding.

**Tech Stack:** Rust, Clap, serde_json, std::fs, tokio/axum.

**Spec:** `.ai/specs/workspace-bootstrap.md`

## Global Constraints

- `start` only creates `project.json`, `shared/`, and `diagrams/`; it never invokes Git or creates `.gitignore`.
- `serve` validates and never repairs or creates workspace content.
- Invalid `project.json` is explained and never overwritten.
- `render` retains standalone-diagram behavior.

### Task 1: Workspace model and red tests

**Files:** Create `src/bootstrap.rs`; modify `src/main.rs`; test in `src/bootstrap.rs`.

- [ ] Write failing tests for empty initialization, title-cased directory default, name override, valid workspace no-op, partial repair plan, and invalid `project.json` refusal.
- [ ] Implement `WorkspaceStatus` inspection and project metadata serialization only after recording the red result.
- [ ] Run focused bootstrap tests, then `cargo test --locked`; commit `test: define workspace bootstrap behavior`.

### Task 2: `start` CLI confirmation and safe mutation

**Files:** Modify `src/main.rs`, `src/bootstrap.rs`; test `src/bootstrap.rs`.

- [ ] Add `start --workspace --name --yes`; default workspace to `.`, derive the directory display name, and print resolved values.
- [ ] For interactive mode, accept confirm, edit workspace/name, or quit before mutations; for non-empty/partial targets list only items to add or repair.
- [ ] Write only after confirmation, emit the exact next `serve --workspace` command, and test cancellation/`--yes`/non-overwrite behavior.
- [ ] Run full Rust tests and commit `feat: initialize project workspaces`.

### Task 3: Strict `serve` validation and documentation

**Files:** Modify `src/main.rs`, `src/server/mod.rs`, `README.md`, `docs/container-usage.md`, `.ai/specs/container-modes.md`; test CLI/workspace module.

- [ ] Validate before creating the Tokio runtime/listener; on failure list every issue and print the exact `start --workspace` repair command with non-zero exit.
- [ ] Add tests proving invalid workspaces prevent server start and valid initialized workspaces proceed to server setup.
- [ ] Document `start` as one-time initialization and `serve` as ongoing development; retain `render` standalone language.
- [ ] Run `cargo test --locked`, web tests/build, and container build; commit `feat: require initialized workspaces for serve`.
