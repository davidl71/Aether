# Python TUI for IB Box Spread Trading

This is the Python replacement for the C++ TUI (`native/src/tui_app.cpp`). It provides the same functionality with better performance, easier maintenance, and shared code with the PWA frontend.

## Features

- **Shared Data Models**: Uses the same data structures as the PWA (`web/src/types/snapshot.ts`)
- **Multiple Providers**: Supports mock, REST API, and file-based data sources
- **Modern UI**: Built with [Textual](https://textual.textualize.io/) for a responsive terminal interface
- **Migration Ready**: Well-documented for future C++ migration via pybind11

## Installation

Install dependencies:

```bash
pip install textual requests
```

Or add to `requirements.txt`:

```
textual>=0.40.0
requests>=2.31.0
```

## Usage

### Basic Usage

Run the TUI with default (mock) provider:

```bash
python -m python.tui
```

### With REST API Provider

```bash
export TUI_BACKEND=rest
export TUI_API_URL=http://localhost:8080/api/snapshot
python -m python.tui
```

### With File Provider

```bash
export TUI_BACKEND=file
export TUI_SNAPSHOT_FILE=web/public/data/snapshot.json
python -m python.tui
```

### Configuration File

Configuration is stored in `~/.config/ib_box_spread/tui_config.json`:

```json
{
  "provider_type": "rest",
  "rest_endpoint": "http://localhost:8080/api/snapshot",
  "update_interval_ms": 1000,
  "refresh_rate_ms": 500,
  "rest_timeout_ms": 5000,
  "rest_verify_ssl": false,
  "show_colors": true,
  "show_footer": true
}
```

## Keyboard Shortcuts

- `Q` or `F10`: Quit
- `F1`: Help
- `F2`: Setup (coming soon)
- `F5`: Refresh
- `Tab` / `Shift+Tab`: Switch tabs

## Architecture

### Data Flow

```
Provider (Mock/REST/File)
    ↓
SnapshotPayload (shared model)
    ↓
TUI App (Textual)
    ↓
Terminal Display
```

### Shared Models

The `tui/models.py` module defines data structures that match:
- TypeScript types in `web/src/types/snapshot.ts`
- C++ types in `native/include/tui_data.h`

This ensures consistency across all frontends.

### Providers

Providers fetch data from various sources:
- **MockProvider**: Generates synthetic data for testing
- **RestProvider**: Polls REST API endpoints
- **FileProvider**: Reads from JSON files on disk

## Migration to C++ (Future)

This Python TUI is designed to be migrated back to C++ using pybind11. Key migration points:

### Data Models (`tui/models.py`)

- Expose dataclasses via `pybind11::class_` with properties
- Use `nlohmann/json` for JSON serialization on C++ side
- Keep Python models as source of truth for API contracts

### Providers (`tui/providers.py`)

- Provider interface can be abstract C++ class
- Python providers can call C++ implementations via pybind11
- Consider keeping Python as thin wrappers around C++ core

### UI (`tui/app.py`)

- UI rendering can stay in Python (Textual is Python-only)
- Data processing can move to C++ and expose via pybind11
- Keep Python TUI as reference implementation

### Example pybind11 Binding

```cpp
// C++ side
#include <pybind11/pybind11.h>
#include <pybind11/stl.h>

class Provider {
public:
    virtual ~Provider() = default;
    virtual void Start() = 0;
    virtual void Stop() = 0;
    virtual SnapshotPayload GetSnapshot() = 0;
};

// Python binding
PYBIND11_MODULE(tui_cpp, m) {
    py::class_<Provider>(m, "Provider")
        .def("start", &Provider::Start)
        .def("stop", &Provider::Stop)
        .def("get_snapshot", &Provider::GetSnapshot);
}
```

## Development

### Running Tests

```bash
pytest python/tui/tests/
```

### Code Style

Follow PEP 8 and use type hints. Run linters:

```bash
ruff check python/tui/
mypy python/tui/
```

## Comparison with C++ TUI

| Feature | C++ TUI | Python TUI |
|---------|---------|------------|
| Library | FTXUI | Textual |
| Performance | Faster | Fast enough |
| Maintenance | Complex | Easier |
| Shared Code | No | Yes (with PWA) |
| Migration | N/A | Ready for pybind11 |

## Notes

- The Python TUI is faster to develop and maintain
- Shared models ensure consistency with PWA
- Well-documented for future C++ migration
- Can use official Python APIs (IBKR, etc.) directly
