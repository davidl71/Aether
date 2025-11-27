# GitHub Copilot Instructions

This repository follows the comprehensive guidelines in [AGENTS.md](../AGENTS.md).

## Quick Reference

### Code Style
- **C++20** standard, **2-space indentation**, Allman braces
- **snake_case** for functions/variables, **PascalCase** for types
- Constants prefixed with `k` (e.g., `kMaxPositions`)

### Project Structure
- Core C++ logic: `native/src/` with headers in `native/include/`
- Python integration: `python/`
- Tests: `native/tests/` (mirror source file names)
- Documentation: `docs/`

### Build Commands
```bash
# Configure
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug

# Build
ninja -C build

# Test
ctest --test-dir build --output-on-failure
```

### Key Documentation
- **API Reference**: `docs/API_DOCUMENTATION_INDEX.md`
- **Architecture**: `docs/research/architecture/`
- **MCP Tools**: `.cursor/mcp.json`

## Important Conventions

1. **Trading Logic**: Always include tests for pricing calculations
2. **Security**: Never commit credentials; use paper trading port (7497)
3. **Dependencies**: Point to IB API at `~/IBJts/source/cppclient`

For complete guidelines, see [AGENTS.md](../AGENTS.md).
