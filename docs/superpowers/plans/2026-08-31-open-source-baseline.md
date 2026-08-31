# Open Source Baseline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish AI Arch Story as an Apache-2.0 licensed GitHub project with CI verification and no published container image.

**Architecture:** Add repository-level legal and CI artifacts only; leave application code unchanged. CI verifies the Rust, web-editor, and container builds but has read-only GitHub permissions and no registry credentials.

**Tech Stack:** Apache License 2.0, GitHub Actions, Rust stable, Node.js 24/npm, Podman.

**Spec:** `.ai/specs/open-source-distribution.md`

## Global Constraints

- GitHub at `https://github.com/RichNasz/ai-arch-story` is canonical; `main` is its default development branch.
- `LICENSE` must contain the unmodified Apache License, Version 2.0.
- CI runs for pull requests and pushes to `main`, but does not publish images or create releases.
- Documentation must not claim a published image, registry tag, or release channel.
- Do not mass-edit existing source files merely to add license headers.

---

### Task 1: Establish the Apache-2.0 legal baseline

**Files:**
- Create: `LICENSE`
- Modify: `README.md: Documentation and Participation section`

**Interfaces:**
- Consumes: canonical Apache License 2.0 text and the approved distribution spec.
- Produces: root legal terms and accurate reader links.

- [ ] **Step 1: Verify the legal artifact is absent**

Run: `test ! -e LICENSE && rg -n "Apache|github.com/RichNasz/ai-arch-story|Issues" README.md`

Expected: `LICENSE` is absent, and the README makes no inaccurate licensing or tracker claim.

- [ ] **Step 2: Create the legal artifact**

Create `LICENSE` using the complete canonical Apache License, Version 2.0, January 2004 text, including the complete Appendix. Do not add custom terms or a shortened notice.

- [ ] **Step 3: Add accurate participation links**

Append the following section to `README.md`:

```markdown
## License and participation

AI Arch Story is open source under the [Apache License 2.0](LICENSE).
Report bugs and feature requests through the [GitHub issue tracker](https://github.com/RichNasz/ai-arch-story/issues).
```

- [ ] **Step 4: Verify the legal artifact and links**

Run: `rg -n "Apache License|Version 2.0, January 2004|http://www.apache.org/licenses/" LICENSE && rg -n "\[Apache License 2.0\]\(LICENSE\)|https://github.com/RichNasz/ai-arch-story/issues" README.md`

Expected: the license has its canonical identifiers and the README has both intended links.

- [ ] **Step 5: Commit the legal baseline**

Run:

```bash
git add LICENSE README.md
git commit -m "docs: add Apache-2.0 licensing"
```

### Task 2: Add non-publishing continuous integration

**Files:**
- Create: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: `Cargo.lock`, `webapp/package-lock.json`, and `Containerfile`.
- Produces: GitHub Actions jobs named `rust`, `webapp`, and `container`.

- [ ] **Step 1: Verify no workflow conflicts**

Run: `test ! -e .github/workflows/ci.yml && find .github/workflows -type f -maxdepth 1 -print 2>/dev/null`

Expected: no `ci.yml` or existing workflow needs to be merged.

- [ ] **Step 2: Create the CI workflow**

Create `.github/workflows/ci.yml` with this exact content:

```yaml
name: CI

on:
  pull_request:
  push:
    branches: [main]

permissions:
  contents: read

jobs:
  rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --locked

  webapp:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: webapp
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 24
          cache: npm
          cache-dependency-path: webapp/package-lock.json
      - run: npm ci
      - run: npm run build
      - run: npm test -- --run

  container:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: podman build -t ai-arch-story:ci .
```

Do not add package write permission, login actions, push commands, `ghcr.io`, release commands, or credentials.

- [ ] **Step 3: Run static workflow safety checks**

Run: `rg -n "pull_request:|branches: \[main\]|contents: read|cargo test --locked|npm ci|npm run build|npm test -- --run|podman build -t ai-arch-story:ci" .github/workflows/ci.yml && ! rg -n "packages: write|login-action|podman push|docker push|gh release|ghcr.io" .github/workflows/ci.yml`

Expected: each required verification string is present and no publishing expression matches.

- [ ] **Step 4: Commit CI**

Run:

```bash
git add .github/workflows/ci.yml
git commit -m "ci: verify application and container build"
```

### Task 3: Commit the existing project baseline

**Files:**
- Add: the existing source-controlled project files, including `.ai/`, `.codex/`, `.mcp.json`, project instructions, application source, tests, templates, documentation, and web editor files.

**Interfaces:**
- Consumes: the project files already present in the working tree, plus the Task 1 and 2 commits.
- Produces: one reviewable initial-project commit with no build output or dependency directory.

- [ ] **Step 1: Check the baseline does not contain secrets or generated output**

Run:

```bash
find . -path ./.git -prune -o -path ./target -prune -o -path ./webapp/node_modules -prune -o -type f -print | sort
test ! -e webapp/dist
```

Expected: the source tree is enumerated without `target/`, `webapp/node_modules/`, or `webapp/dist/` content.

- [ ] **Step 2: Stage only the intentional project baseline**

Run:

```bash
git add .ai .codex .gitignore .mcp.json AGENTS.md CLAUDE.md Cargo.lock Cargo.toml Containerfile docs src templates test webapp
git status --short
```

Expected: all project-owned source, instructions, specs, tests, and documentation are staged; ignored build output and dependencies are absent.

- [ ] **Step 3: Inspect staged content and commit it**

Run:

```bash
git diff --cached --check
git diff --cached --stat
git commit -m "feat: add AI Arch Story"
```

Expected: no whitespace errors; the commit includes the intended project baseline.

### Task 4: Verify and publish the baseline

**Files:**
- Modify: Git metadata only (`master` becomes `main`; `origin` targets GitHub)

**Interfaces:**
- Consumes: commits from Tasks 1–3.
- Produces: an upstream `main` branch that triggers CI.

- [ ] **Step 1: Run local verification**

Run:

```bash
cargo test --locked
(cd webapp && npm ci && npm run build && npm test -- --run)
podman build -t ai-arch-story:ci .
```

Expected: each command exits with status zero.

- [ ] **Step 2: Inspect the publication state**

Run: `git status --short && git log --oneline --decorate --max-count=5 && git diff --check HEAD`

Expected: no uncommitted files and no whitespace errors.

- [ ] **Step 3: Configure the canonical branch and remote**

Run:

```bash
git branch -M main
git remote add origin https://github.com/RichNasz/ai-arch-story.git
git remote -v
```

Expected: `origin` fetch and push point at the supplied repository.

- [ ] **Step 4: Push to GitHub and inspect CI**

Run: `git push -u origin main`

Expected: GitHub accepts `main -> main`; then inspect `https://github.com/RichNasz/ai-arch-story/actions` for the CI run.
