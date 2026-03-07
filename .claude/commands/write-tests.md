---
description: Generate Catch2 tests for a C++ source file
argument-hint: "<path to source file, e.g. native/src/risk_calculator.cpp>"
---

Read the source file and generate Catch2 v3 tests for it.

READ $ARGUMENTS
READ native/tests/test_risk_calculator.cpp

Write Catch2 v3 unit tests following these conventions:
- File: `native/tests/test_<module>.cpp`
- Use `TEST_CASE` with `[module_name]` tags, `SECTION` for sub-scenarios
- Test edge cases: zero, negative, boundary values
- Test trading-specific cases: invalid strikes, expired contracts, zero notional
- Use `Catch::Matchers::WithinAbs` for floating-point comparisons
- No network or TWS connection required

After writing, register the new test file in `native/tests/CMakeLists.txt`.
