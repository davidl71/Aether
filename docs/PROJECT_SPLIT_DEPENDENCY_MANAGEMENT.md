# Project Split - Dependency Management Strategy

**Date**: 2025-11-22
**Task**: T-199
**Status**: ✅ Complete
**Purpose**: Establish how split projects will reference each other and manage dependencies

---

## Executive Summary

This document defines the **dependency management mechanism** for split projects. It covers:

1. **Package Manager Strategy** - How projects reference each other
2. **Package Registry Setup** - Accounts and publishing workflows
3. **Semantic Versioning** - Version numbering strategy
4. **Dependency Documentation** - How to document dependencies
5. **Version Compatibility Matrix** - Compatibility tracking

---

## Package Manager Strategy

### Decision: Use Native Package Managers (Recommended)

**Rationale**: Native package managers provide:

- ✅ Standard tooling (pip, cargo, npm, CMake)
- ✅ Version resolution and dependency management
- ✅ Easy installation for users
- ✅ CI/CD integration
- ✅ Security scanning support

**Alternative Considered**: Git submodules

- ❌ More complex to manage
- ❌ Harder versioning
- ❌ Less standard tooling

---

## Language-Specific Strategies

### C++ Projects (`box-spread-cpp`)

#### Package Manager: CMake FetchContent + Conan/vcpkg (Future)

**Current State**: Uses CMake FetchContent for dependencies

**Strategy**:

1. **Phase 1**: Publish to GitHub Releases (CMake FetchContent)
   - Tag releases: `v1.0.0`, `v1.1.0`, etc.
   - CMake can fetch from GitHub releases
   - Simple, no registry needed initially

2. **Phase 2**: Publish to Conan or vcpkg (Optional)
   - Better for C++ ecosystem
   - More discoverable
   - Requires registry setup

**CMake FetchContent Example**:

```cmake
include(FetchContent)

FetchContent_Declare(
  box_spread_cpp
  GIT_REPOSITORY https://github.com/davidl71/box-spread-cpp.git
  GIT_TAG v1.0.0
)

FetchContent_MakeAvailable(box_spread_cpp)

target_link_libraries(my_app PRIVATE box_spread_cpp)
```

**Conan Example** (Future):

```cmake

# conanfile.txt

[requires]
box-spread-cpp/1.0.0@davidl71/stable

[generators]
CMakeDeps
CMakeToolchain
```

---

### Python Projects (`box-spread-python`, `project-housekeeping-tools`)

#### Package Manager: PyPI

**Strategy**: Publish to PyPI (Python Package Index)

**Registry Setup**:

1. **Create PyPI Account**: `davidl71` (or organization account)
2. **API Token**: Generate token for CI/CD publishing
3. **TestPyPI**: Use for testing before production

**PyPI Package Names**:

- `box-spread-python` - Main Python package
- `project-housekeeping-tools` - Automation framework

**Versioning**: Semantic versioning (see below)

**Installation Example**:

```bash
pip install box-spread-python>=1.0.0
```

**pyproject.toml Example**:

```toml
[project]
name = "box-spread-python"
version = "1.0.0"
dependencies = [
    "box-spread-cpp>=1.0.0",  # If C++ lib is also on PyPI
    "numpy>=1.24.0",
]
```

**Publishing Workflow**:

```bash

# Build package

python -m build

# Publish to TestPyPI (testing)

python -m twine upload --repository testpypi dist/*

# Publish to PyPI (production)

python -m twine upload dist/*
```

---

### Rust Projects (Future: If Rust components are extracted)

#### Package Manager: crates.io

**Strategy**: Publish to crates.io (Rust package registry)

**Registry Setup**:

1. **Create crates.io Account**: Link GitHub account
2. **API Token**: Generate for CI/CD
3. **Publishing**: Use `cargo publish`

**Package Name**: `box-spread-rust` (if extracted)

**Cargo.toml Example**:

```toml
[package]
name = "box-spread-rust"
version = "1.0.0"

[dependencies]
box-spread-cpp = "1.0.0"  # If C++ lib has Rust bindings
```

---

### TypeScript/Node Projects (Future: If web components are extracted)

#### Package Manager: npm

**Strategy**: Publish to npm (Node Package Manager)

**Registry Setup**:

1. **Create npm Account**: `davidl71` (or organization)
2. **Access Token**: Generate for CI/CD
3. **Scoped Packages**: Use `@davidl71/` scope

**Package Name**: `@davidl71/trading-mcp-servers` (if MCP servers have Node components)

**package.json Example**:

```json
{
  "name": "@davidl71/trading-mcp-servers",
  "version": "1.0.0",
  "dependencies": {
    "@modelcontextprotocol/sdk": "^1.0.0"
  }
}
```

---

## Semantic Versioning Strategy

### Version Format: `MAJOR.MINOR.PATCH`

**Rules**:

- **MAJOR**: Breaking changes (incompatible API changes)
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Versioning Examples

**Initial Release**:

- `box-spread-cpp`: `1.0.0`
- `box-spread-python`: `1.0.0`
- `trading-mcp-servers`: `1.0.0`

**Feature Addition** (Backward Compatible):

- `box-spread-cpp`: `1.0.0` → `1.1.0`
- `box-spread-python`: `1.0.0` → `1.1.0`

**Bug Fix**:

- `box-spread-cpp`: `1.1.0` → `1.1.1`
- `box-spread-python`: `1.1.0` → `1.1.1`

**Breaking Change**:

- `box-spread-cpp`: `1.1.1` → `2.0.0`
- `box-spread-python`: `1.1.1` → `2.0.0` (must update to match)

---

## Dependency Documentation

### README.md Template

Each public repository should include:

```markdown

## Dependencies

### Runtime Dependencies
- `box-spread-cpp>=1.0.0` - Core C++ engine
- `numpy>=1.24.0` - Numerical computing

### Development Dependencies
- `pytest>=7.4.0` - Testing framework
- `cython>=3.0.0` - Cython compiler

## Installation

```bash
pip install box-spread-python>=1.0.0
```

## Version Compatibility

| box-spread-python | box-spread-cpp | Python |
|-------------------|----------------|--------|
| 1.0.x             | 1.0.x          | >=3.11 |
| 1.1.x             | 1.0.x - 1.1.x  | >=3.11 |
| 2.0.x             | 2.0.x          | >=3.11 |

```

---

## Version Compatibility Matrix

### Cross-Project Compatibility

| Project | Version | Compatible With |
|---------|---------|-----------------|
| `box-spread-cpp` | 1.0.0 | All 1.x versions |
| `box-spread-python` | 1.0.0 | `box-spread-cpp>=1.0.0,<2.0.0` |
| `trading-mcp-servers` | 1.0.0 | `box-spread-python>=1.0.0` (optional) |
| `project-housekeeping-tools` | 1.0.0 | Independent |

### Breaking Change Policy

**When to Bump MAJOR Version**:
1. API changes that break existing code
2. Removing deprecated features
3. Changing dependency requirements significantly

**Coordination**:

- If `box-spread-cpp` bumps to 2.0.0, `box-spread-python` should also bump to 2.0.0
- Document breaking changes in CHANGELOG.md
- Provide migration guide

---

## Registry Account Setup

### PyPI Account

**Steps**:
1. Create account at https://pypi.org/account/register/
2. Verify email
3. Generate API token: Account Settings → API tokens
4. Store token in CI/CD secrets

**TestPyPI** (for testing):
1. Create account at https://test.pypi.org/account/register/
2. Use separate token for TestPyPI

### GitHub Releases (for C++)

**Steps**:
1. Tag releases: `git tag v1.0.0`
2. Push tags: `git push origin v1.0.0`
3. Create release on GitHub
4. Upload artifacts (optional)

### npm Account (if needed)

**Steps**:
1. Create account at https://www.npmjs.com/signup
2. Verify email
3. Generate access token: Account Settings → Access Tokens
4. Store token in CI/CD secrets

---

## CI/CD Publishing Workflows

### GitHub Actions Example (PyPI)

```yaml
name: Publish to PyPI

on:
  release:
    types: [published]

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: '3.11'

      - name: Build package
        run: python -m build

      - name: Publish to PyPI
        env:
          TWINE_USERNAME: __token__
          TWINE_PASSWORD: ${{ secrets.PYPI_API_TOKEN }}
        run: python -m twine upload dist/*
```

### GitHub Actions Example (C++ GitHub Releases)

```yaml
name: Create Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build
        run: |
          cmake --preset macos-universal-release
          cmake --build build

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: build/ib_box_spread
```

---

## Dependency Update Workflow

### Automated Updates

**Dependabot Configuration** (`.github/dependabot.yml`):

```yaml
version: 2
updates:
  - package-ecosystem: "pip"
    directory: "/python"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 10

  - package-ecosystem: "cargo"
    directory: "/agents/backend"
    schedule:
      interval: "weekly"

  - package-ecosystem: "npm"
    directory: "/web"
    schedule:
      interval: "weekly"
```

### Manual Updates

**Process**:

1. Review dependency updates
2. Test compatibility
3. Update version constraints
4. Run tests
5. Update CHANGELOG.md
6. Commit and tag release

---

## Security Considerations

### Dependency Scanning

**Tools**:

- **Python**: `pip-audit`, `safety`
- **Rust**: `cargo audit`
- **npm**: `npm audit`
- **C++**: Manual review (no standard tool)

**Automation**:

- Run scans in CI/CD
- Block merges if critical vulnerabilities found
- Auto-update non-breaking security patches

### Supply Chain Security

**Best Practices**:

1. Pin exact versions in production (lock files)
2. Use dependency scanning tools
3. Review dependency licenses
4. Monitor for security advisories
5. Use signed releases (GPG signing)

---

## Migration Plan

### Phase 1: Setup (T-199)

1. ✅ Create PyPI account
2. ✅ Set up GitHub Releases workflow
3. ✅ Document versioning strategy
4. ✅ Create dependency documentation templates

### Phase 2: First Releases

1. Publish `box-spread-cpp` v1.0.0 to GitHub Releases
2. Publish `box-spread-python` v1.0.0 to PyPI
3. Update private repo to use published packages
4. Test dependency resolution

### Phase 3: Expand

1. Publish remaining packages as they're extracted
2. Set up automated publishing workflows
3. Enable Dependabot for dependency updates
4. Monitor and maintain compatibility matrix

---

## Dependency Graph Visualization

```
┌─────────────────────┐
│  box-spread-cpp     │ (Independent)
│  (C++ Library)      │
└──────────┬──────────┘
           │
           │ depends on
           ↓
┌─────────────────────┐
│  box-spread-python   │ (Depends on C++ lib)
│  (Python Package)    │
└──────────┬──────────┘
           │
           │ optional
           ↓
┌─────────────────────┐
│ trading-mcp-servers  │ (Optional dependency)
│  (MCP Servers)       │
└─────────────────────┘

┌─────────────────────┐
│ project-housekeeping │ (Independent)
│ -tools              │
└─────────────────────┘
```

---

## Next Steps

1. ✅ **Dependency Management Strategy Defined** (this document)
2. ⬜ **Create PyPI Account** (manual step)
3. ⬜ **Set Up CI/CD Publishing Workflows** (T-200+)
4. ⬜ **Publish First Packages** (T-204, T-205)
5. ⬜ **Update Private Repo** (T-213)

---

## Summary

**Package Managers**:

- C++: CMake FetchContent (GitHub Releases) → Conan/vcpkg (future)
- Python: PyPI
- Rust: crates.io (if needed)
- npm: npm (if needed)

**Versioning**: Semantic versioning (MAJOR.MINOR.PATCH)

**Registry Accounts**: PyPI account needed, GitHub Releases for C++

**Status**: ✅ Strategy complete, ready for implementation
