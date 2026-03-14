# Contributing to Aether

Thank you for your interest in contributing! This document provides guidelines for contributing to the project.

## Quick Start

1. **Clone the repository**

   ```bash
   git clone <repository-url>
   cd ib_box_spread_full_universal
   ```

2. **Set up development environment**

   ```bash
   # Configure CMake with Ninja
   cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug \
     -DIBAPI_INCLUDE_DIR=~/IBJts/source/cppclient \
     -DIBAPI_LIB=~/IBJts/source/cppclient/libTwsApiCpp.dylib
   ```

3. **Build and test**

   ```bash
   ninja -C build
   ctest --test-dir build --output-on-failure
   ```

## Code Style

We follow these conventions (see [AGENTS.md](AGENTS.md) for complete guidelines):

| Element | Convention | Example |
|---------|------------|---------|
| Indentation | 2 spaces | |
| Braces | Allman style | `if (x)\n{` |
| Functions | `snake_case` | `calculate_spread()` |
| Types | `PascalCase` | `BoxSpread` |
| Constants | `k` prefix | `kMaxPositions` |
| Standard | C++20 | |

## Pull Request Process

1. **Create a feature branch**

   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make changes and test**

   ```bash
   # Run linters
   ./scripts/run_linters.sh

   # Run tests
   ctest --test-dir build --output-on-failure
   ```

3. **Commit with descriptive messages**
   - Use imperative mood: "Add feature" not "Added feature"
   - Keep subject line under 72 characters
   - Include context in body when needed

4. **Push and create PR**
   - Reference any related issues
   - Include test output if applicable
   - Note any configuration changes

## Testing Requirements

- **All trading logic** must have corresponding tests
- **Risk calculations** require test coverage
- Test files mirror source files: `foo.cpp` → `foo_test.cpp`
- Run tests locally before pushing

## Security Guidelines

- **Never** commit credentials, API keys, or secrets
- **Never** log sensitive information
- Use paper trading port (7497) for testing
- Gate live trading behind configuration flags

## Project Structure

```
native/
├── src/           # C++ source files
├── include/       # Header files
├── tests/         # Test files
│   └── python/    # Python binding tests
docs/              # Documentation
scripts/           # Build and utility scripts
.cursor/           # Cursor IDE configuration
```

The active Python surface in this repo is limited to native binding tests under `native/tests/python/`
and selected helper scripts. There is no top-level `python/` application directory anymore.

## Getting Help

- Check `docs/` for detailed documentation
- See `docs/API_DOCUMENTATION_INDEX.md` for API reference
- Review `.cursor/rules/` for AI-assisted development guidelines
- Open an issue for questions or problems

## Code of Conduct

Be respectful and constructive. We're all here to build great trading software.

---

For complete project guidelines, see [AGENTS.md](AGENTS.md).
