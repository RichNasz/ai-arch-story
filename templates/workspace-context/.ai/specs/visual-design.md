# Visual Design

The renderer supplies presentation-ready defaults. Prefer semantic node types
and restrained styling over manual positioning or broad restyling. Use explicit
style overrides only for user-requested emphasis, branding, or accessibility.

Project-wide visual choices belong in `shared/theme.json`; diagram-specific
choices belong in its `diagram.json`. Keep related diagrams visually
consistent by reusing project-level settings.
