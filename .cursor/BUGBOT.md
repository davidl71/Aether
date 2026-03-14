# Bugbot Review Rules for Aether

This file provides project-specific context for Cursor Bugbot when reviewing pull requests.

## Project Overview

Multi-asset synthetic financing platform with a C++20 core, pybind11-backed Python bindings/tests, Rust backend agents, and archived web surfaces. Box spreads are one strategy component, not the whole product.

## Critical Security Requirements

### Trading Software Safety

- **NEVER** commit credentials, API keys, or secrets
- **NEVER** log sensitive information (account numbers, positions, PII)
- **ALWAYS** use paper trading port (7497) for testing
- **GATE** live trading behind explicit configuration flags
- **VALIDATE** all configuration before use
- **REQUIRE** explicit confirmation for any live trading operations

### Code Security Checks

- Scan for hardcoded credentials or API keys
- Verify no sensitive data in logs or error messages
- Check that authentication tokens are never committed
- Ensure environment variables are used for secrets
- Validate input sanitization for all user inputs

## Code Style Requirements

### C++ Code Style

- **Standard**: C++20 (ISO C++20)
- **Indentation**: 2 spaces (not tabs)
- **Braces**: Allman style for multi-line scopes
- **Line length**: 100 character soft wrap
- **Naming**:
  - Types: `PascalCase` (e.g., `Scenario`, `OrderManager`)
  - Functions: `snake_case` (e.g., `make_scenario`, `calculate_profit`)
  - Variables: `snake_case`
  - Constants: `k` prefix (e.g., `kMaxPositions`)
- **Comments**: Add `//` comments only where trading math is non-obvious

### File Organization

- Core logic: `native/src/` with headers in `native/include/`
- Tests: `native/tests/` mirroring source file names
- Helper scripts: Top-level `scripts/`
- Generated output: `build/`, `protobuf-build/` (disposable)

## Build System Requirements

### CMake Configuration

- Use CMake presets: `macos-universal-debug`, `macos-universal-release`
- Dependencies must be properly configured:
  - TWS API: `native/third_party/tws-api/IBJts/source/cppclient/client/`
  - Intel Decimal: `native/third_party/IntelRDFPMathLib20U2/LIBRARY/libbid.a`
  - Protocol Buffers: System-installed
  - Abseil: System-installed

### Build Commands

```bash
cmake --preset macos-universal-debug
cmake --build --preset macos-universal-debug
./scripts/build_universal.sh
./scripts/run_linters.sh
ctest --output-on-failure
```

## Testing Requirements

- **All tests must pass** before merging
- Tests mirror source file names (use Catch2 framework)
- Run: `ctest --output-on-failure`
- New features require corresponding tests
- Critical trading logic requires comprehensive test coverage

## Static Analysis

- Run linters before committing: `./scripts/run_linters.sh`
- Tools: cppcheck, Clang Static Analyzer, Infer, clang-tidy
- Add annotations for critical functions: `[[nodiscard]]`, `__attribute__((nonnull))`
- See `docs/STATIC_ANALYSIS_ANNOTATIONS.md` for details

## Common Issues to Flag

### Security Issues

- Hardcoded credentials or API keys
- Sensitive data in logs
- Missing input validation
- Unsafe memory operations (buffer overflows, use-after-free)
- Missing error handling for critical operations

### Code Quality Issues

- Inconsistent indentation (must be 2 spaces)
- Incorrect brace style (must be Allman for multi-line)
- Line length exceeding 100 characters
- Missing or incorrect naming conventions
- Missing comments for non-obvious trading math

### Trading Logic Issues

- Missing validation for financial calculations
- Incorrect decimal precision handling
- Missing error handling for API failures
- Race conditions in concurrent operations
- Missing bounds checking for market data

### Build System Issues

- Missing CMake configuration
- Incorrect dependency paths
- Missing test targets
- Build artifacts committed to repository

## Pre-Commit Checklist

Before approving a PR, verify:

1. ✅ All tests pass (`ctest --output-on-failure`)
2. ✅ Linters pass (`./scripts/run_linters.sh`)
3. ✅ Build succeeds (`cmake --build --preset macos-universal-debug`)
4. ✅ No credentials or secrets committed
5. ✅ Code follows style guidelines (2-space indent, Allman braces)
6. ✅ Documentation updated if needed
7. ✅ Static analysis annotations added for critical functions

## Documentation Requirements

- Update `docs/API_DOCUMENTATION_INDEX.md` for API changes
- Add comments for non-obvious trading math
- Update README if project structure changes
- Document new configuration options

## Multi-Language Considerations

This project includes:

- **C++**: Core trading logic (`native/src/`)
- **Python**: pybind11 bindings and binding tests (`native/src/box_spread_pybind.cpp`, `native/tests/python/`)
- **Rust**: Backend services (`agents/backend/`)
- **Go**: Agents (`agents/go/`)
- **TypeScript**: Web interface (`web/`)

When reviewing:

- Verify language-specific conventions are followed
- Check that cross-language interfaces are properly defined
- Ensure build system handles all languages correctly

## Git Workflow

- Use `scripts/setup_worktree.sh` for new worktrees
- Commit messages: Imperative mood, 72-character subject lines
- Never commit build artifacts, logs, or credentials
- PRs should be focused and atomic

## References

- `.cursorrules` - Main project rules
- `docs/STATIC_ANALYSIS_ANNOTATIONS.md` - Static analysis guidelines
- `docs/API_DOCUMENTATION_INDEX.md` - API documentation
- `docs/research/integration/CURSOR_SETUP.md` - Cursor configuration
