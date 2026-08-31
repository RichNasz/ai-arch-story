# Task 1: Workspace model and red tests

## Scope

Added `src/bootstrap.rs` with a shared, read-only workspace inspection model:

- `ProjectMetadata` serializes the required `name` and `version: "1.0"` fields.
- `WorkspaceStatus` reports a valid workspace, an itemized repair plan, or invalid project metadata.
- `default_project_name` derives a title-cased directory name.

No `start` CLI interaction or `serve` behavior changed.

## TDD evidence

Initial red command:

```text
cargo test --locked bootstrap
```

It failed at compilation with unresolved imports for the new API (`default_project_name`,
`inspect_workspace`, `ProjectMetadata`, `WorkspaceItem`, and `WorkspaceStatus`), which was
expected before the implementation existed.

Focused green command:

```text
cargo test --locked bootstrap
```

Result: 6 passed, 0 failed.

Full verification command:

```text
cargo test --locked
```

Result: 29 passed, 0 failed.

## Formatting

`rustfmt --check` across the whole crate identifies pre-existing formatting differences in
unrelated source files. `src/bootstrap.rs` was formatted directly; no unrelated formatting
changes are included in this task.
