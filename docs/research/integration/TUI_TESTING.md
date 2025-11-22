# TUI Testing Guide

**Note**: TUI has been migrated from Go to C++ (FTXUI library). This document is being updated.

## Overview

The C++ TUI uses FTXUI for rendering and is built as part of the main CMake build system.

## Current Status

- **Implementation**: C++ with FTXUI (`native/src/tui_app.cpp`)
- **Build**: `cmake --build build --target ib_box_spread_tui`
- **Testing**: C++ TUI tests should be added to `native/tests/`

## Testing Strategy (To Be Implemented)

### 1. Unit Tests

Test individual functions and components in isolation.

**Location**: `native/tests/tui_app_test.cpp` (to be created)

**Examples**:

- Data conversion functions (`tui_converter.cpp`)
- Configuration loading (`tui_config.cpp`)
- Provider implementations (`tui_provider.cpp`)

### 2. Integration Tests

Test TUI components working together.

**Location**: `native/tests/tui_integration_test.cpp` (to be created)

**Examples**:

- Provider → Converter → Display pipeline
- Configuration file loading
- Mock provider data flow

### 3. Manual Testing

Since FTXUI is an interactive library, manual testing is important:

```bash
# Build TUI
cmake --build build --target ib_box_spread_tui

# Run with mock data
./build/ib_box_spread_tui

# Test with different backends (via config file)
TUI_BACKEND=mock ./build/ib_box_spread_tui
TUI_BACKEND=rest ./build/ib_box_spread_tui
```

## Configuration

TUI uses config files instead of command-line flags:

- **Config location**: `~/.config/ib_box_spread/tui_config.json`
- **Environment variables**: `TUI_BACKEND`, `TUI_API_URL`

## Future Testing Improvements

1. Add Catch2 tests for TUI components
2. Add snapshot testing for UI rendering
3. Add integration tests for provider → display pipeline
4. Add performance tests for real-time updates

## Migration Notes

The previous Go TUI (`tui/`) has been removed. All TUI functionality is now in C++:

- `native/src/tui_app.cpp` - Main TUI application
- `native/src/tui_provider.cpp` - Data providers
- `native/src/tui_converter.cpp` - Type conversions
- `native/src/tui_config.cpp` - Configuration management
- `native/include/tui_*.h` - Headers
