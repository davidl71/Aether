---
description: Auto-fix all fixable code quality issues (clang-format + web lint)
---

Auto-fix formatting and lint issues:

```bash
find native/src native/include -name '*.cpp' -o -name '*.h' | xargs clang-format -i
cd web && npm run lint:fix 2>/dev/null || true
cd web && npm run lint:css:fix 2>/dev/null || true
```

Report what was auto-fixed. Note: some issues require manual fixes — list those separately.
