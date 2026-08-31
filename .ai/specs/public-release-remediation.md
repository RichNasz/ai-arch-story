# Public Release Remediation

## What

This specification defines the remediation required after AI Arch Story's
initial public publication. It eliminates path traversal in diagram APIs,
removes non-project-owned branding material, keeps local material out of
container build contexts, reconciles documented API behavior with the server,
and provides a controlled option to rewrite public Git history.

## Security Boundary

`{name}` in every `/api/v1/diagrams/{name}` route is a diagram slug, not a
filesystem path. A valid slug matches:

```text
^[a-z0-9]+(?:-[a-z0-9]+)*$
```

The service rejects any empty value, uppercase character, slash, backslash,
dot, percent-decoded traversal component, or absolute path with HTTP 400 and
`INVALID_DIAGRAM_NAME`. Every operation that reads, writes, renders, previews,
or deletes a diagram obtains its path through the same validated resolver.
No handler independently joins an unvalidated diagram name to a workspace path.

All persisted JSON writes use one same-directory atomic writer: serialize,
write a uniquely named temporary sibling with restrictive permissions, flush it,
then rename it over the destination. Temporary files are removed when an error
occurs before rename. This applies to diagram, branding, and project-type JSON
writes.

## Public Assets and Provenance

The public CloudBrew fixture must not contain Red Hat logos, corporate
trademarks, employee addresses, internal audience labels, asset identifiers, or
other non-project metadata. Replace `shared/logo.svg` with a small,
project-owned, metadata-free CloudBrew SVG and update `shared/branding.json` to
refer only to that neutral identity. The example remains a functional branding
fixture but makes no affiliation claim.

## Build Context

The repository contains a root `.containerignore` used by Podman/Docker build
contexts. It excludes VCS metadata, agent/MCP local configuration, local
environment files, generated output, dependency/build directories, OS metadata,
and test artifacts not copied by the Containerfile. It must not exclude files
that the Containerfile copies: `Cargo.toml`, `Cargo.lock`, `src/`,
`templates/`, `webapp/package.json`, `webapp/package-lock.json`, or `webapp/`
source needed by `npm run build`.

## API Contract Alignment

The server implements the authoritative `web-api.md` contract before that spec
returns to **Done**:

- add `GET` and `PUT` `/shared/theme` using the same validated JSON persistence
  rules as branding;
- accept `multipart/form-data` at `POST /project/shapes`, with a shape name and
  SVG file part, reject non-SVG or unsafe names, and retain list/delete behavior;
- test those routes along with rejection of unsafe diagram slugs and atomic
  write failure behavior.

If an endpoint cannot meet the contract, change `web-api.md` first and state
the supported contract precisely; do not leave an implementation/spec mismatch.

## Publication and History

The remediation is delivered in two phases:

1. A normal, reviewable remediation branch removes unsafe current-tree content,
   applies code/tests, and runs all local and GitHub CI checks.
2. Only with explicit owner approval, rewrite public history to remove the
   original metadata-bearing logo and generated artifacts from every reachable
   commit, force-push with lease, and verify GitHub's default branch and Actions
   afterward.

History rewriting is destructive and does not control clones, forks, caches, or
third-party copies. Before the force-push, retain an immutable local backup ref,
verify the exact paths to remove, and notify contributors to re-clone or reset.
If metadata is sensitive beyond ordinary public-source cleanup, contact GitHub
Support for cached-object guidance after rewrite.

## Completion Criteria

- Path traversal requests never create, read, render, preview, or delete outside
  the workspace; regression tests demonstrate rejection.
- All JSON mutation paths use the common atomic writer and tests cover its
  failure cleanup.
- CloudBrew's committed branding assets contain no Red Hat/internal metadata or
  affiliation.
- A container build context inspection proves excluded local files are absent
  while the production image still builds and renders the CloudBrew example.
- API routes and `web-api.md` agree, with route-level tests.
- The normal remediation commit passes Rust, web editor, and container CI.
- If separately approved, rewritten Git history contains none of the explicitly
  removed asset or generated-output paths and the remote `main` branch is
  verified after the force-push.
