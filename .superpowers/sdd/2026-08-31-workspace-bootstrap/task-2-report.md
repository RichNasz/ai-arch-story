# Task 2: `start` CLI confirmation and safe mutation

## Scope

Added `ai-arch-story start [--workspace <path>] [--name <name>] [--yes]`.
It resolves and displays the target workspace and project name, supports
confirmation, editing, and cancellation, and writes only the missing standard
workspace items after confirmation. Existing `project.json` is validated and
never replaced; valid workspaces are no-ops. The command prints:

```text
Next: ai-arch-story serve --workspace <resolved-path>
```

`serve` and documentation were intentionally left unchanged for Task 3.

## TDD evidence

The bootstrap tests were first run with the new `run_start` API absent:

```text
cargo test --locked bootstrap::tests
```

It failed with an unresolved `run_start` import, as expected. The CLI parsing
test was also first run without a `Start` command and failed because that
variant did not exist.

Focused tests then covered cancellation, `--yes` initialization, partial
repair without overwrite, invalid `project.json` refusal, interactive
workspace/name edits, and CLI argument parsing.

## Verification

```text
git diff --check
cargo test --locked
```

Result: 35 passed, 0 failed.
