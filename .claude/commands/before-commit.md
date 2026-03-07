---
description: Run pre-commit checklist before committing
---

Run through the pre-commit checklist:

1. **Format check**: `find native/src native/include -name '*.cpp' -o -name '*.h' | xargs clang-format --dry-run --Werror`
2. **Lint**: `./scripts/run_linters.sh`
3. **Tests**: `ctest --test-dir build --output-on-failure`
4. **Secrets**: Scan staged files for credentials, API keys, tokens — never commit these
5. **Docs**: If behavior or APIs changed, check if docs need updating (run `exarp-go check_documentation_health_tool` with workingDirectory = project root)
6. **Proto**: C++ proto is generated at build by CMake — only run `./proto/generate.sh` if Python/Go/TypeScript codegen is needed

Report pass/fail for each step. List any issues that must be fixed before committing.
