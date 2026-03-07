---
description: Run all pre-push checks (format, lint, test, build)
---

Run the full pre-push validation:

```bash
find native/src native/include -name '*.cpp' -o -name '*.h' | xargs clang-format --dry-run --Werror
./scripts/run_linters.sh
ctest --test-dir build --output-on-failure
ninja -C build
```

Run in order; stop and report on first failure. Summarize what passed/failed and what to fix before pushing.
