# Workspace Structure

`project.json` contains project metadata. `shared/` contains assets inherited
by diagrams, including optional theme, branding, type, component, glossary, and
SVG-shape files. Each diagram is stored in `diagrams/<kebab-case-name>/` with a
`diagram.json`, optional `assets/`, and generated `output/` HTML.

Read existing files before changing them. Reuse shared assets when they apply
to multiple diagrams; use diagram-local assets only when they are specific to
one diagram. Generated HTML is reproducible output, not the source of truth.
