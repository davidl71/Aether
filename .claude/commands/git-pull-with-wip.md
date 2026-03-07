---
description: Safely pull when you have uncommitted local changes
---

Pull with uncommitted changes safely:

```bash
git stash --include-untracked
git pull
git stash pop
```

If stash pop has conflicts, resolve them manually then run `git stash drop`.

Do NOT run raw `git pull` with a dirty working tree — it will fail under rebase mode.
