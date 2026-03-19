# Git Worktree Setup

Use Git’s built-in worktree support to create additional working trees.

## Create a worktree

```bash
# From the repository root
git worktree add ../my-feature main
cd ../my-feature
```

Optional: create a new branch in the worktree:

```bash
git worktree add -b feature/xyz ../my-feature main
```

## Remove a worktree

```bash
git worktree remove ../my-feature
# or
git worktree remove my-feature  # by worktree path name
```

## List worktrees

```bash
git worktree list
```

## Note

The previous `setup_worktree.sh` script (which also built native/TWS dependencies) was removed when the native C++ build was retired. Use `git worktree add` as above; build and test via the Rust/agents stack (e.g. `cargo build` in `agents/backend`).
