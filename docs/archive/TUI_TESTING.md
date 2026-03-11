# TUI Testing Guide

**Note**: TUI has been migrated from C++ (FTXUI) to Python (Textual). This document has been updated.

## Overview

The Python TUI uses Textual for rendering and is run as a Python module.

## Current Status

- **Implementation**: Python with Textual (`python/tui/app.py`)
- **Run**: `python -m python.tui`
- **Testing**: Python TUI tests are in `python/tests/test_tui_*.py`

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

Since Textual is an interactive library, manual testing is important:

```bash
# Install dependencies
pip install textual requests

# Run with mock data (default)
python -m python.tui

# Test with different backends (via environment variables)
TUI_BACKEND=rest TUI_API_URL=http://localhost:8080/api/snapshot python -m python.tui
TUI_BACKEND=file TUI_SNAPSHOT_FILE=web/public/data/snapshot.json python -m python.tui
```

## Configuration

TUI uses config files instead of command-line flags:

- **Config location**: `~/.config/ib_box_spread/tui_config.json`
- **Environment variables**: `TUI_BACKEND`, `TUI_API_URL`

## Future Testing Improvements

1. Add pytest tests for TUI components
2. Add snapshot testing for UI rendering
3. Add integration tests for provider → display pipeline
4. Add performance tests for real-time updates

## Migration Notes

The C++ TUI (`native/src/tui_app.cpp`) has been removed. All TUI functionality is now in Python:

- `python/tui/app.py` - Main TUI application (Textual)
- `python/tui/providers.py` - Data providers
- `python/tui/models.py` - Data models (shared with PWA)

See `docs/archive/TUI_PYTHON_MIGRATION.md` for migration details.
