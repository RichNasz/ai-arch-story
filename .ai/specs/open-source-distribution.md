# Open Source Distribution

## What

AI Arch Story is a public, Apache License 2.0 open-source project. GitHub at
`https://github.com/RichNasz/ai-arch-story` is its canonical source repository.

The active-development baseline establishes licensing, discoverability,
continuous verification, and a development container image.

## Why

Prospective users and contributors need clear legal terms and a trustworthy
public source. The repository needs automated verification so a future
published image is based on a known-good build, without making an unversioned
image tag the project's release contract.

## Repository Baseline

- The default development branch is `main`.
- The root `LICENSE` file contains the unmodified Apache License, Version 2.0.
- The README identifies the project as Apache-2.0 licensed and links to
  `LICENSE`.
- The README may link users to the GitHub issue tracker once the repository is
  configured as the authoritative place to report issues.
- Existing source files are not mass-edited solely to add license headers.
  Contributors add the appropriate Apache-2.0 header to newly created
  project-owned source files when the language and project conventions support
  one.

## Continuous Integration

A GitHub Actions workflow runs on pull requests and pushes to `main`. It must:

1. run the Rust test suite;
2. install the web editor dependencies deterministically and run its build and
   test suite; and
3. build the `Containerfile` as a container-image verification step.
4. run repository-hygiene verification before other jobs.

On successful pushes to `main`, the workflow publishes the development image
`ghcr.io/richnasz/ai-arch-story:main`. It uses GitHub's ephemeral token with
only `contents: read` and `packages: write` permissions. Pull-request runs
verify builds but never publish an image, create a GitHub Release, or add
registry/download/release badges.

Repository hygiene fails if tracked paths include `.superpowers/`, `.worktrees/`,
`.DS_Store`, or generated `output/` content. It also runs `git diff --check`
against the triggering commit.

## Deferred Container Publication

`main` is a mutable active-development tag, not a release or compatibility
promise. A separate release specification is still required before adding
immutable version tags, `latest`, or GitHub Releases. That specification must
define:

- registry identity and public/private access policy;
- semantic-versioning and tag policy, including the meaning of `latest`;
- release trigger and changelog/release-note process;
- image signing, provenance, and vulnerability-response expectations; and
- supported platforms and compatibility policy.

After that decision, `container-modes.md`, `project-documentation.md`, and the
container-usage documentation must be updated together with the publishing
workflow. Until then, user documentation uses a locally built image name only.

## Verification

Before a baseline commit is published, verify that the license file exists,
README links resolve locally, the CI workflow syntax is valid by inspection,
and the repository has the configured GitHub remote. GitHub Actions execution
is verified after push through the repository's Actions page.
