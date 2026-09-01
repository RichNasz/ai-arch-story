# Agent Workflow

Before editing, read `AGENTS.md`, this context index, `project.json`, and the
existing diagram or shared asset involved. Confirm the user's requested scope;
preserve unrelated content. For a new diagram, use a kebab-case directory name
under `diagrams/` and create a valid `diagram.json`.

When `ai-arch-story serve` is running, use its HTTP API to read, validate,
write, and render diagram content. The browser editor and agent then observe
the same disk-backed workspace. Report changes briefly and identify the output
HTML after rendering.
