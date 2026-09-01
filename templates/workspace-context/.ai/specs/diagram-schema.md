# Diagram Schema

Each `diagram.json` uses version `1.0` and contains a title plus nodes, edges,
flows, and groups as needed. Use stable, unique kebab-case IDs. Edges reference
existing node IDs; flow steps reference edge IDs and must form a traversable
path; group membership references existing nodes or groups.

Use semantic node types such as `service`, `datastore`, `queue`, `gateway`,
`frontend`, `user`, `external`, `function`, and `storage`. Omit optional style
and position fields unless the user asks for them. Preserve untouched elements
when editing an existing diagram.
