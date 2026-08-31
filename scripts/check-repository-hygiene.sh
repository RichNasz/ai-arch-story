#!/usr/bin/env bash
set -euo pipefail

if git ls-files | rg -n '(^|/)(\.superpowers/|\.worktrees/|\.DS_Store$|output/)'; then
  echo 'Tracked local-agent, platform, or generated artifacts are prohibited.' >&2
  exit 1
fi

git diff --check HEAD^!
