# Open Source Distribution

## What

AI Arch Story is a public, Apache License 2.0 open-source project. GitHub at
`https://github.com/RichNasz/ai-arch-story` is its canonical source repository.

The initial public baseline establishes licensing, discoverability, and
continuous verification. It does not claim that a container image has been
published or that the project has a supported release channel.

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

The workflow must not publish a container image, create a GitHub Release, or
add registry/download/release badges. It may use GitHub-hosted runners and
public upstream actions necessary to perform those checks.

## Deferred Container Publication

The distribution container remains a local-image artifact until a separate
release specification defines all of the following:

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
