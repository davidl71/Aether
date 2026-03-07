---
description: Review a file for financial correctness, trading safety, and style
argument-hint: "<file path>"
---

READ $ARGUMENTS

Review this file for:
1. **Financial correctness** — pricing formulas, APR/yield calculations, day-count conventions, decimal precision (no `float` for monetary values), rounding
2. **Trading safety** — position limits enforced before order submission, dry-run/paper gating, order validation, no hardcoded credentials
3. **C++20 style** — 2-space indent, Allman braces, `PascalCase` types, `snake_case` functions, `k` prefix constants
4. **Test coverage gaps** — missing edge cases, untested branches
5. **Security** — credentials, sensitive logging, input validation

Organize findings as: Critical Issues → Important Improvements → Suggestions → Positives.
