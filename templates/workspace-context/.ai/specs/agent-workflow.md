# Agent Workflow

Before editing, read `AGENTS.md`, this context index, `project.json`, and the
existing diagram or shared asset involved. Confirm the user's requested scope;
preserve unrelated content. For a new diagram, use a kebab-case directory name
under `diagrams/` and create a valid `diagram.json`.

When `ai-arch-story serve` is running, use its HTTP API to read, validate,
write, and render diagram content. The browser editor and agent then observe
the same disk-backed workspace. An agent must not report a diagram created or
updated successfully until `POST /api/v1/diagrams/{name}/validate` and then
`POST /api/v1/diagrams/{name}/render` succeed, and the render response names
the output HTML.

Without the service, run `ai-arch-story render diagrams/<name>/diagram.json`
and confirm the output HTML exists. JSON parsing and custom reference checks
alone are not enough. Never use unknown or guessed fields; if no validator is
available, copy the required object shape from a known-good `diagram.json`
produced by the same AI Arch Story version.
