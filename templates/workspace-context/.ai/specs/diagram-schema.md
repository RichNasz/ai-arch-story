# Diagram Schema

Each `diagram.json` uses version `1.0` and contains a title plus nodes, edges,
flows, and groups as needed. Use stable, unique kebab-case IDs. Edges reference
existing node IDs; flow steps reference edge IDs and must form a traversable
path; group membership references existing nodes or groups.

Use this canonical complete flow shape:

```json
{
  "id": "flow-id",
  "label": "Flow label",
  "description": "Optional narrative",
  "steps": [
    { "edge": "edge-id", "label": null, "description": null, "parallel": null }
  ],
  "style": { "color": null, "speed": null, "animation": "pulse" },
  "metadata": {}
}
```

`steps` is an array of objects, not an array of edge-ID strings. Animation
configuration belongs inside `style`, not at the flow's top level. Never use
unknown or guessed fields; when a validator is unavailable, copy the object
shape from a known-good `diagram.json` produced by the same AI Arch Story
version.

Use semantic node types such as `service`, `datastore`, `queue`, `gateway`,
`frontend`, `user`, `external`, `function`, and `storage`. Omit optional style
and position fields unless the user asks for them. Preserve untouched elements
when editing an existing diagram.
