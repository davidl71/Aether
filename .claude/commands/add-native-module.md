---
description: Add a new native C++ module (source, header, tests)
argument-hint: "<module name, e.g. convexity_calculator>"
---

Follow these steps to add a new native C++ module named `$ARGUMENTS`:

1. Create `native/src/$ARGUMENTS.cpp` and `native/include/$ARGUMENTS.h`
   - Use sub-dirs (`brokers/`, `strategies/`) if appropriate
2. Add source to `SOURCES` list and header to `HEADERS` list in `native/CMakeLists.txt`
3. Create `native/tests/test_$ARGUMENTS.cpp` using Catch2 v3
4. Register the test in `native/tests/CMakeLists.txt`
5. Follow project style: C++20, 2-space indent, Allman braces, `snake_case` functions, `PascalCase` types, `k` prefix constants
6. Build and verify: `ninja -C build && ctest --test-dir build -R test_$ARGUMENTS --output-on-failure`

Note: C++ proto is generated at build by CMake — no need to run `./proto/generate.sh` for C++.
