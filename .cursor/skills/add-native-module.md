# Skill: Add a new native C++ module

**When:** User wants to add a new C++ source file and tests to the native core.

**Steps:**

1. Create `native/src/<name>.cpp` and `native/include/<name>.h` (or under `native/include/brokers/` or `native/include/strategies/` if appropriate).
2. Add the source to the `SOURCES` list and the header to the `HEADERS` list in `native/CMakeLists.txt`.
3. Create `native/tests/test_<name>.cpp` using Catch2. Register the test in `native/tests/CMakeLists.txt`.
4. Follow project style: C++20, 2-space indent, Allman braces, `snake_case` functions, `PascalCase` types, `k` prefix for constants (see AGENTS.md / CLAUDE.md).
5. Build and test: `ninja -C build` (or preset), or `build:ai-friendly` for quiet + JSON. Run `ctest --test-dir build --output-on-failure`.

**Proto:** C++ from `proto/messages.proto` is generated at build by CMake; no need to run `./proto/generate.sh` for C++.

**Reference:** CLAUDE.md "Adding a new source file", AGENTS.md "Key Source Files", native/CMakeLists.txt.
