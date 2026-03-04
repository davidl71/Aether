# Skill: Pull with uncommitted changes

**When:** User wants to `git pull` but has local uncommitted changes.

**Do:**

1. Run the project command **`git:pull-safe`** (or `./scripts/git_pull_safe.sh`).
2. This stashes changes (including untracked), pulls, then pops the stash.
3. If stash pop has conflicts, tell the user to resolve and run `git stash drop` when done.

**Do not:** Run raw `git pull` when the working tree is dirty; it will fail under rebase.

**Reference:** scripts/git_pull_safe.sh, .cursor/commands.json → `git:pull-safe`.
