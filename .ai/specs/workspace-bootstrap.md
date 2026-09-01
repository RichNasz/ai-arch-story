# Workspace Bootstrap

## What

`ai-arch-story start` initializes or repairs an AI Arch Story project workspace
and then exits. `ai-arch-story serve` runs only against a valid project
workspace; it never initializes or repairs one.

## Commands

```text
ai-arch-story start [--workspace <path>] [--name <name>] [--yes]
ai-arch-story serve [--workspace <path>] [--port <port>] [--host <host>] [--static-dir <path>]
```

`start` defaults `--workspace` to the current directory. Its default project
name is the title-cased basename of that resolved directory. `--workspace` and
`--name` override either default. The command prints both resolved values and,
unless `--yes` is provided, lets the user confirm, edit either value, or quit
before any filesystem mutation.

## Valid Project Workspace

A valid workspace contains:

```text
<workspace>/
  AGENTS.md
  .ai/specs/
    README.md
    workspace-purpose.md
    workspace-structure.md
    diagram-schema.md
    visual-design.md
    flow-visualization.md
    custom-types.md
    agent-workflow.md
  project.json
  shared/
  diagrams/
```

`project.json` must parse as project metadata and contain a non-empty `name`
and schema `version` of `1.0`. `shared`, `diagrams`, `.ai`, and `.ai/specs`
must be directories. Each required agent-context file must be a non-empty
regular file. Bootstrap writes the missing standard workspace items, including
the agent-context bundle. It does not create theme, components, glossary, type,
asset, Git, or `.gitignore` files.

The context bundle is self-contained guidance for agents working in this
user-owned workspace. It does not make the workspace a checkout of AI Arch
Story and must not contain project build, release, or contributor instructions.

## Start Behavior

- In an empty directory, create the valid workspace after confirmation and
  exit with the exact suggested `serve` command.
- In a non-empty directory with no workspace metadata, warn that existing files
  will not be changed, list the missing standard workspace items, and offer continue
  or quit.
- In a partial workspace, list only missing or invalid standard items and offer
  to repair them. Existing `project.json` and other user files are never
  overwritten. If `project.json` is invalid, explain the validation error and
  quit rather than replacing it.
- In a valid workspace, make no change and exit successfully with guidance to
  run `serve`.
- Existing non-empty `AGENTS.md` and context specs are user-owned and are never
  overwritten. Missing files are safely added during repair. If `.ai` or
  `.ai/specs` is an existing non-directory path, bootstrap fails before writing
  any workspace item.
- `--yes` accepts the derived or provided values but does not bypass invalid
  `project.json` protection.

## Serve Behavior

Before opening a listener, `serve` validates the workspace using the same
validator. If invalid, it exits non-zero, lists each missing/invalid item, and
prints a `start --workspace <resolved-path>` repair command. It creates no
directories or files. `render` keeps standalone `diagram.json` support and is
not subject to this project-workspace requirement.

## Git and Safety

Bootstrap does not invoke Git, create `.git`, create `.gitignore`, overwrite
existing content, or make network calls. All writes occur only after the user
confirms or passes `--yes`.

## Verification

Tests cover empty initialization, creation of non-empty agent guidance,
defaults and overrides, cancellation, non-empty directory confirmation,
partial-workspace repair, preservation of customized guidance, empty guidance
rejection, invalid `project.json` refusal, directory/file conflicts,
idempotent valid workspaces, `--yes`, and `serve` failure before it binds a
port. CLI help documents the distinction between `start` and `serve`.
