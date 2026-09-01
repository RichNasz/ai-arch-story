# Custom Types and Shapes

Use built-in semantic node types unless a repeated domain concept needs a
consistent visual treatment. Project-wide custom types belong in
`shared/types.json`; diagram-only types belong in the relevant `diagram.json`.
Project SVG shapes belong in `shared/shapes/`; diagram-specific shapes belong
in `diagrams/<name>/assets/shapes/`.

Prefer a project-level type or shape when multiple diagrams share it. Keep
custom type keys stable and descriptive.
