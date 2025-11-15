# Cursor Global Documentation Guide

This document lists all recommended global documentation files to add to Cursor's `@docs` feature for optimal AI assistance.

## Overview

Cursor's `@docs` feature allows you to reference documentation files directly in prompts, giving the AI better context about your project. This guide categorizes documentation into **internal** (project-specific) and **external** (third-party) resources.

## How to Add Global Docs in Cursor

1. Open Cursor Settings
2. Navigate to "Features" → "Docs"
3. Click "Add Doc" or "Add Folder"
4. Select the files/directories listed below
5. Files will be automatically indexed and available via `@docs`

## High-Priority Internal Documentation

These files should be added as **global Docs** for maximum AI context:

### Core API & Integration Documentation
- ✅ **`docs/API_DOCUMENTATION_INDEX.md`** - Complete index of all external APIs and libraries
  - **Why**: Primary reference for all API usage patterns
  - **Use Case**: "How do I use TWS API?" → `@docs API_DOCUMENTATION_INDEX.md`

- ✅ **`docs/TWS_INTEGRATION_STATUS.md`** - TWS API integration details and status
  - **Why**: Specific TWS API implementation patterns and gotchas
  - **Use Case**: "How do I connect to TWS?" → `@docs TWS_INTEGRATION_STATUS.md`

- ✅ **`docs/EWRAPPER_STATUS.md`** - EWrapper implementation status
  - **Why**: EWrapper callback implementation details
  - **Use Case**: "How do I implement EWrapper callbacks?" → `@docs EWRAPPER_STATUS.md`

### Architecture & Design
- ✅ **`docs/CODEBASE_ARCHITECTURE.md`** - System design and component interactions
  - **Why**: Understanding overall system architecture
  - **Use Case**: "How does the order manager interact with TWS?" → `@docs CODEBASE_ARCHITECTURE.md`

- ✅ **`docs/COMMON_PATTERNS.md`** - Coding patterns, conventions, and idioms
  - **Why**: Ensures code consistency with project patterns
  - **Use Case**: "How should I structure this class?" → `@docs COMMON_PATTERNS.md`

### Trading Domain Knowledge
- ✅ **`docs/BOX_SPREAD_COMPREHENSIVE_GUIDE.md`** - Complete box spread mechanics and implementation
  - **Why**: Core trading strategy knowledge
  - **Use Case**: "What are the risks of box spreads?" → `@docs BOX_SPREAD_COMPREHENSIVE_GUIDE.md`

### Code Quality & Best Practices
- ✅ **`docs/AI_FRIENDLY_CODE.md`** - Best practices for writing AI-friendly code
  - **Why**: Ensures code is maintainable and AI-assistable
  - **Use Case**: "How should I name this function?" → `@docs AI_FRIENDLY_CODE.md`

- ✅ **`docs/STATIC_ANALYSIS_ANNOTATIONS.md`** - Guide on static analysis annotations
  - **Why**: Proper use of function attributes and annotations
  - **Use Case**: "Should I use [[nodiscard]] here?" → `@docs STATIC_ANALYSIS_ANNOTATIONS.md`

### Implementation Guides
- ✅ **`docs/IMPLEMENTATION_GUIDE.md`** - Step-by-step implementation guide
  - **Why**: Step-by-step development workflow
  - **Use Case**: "What's the next step in implementation?" → `@docs IMPLEMENTATION_GUIDE.md`

- ✅ **`docs/QUICK_START.md`** - Quick start guide
  - **Why**: Getting started quickly
  - **Use Case**: "How do I build and run this?" → `@docs QUICK_START.md`

## Secondary Internal Documentation

These files are useful but less critical for global context:

### Build & Development
- `docs/DISTRIBUTED_COMPILATION.md` - Build optimization guide
- `docs/WORKTREE_SETUP.md` - Development worktree setup
- `docs/CURSOR_SETUP.md` - Cursor IDE setup
- `docs/CURSOR_DOCS_USAGE.md` - How to use @docs (meta-documentation)

### Testing & Quality
- `docs/STATIC_ANALYSIS.md` - Static analysis tools overview
- `docs/INTEGRATION_TESTING.md` - Integration testing guide
- `docs/TUI_TESTING.md` - Terminal UI testing

### External Integrations
- `docs/ORATS_INTEGRATION.md` - ORATS API integration
- `docs/ONEPASSWORD_INTEGRATION.md` - 1Password integration

## External Documentation (Downloaded)

The following external documentation has been downloaded and added to `docs/external/`:

### TWS API Documentation
- ✅ **`docs/external/TWS_API_QUICK_REFERENCE.md`** - TWS API quick reference
  - **Source**: Interactive Brokers TWS API documentation
  - **Why**: Essential TWS API patterns and classes
  - **Use Case**: "What's the EClient class structure?" → `@docs external/TWS_API_QUICK_REFERENCE.md`

- ✅ **`docs/external/ECLIENT_EWRAPPER_PATTERNS.md`** - EClient/EWrapper patterns
  - **Source**: IBKR Campus and official TWS API docs
  - **Why**: Common patterns for TWS API usage
  - **Use Case**: "How do I structure my TWS client?" → `@docs external/ECLIENT_EWRAPPER_PATTERNS.md`

### CMake Documentation
- ✅ **`docs/external/CMake_PRESETS_GUIDE.md`** - CMake presets guide
  - **Source**: CMake official documentation
  - **Why**: CMake presets configuration and usage
  - **Use Case**: "How do I configure CMake presets?" → `@docs external/CMake_PRESETS_GUIDE.md`

### C++ Standards & Libraries
- ✅ **`docs/external/CPP20_FEATURES.md`** - C++20 features reference
  - **Source**: cppreference.com
  - **Why**: C++20 features used in project
  - **Use Case**: "What C++20 features can I use?" → `@docs external/CPP20_FEATURES.md`

## External Documentation (URL References)

These external resources are best referenced via URL (if Cursor supports it) or bookmarked:

### TWS API
- **Official Docs**: https://interactivebrokers.github.io/tws-api/
- **GitHub**: https://github.com/InteractiveBrokers/tws-api
- **IBKR Campus**: https://www.interactivebrokers.com/campus/ibkr-quant-news/
- **Release Notes**: https://ibkrguides.com/releasenotes/prod-2025.htm

### CMake
- **Official Docs**: https://cmake.org/documentation/
- **Presets**: https://cmake.org/cmake/help/latest/manual/cmake-presets.7.html
- **CMakeLists.txt Guide**: https://cmake.org/cmake/help/latest/manual/cmake-buildsystem.7.html

### C++ Reference
- **cppreference.com**: https://en.cppreference.com/
- **C++20 Standard**: https://en.cppreference.com/w/cpp/20

### Libraries
- **spdlog**: https://github.com/gabime/spdlog
- **Catch2**: https://github.com/catchorg/Catch2
- **Protocol Buffers**: https://protobuf.dev/

## Usage Examples

### Example 1: TWS API Connection
```
@docs API_DOCUMENTATION_INDEX.md @docs TWS_INTEGRATION_STATUS.md
How do I connect to TWS API on port 7497 (paper trading)?
```

### Example 2: Code Pattern Question
```
@docs COMMON_PATTERNS.md @docs AI_FRIENDLY_CODE.md
How should I structure a new order manager class?
```

### Example 3: Trading Strategy Question
```
@docs BOX_SPREAD_COMPREHENSIVE_GUIDE.md
What are the risks of short box spreads?
```

### Example 4: Build System Question
```
@docs external/CMake_PRESETS_GUIDE.md
How do I add a new CMake preset for testing?
```

### Example 5: Static Analysis
```
@docs STATIC_ANALYSIS_ANNOTATIONS.md
Should I add [[nodiscard]] to this function?
```

## Automation

**See `docs/CURSOR_GLOBAL_DOCS_AUTOMATION.md` for complete automation guide.**

### Quick Commands

```bash
# Validate all docs exist
python3 scripts/sync_global_docs.py --check

# Generate path lists for Cursor
python3 scripts/sync_global_docs.py --generate-paths

# Detect new documentation files
python3 scripts/sync_global_docs.py --detect-new

# Full sync (validate + generate + update)
python3 scripts/sync_global_docs.py --update-config --generate-paths
```

### Configuration File

All global docs are defined in `.cursor/global-docs.json`. This file is:
- Version controlled
- Used by automation scripts
- Source of truth for all global docs

## Maintenance

### When to Update This Guide
- Add new high-priority documentation files
- Download new external documentation sections
- Update external URL references
- Document new patterns or conventions

### Keeping Documentation Current
1. Run `python3 scripts/sync_global_docs.py --check` regularly
2. Review this guide quarterly
3. Update when major features are added
4. Remove deprecated documentation references
5. Add new external docs as they become relevant
6. Use `--detect-new` to find new documentation files

## Quick Reference Checklist

Use this checklist when setting up Cursor global Docs:

### Must-Have (Add These First)
- [ ] `docs/API_DOCUMENTATION_INDEX.md`
- [ ] `docs/CODEBASE_ARCHITECTURE.md`
- [ ] `docs/COMMON_PATTERNS.md`
- [ ] `docs/AI_FRIENDLY_CODE.md`
- [ ] `docs/TWS_INTEGRATION_STATUS.md`
- [ ] `docs/BOX_SPREAD_COMPREHENSIVE_GUIDE.md`
- [ ] `docs/STATIC_ANALYSIS_ANNOTATIONS.md`
- [ ] `docs/IMPLEMENTATION_GUIDE.md`

### Should-Have (Add After Must-Haves)
- [ ] `docs/EWRAPPER_STATUS.md`
- [ ] `docs/QUICK_START.md`
- [ ] `docs/external/TWS_API_QUICK_REFERENCE.md`
- [ ] `docs/external/ECLIENT_EWRAPPER_PATTERNS.md`
- [ ] `docs/external/CMake_PRESETS_GUIDE.md`

### Nice-to-Have (Add As Needed)
- [ ] Other secondary documentation files
- [ ] Additional external documentation sections

## Related Documentation

- **`docs/CURSOR_DOCS_USAGE.md`** - Detailed guide on using `@docs` in Cursor
- **`.cursorrules`** - Project rules that reference global Docs strategy
- **`docs/DOCUMENTATION_INDEX.md`** - Complete documentation index

## Notes

- Global Docs are indexed by Cursor for fast retrieval
- You can reference multiple docs in a single prompt
- Use specific section anchors when possible: `@docs FILE.md#section`
- External URLs may not work directly with `@docs` - download key sections instead
- Keep external docs updated when APIs change
