#!/usr/bin/env bash
# Stash uncommitted changes (if any), pull, then pop stash.
# Use when you run "git pull" often and have local WIP.
set -e

REPO_ROOT="${1:-.}"
cd "$REPO_ROOT"

STASHED=0
if [ -n "$(git status --porcelain -u)" ]; then
  echo "[git-pull-safe] Uncommitted changes detected; stashing..."
  git stash push -u -m "git-pull-safe $(date +%Y-%m-%dT%H:%M:%S)"
  STASHED=1
fi

if ! git pull; then
  echo "[git-pull-safe] Pull failed. Restoring stash if we stashed."
  if [ "$STASHED" -eq 1 ]; then git stash pop; fi
  exit 1
fi

if [ "$STASHED" -eq 1 ]; then
  echo "[git-pull-safe] Restoring stashed changes..."
  if ! git stash pop; then
    echo "[git-pull-safe] Stash pop had conflicts. Resolve, then run: git stash drop"
    exit 1
  fi
fi

echo "[git-pull-safe] Done."
