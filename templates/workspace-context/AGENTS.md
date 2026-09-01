# AI Arch Story Workspace Instructions

This directory is a user-owned architecture-diagram workspace. It is not the
AI Arch Story source repository: do not modify the tool, its container image,
or its implementation from here.

Read `.ai/specs/README.md` before creating or changing workspace content.
Follow the local specs for the diagram schema, shared assets, visual design,
flows, custom types, and the agent workflow.

Work only with `project.json`, `shared/`, and `diagrams/`. Preserve existing
user intent. When the AI Arch Story service is running, use its API for diagram
operations so the agent and web editor work on the same validated files.
