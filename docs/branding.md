# Branding Diagrams

AI Arch Story branding adds your organization's identity to diagrams without
changing their structural content. It is optional: a workspace with no branding
continues to render normally. When present, brand assets are embedded in the
exported HTML, so shared diagrams do not depend on an external logo or favicon
URL.

## Choose Where to Apply Branding

Use project branding when most diagrams in a workspace share the same identity.
It lives in `shared/branding.json` and is inherited by every diagram. Use a
diagram-level override when one diagram needs a different footer or other
exception. Set `branding.enabled` to `false` when a diagram must omit all
inherited branding.

For interactive work, start the local service, open the web editor, and use its
**Branding** tab. A coding agent can also set up or update project branding
through the local API. The JSON below is the version-controlled workspace
representation; it is useful when reviewing branding alongside your diagrams.

## Set Project Branding

Place assets in the workspace's `shared/` directory, then create or update
`shared/branding.json`. Asset paths in this file are relative to `shared/`.

```json
{
  "organization": "Acme Corp",
  "logo": {
    "src": "logo.svg",
    "alt": "Acme Corp logo",
    "placement": "header",
    "height": 28
  },
  "colors": {
    "primary": "#EE0000",
    "secondary": "#2B2B2B"
  },
  "footer": {
    "text": "Acme Corp — Architecture Team",
    "showGeneratedDate": true
  },
  "favicon": {
    "src": "favicon.png"
  }
}
```

All fields are optional. An absent or empty `shared/branding.json` applies no
branding.

| Field | What it controls |
| --- | --- |
| `organization` | Organization name. It is used in the header when no logo is configured and as the default footer text when no footer text is set. |
| `logo.src` | Logo path relative to `shared/`, or an inline `data:` URI. SVG is preferred; PNG is also supported. |
| `logo.alt` | Accessible logo description. It defaults to `organization`, but write meaningful alternative text whenever possible. |
| `logo.placement` | `header` (the default) places the logo in the diagram header; `corner` places it in the top-right diagram canvas corner. |
| `logo.height` | Logo height in pixels; defaults to 24 and preserves the logo's aspect ratio. |
| `colors.primary` | Identity color used for the header accent and active UI elements. |
| `colors.secondary` | Secondary identity color available for group borders or edge accents. |
| `footer.text` | Footer attribution. It defaults to the organization name when omitted. |
| `footer.showGeneratedDate` | Adds a generation timestamp to the footer; defaults to `false`. |
| `favicon.src` | Favicon path relative to `shared/`, or an inline `data:` URI. A 32x32 PNG is recommended. |

Keep a logo to 16KB or less after base64 encoding so it does not materially
increase the exported HTML size.

## Override or Disable Branding for One Diagram

Add a `branding` object to the affected diagram's `diagram.json`. Values merge
with project branding, so include only the values that differ.

For example, change only the footer for a customer-facing export:

```json
{
  "branding": {
    "footer": {
      "text": "Prepared for: Customer Inc."
    }
  }
}
```

To suppress all project branding for an internal draft:

```json
{
  "branding": {
    "enabled": false
  }
}
```

For a standalone diagram with no `project.json`, put the full branding object
directly in `diagram.json`. Its logo and favicon paths are then relative to the
diagram directory instead of `shared/`.

## Branding, Themes, and Rendered Output

Branding answers “whose diagram is this?”; themes control the diagram's visual
system. Values resolve in this order:

1. Built-in defaults (no branding)
2. Project branding in `shared/branding.json`
3. Diagram-level `branding` override
4. Explicit project theme values in `shared/theme.json`

Explicit theme colors take precedence over brand colors for diagram content.
Brand colors are identity defaults: they can affect the header accent, footer
text, and active flow controls, but do not automatically recolor nodes, edges,
or the entire diagram.

During export, logo and favicon assets are inlined. Active branding can extend
the header with an organization name or logo, add a footer attribution and
optional date, and add the favicon to the document head. A `corner` logo is
positioned in the diagram canvas rather than the header.

## Verify Your Branding

Render a diagram after a branding change and open the resulting HTML file in a
browser. Confirm that the logo and favicon load, alternative text is meaningful,
the footer text is appropriate for the audience, and explicit theme colors still
have the intended precedence.

## Related References

- [Branding specification](../.ai/specs/branding.md)
- [Diagram schema](../.ai/specs/diagram-schema.md)
- [Workspace structure](../.ai/specs/workspace-structure.md)
- [Web editor](../.ai/specs/web-editor.md)
- [Web API](../.ai/specs/web-api.md)
- [Container usage reference](container-usage.md)
