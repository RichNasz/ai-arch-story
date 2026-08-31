# Development workflow

Keep one feature objective active at a time. Work on a branch, run the focused
tests, then run the relevant full test/build commands before a merge.

Before committing, merging, or publishing, run:

```bash
git status --short
git diff --cached --name-only
git diff --check
bash scripts/check-repository-hygiene.sh
```

Do not commit generated diagram `output/`, `.DS_Store`, local agent artifacts,
or secrets. Before a public merge or image publication, record the branch,
commit, changed-path list, test/build evidence, CI state, and any action that
still requires approval.
