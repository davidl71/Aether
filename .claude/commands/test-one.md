---
description: Run a single C++ test by name pattern
argument-hint: "<test name pattern, e.g. test_risk_calculator>"
---

Run a specific test by name pattern:

```bash
ctest --test-dir build -R $ARGUMENTS --output-on-failure -V
```

Report full test output including any assertion failures.
