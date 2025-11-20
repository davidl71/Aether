# Cursor Ignore Configuration

## Overview

This document explains the `.cursorignore` file and VS Code settings that exclude files from Cursor's AI analysis and [codebase indexing](https://cursor.com/docs/context/codebase-indexing) to streamline development and reduce unnecessary prompts.

**Important**: Files in `.cursorignore` (and `.gitignore`) are excluded from Cursor's codebase indexing, which means they won't appear in semantic search results. This improves indexing performance and answer accuracy.

## What's Excluded

### Third-Party Vendor Code (~148MB)

These large vendor directories are excluded from AI analysis:

1. **TWS API** (21MB)
   - Samples and examples (`IBJts/samples/`)
   - Python/Java clients (not used)
   - Generated protobuf files (`*.pb.cc`, `*.pb.h`)
   - Build artifacts

2. **Intel Decimal Library** (38MB)
   - Example code (`EXAMPLES/`)
   - Test suites (`TESTS/`)
   - Source implementation (`LIBRARY/src/`)
   - Float128 implementation (`LIBRARY/float128/`)
   - Build scripts and artifacts

3. **Nautilus Trader** (89MB)
   - Python wheel file (binary distribution)
   - Not source code, just a dependency

### Build Artifacts & Generated Files

- All `build/` directories
- CMake generated files (`CMakeFiles/`, `CMakeCache.txt`, etc.)
- Compiled binaries (`*.o`, `*.so`, `*.dylib`, etc.)
- Generated protobuf files (`*.pb.cc`, `*.pb.h`, `*.pb.go`)

### Dependencies

- Python: `__pycache__/`, `venv/`, `dist/`
- Node.js: `node_modules/`
- Rust: `target/`, `Cargo.lock`
- Go: `vendor/`

### Other Exclusions

- Test artifacts and coverage files
- Logs and temporary files
- Distribution archives
- User-specific configuration files
- Git worktrees

## What's Still Available

**Important**: Excluding files from AI analysis doesn't prevent:
- **IntelliSense**: Headers are still indexed for autocomplete
- **Build System**: CMake can still find all dependencies
- **Compilation**: All source files are still compiled

The TWS API headers (`native/third_party/tws-api/IBJts/source/cppclient/client/*.h`) remain accessible for:
- IntelliSense autocomplete
- Code navigation
- Build system

Only the **samples**, **tests**, **generated protobuf files**, and **build artifacts** are excluded from AI analysis.

## Benefits

1. **Faster AI Responses**: Excluding ~148MB of vendor code reduces context size
2. **Fewer Irrelevant Prompts**: AI won't suggest changes to vendor code
3. **Better Focus**: AI focuses on your actual source code
4. **Reduced Token Usage**: Smaller context means lower API costs

## Files Modified

1. **`.cursorignore`**: New file for Cursor-specific exclusions
2. **`.vscode/settings.json`**: Updated `files.exclude`, `search.exclude`, and `files.watcherExclude`

## Verification

To verify exclusions are working:

1. Open a vendor file (e.g., `native/third_party/tws-api/IBJts/samples/...`)
2. Try asking Cursor AI about it - it should not have context
3. Check file explorer - excluded directories should be grayed out or hidden

## Customization

To add more exclusions, edit `.cursorignore` using `.gitignore` syntax:

```bash
# Example: Exclude a specific directory
path/to/exclude/

# Example: Exclude a file pattern
**/*.generated.cpp
```

## Notes

- `.cursorignore` uses the same syntax as `.gitignore`
- VS Code settings (`files.exclude`) control file explorer visibility
- Cursor AI respects both `.cursorignore` and VS Code exclusions
- IntelliSense still works for excluded files (headers remain indexed)
