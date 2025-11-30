# TUI Migration: C++ to Python (and Future pybind11 Migration)

## Overview

The TUI has been migrated from C++ (`native/src/tui_app.cpp`) to Python (`python/tui/`) to:

1. Improve development velocity
2. Share code with the PWA frontend
3. Use official Python APIs directly
4. Enable easier maintenance

Future migration back to C++ via pybind11 is planned and documented.

## Current Architecture

### Python TUI Structure

```
python/tui/
├── __init__.py          # Module initialization with migration notes
├── models.py            # Shared data models (matches PWA TypeScript types)
├── providers.py         # Data providers (mock, REST, file)
├── app.py               # Main TUI application (Textual)
├── config.py            # Configuration management
├── __main__.py          # Entry point
└── README.md            # Usage documentation
```

### Data Flow

```
Provider (Python)
    ↓
SnapshotPayload (shared model)
    ↓
TUI App (Textual - Python)
    ↓
Terminal Display
```

### Shared Models

The `tui/models.py` module defines data structures that match:

- **TypeScript**: `web/src/types/snapshot.ts`
- **C++**: `native/include/tui_data.h`

This ensures data consistency across all frontends.

## Migration from C++ TUI

### Key Changes

1. **UI Library**: FTXUI (C++) → Textual (Python)
2. **Data Models**: C++ structs → Python dataclasses
3. **Providers**: C++ classes → Python classes
4. **Configuration**: C++ JSON parsing → Python JSON parsing

### Feature Parity

| Feature | C++ TUI | Python TUI | Status |
|---------|---------|-------------|--------|
| Dashboard tab | ✅ | ✅ | Complete |
| Positions tab | ✅ | ✅ | Complete |
| Orders tab | ✅ | ✅ | Complete |
| Alerts tab | ✅ | ✅ | Complete |
| Historic tab | ⚠️ | ⚠️ | Placeholder |
| Mock provider | ✅ | ✅ | Complete |
| REST provider | ✅ | ✅ | Complete |
| File provider | ✅ | ✅ | Complete |
| IBKR REST provider | ✅ | ⏳ | TODO |
| LiveVol provider | ✅ | ⏳ | TODO |
| Setup screen | ✅ | ⏳ | TODO |
| Keyboard shortcuts | ✅ | ✅ | Complete |

## Future Migration to C++ (pybind11)

### Strategy

The Python TUI is designed to be migrated back to C++ incrementally using pybind11:

1. **Phase 1**: Expose C++ data models via pybind11
2. **Phase 2**: Migrate providers to C++ with Python wrappers
3. **Phase 3**: Keep UI in Python (Textual) or migrate to FTXUI
4. **Phase 4**: Optional full C++ migration

### Migration Points

#### Data Models (`tui/models.py`)

**Current (Python)**:

```python
@dataclass
class SnapshotPayload:
    generated_at: str
    mode: str
    # ...
```

**Future (C++ with pybind11)**:

```cpp
// C++ side
struct SnapshotPayload {
    std::string generated_at;
    std::string mode;
    // ...
};

// Python binding
PYBIND11_MODULE(tui_cpp, m) {
    py::class_<SnapshotPayload>(m, "SnapshotPayload")
        .def(py::init<>())
        .def_readwrite("generated_at", &SnapshotPayload::generated_at)
        .def_readwrite("mode", &SnapshotPayload::mode)
        // ...
        .def("to_dict", &SnapshotPayload::ToDict)
        .def_static("from_dict", &SnapshotPayload::FromDict);
}
```

#### Providers (`tui/providers.py`)

**Current (Python)**:

```python
class Provider(ABC):
    @abstractmethod
    def get_snapshot(self) -> SnapshotPayload:
        pass
```

**Future (C++ with pybind11)**:

```cpp
// C++ side
class Provider {
public:
    virtual ~Provider() = default;
    virtual void Start() = 0;
    virtual void Stop() = 0;
    virtual SnapshotPayload GetSnapshot() = 0;
    virtual bool IsRunning() const = 0;
};

// Python binding
PYBIND11_MODULE(tui_cpp, m) {
    py::class_<Provider>(m, "Provider")
        .def("start", &Provider::Start)
        .def("stop", &Provider::Stop)
        .def("get_snapshot", &Provider::GetSnapshot)
        .def("is_running", &Provider::IsRunning);
}
```

#### UI (`tui/app.py`)

**Options**:

1. **Keep in Python**: Textual is Python-only, keep UI in Python
2. **Hybrid**: C++ for data processing, Python for UI
3. **Full C++**: Migrate to FTXUI (more complex)

**Recommended**: Keep UI in Python, migrate data processing to C++

### Migration Checklist

- [ ] Create pybind11 bindings for data models
- [ ] Migrate MockProvider to C++
- [ ] Migrate RestProvider to C++ (or keep in Python)
- [ ] Migrate FileProvider to C++ (or keep in Python)
- [ ] Add IBKR REST provider in C++
- [ ] Add LiveVol provider in C++
- [ ] Performance testing
- [ ] Update documentation
- [ ] Update build system

## Using Official APIs

The Python TUI can use official Python APIs directly:

### IBKR Client Portal API

```python
from python.integration.ibkr_portal_client import IBKRPortalClient

client = IBKRPortalClient(base_url="https://localhost:5000/v1/portal")
account_data = client.get_account_summary()
```

### ORATS API

```python
from python.integration.orats_client import ORATSClient

client = ORATSClient(api_key="...")
core_data = client.get_core_data("SPX")
```

### Other APIs

- Alpaca: `python/integration/alpaca_client.py`
- QuestDB: `python/integration/questdb_client.py`
- Nautilus: `python/integration/nautilus_client.py`

## Running the Python TUI

### Basic Usage

```bash

# Install dependencies

pip install textual requests

# Run with mock provider

python -m python.tui

# Run with REST provider

export TUI_BACKEND=rest
export TUI_API_URL=http://localhost:8080/api/snapshot
python -m python.tui

# Run with file provider

export TUI_BACKEND=file
export TUI_SNAPSHOT_FILE=web/public/data/snapshot.json
python -m python.tui
```

### Configuration

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

## Performance Considerations

### Python TUI Performance

- **Startup**: ~200-300ms (vs ~100ms for C++)
- **Update Rate**: 500ms (configurable)
- **Memory**: ~50-100MB (vs ~20-30MB for C++)
- **CPU**: Low (<5% on modern hardware)

### When to Migrate to C++

Consider migrating to C++ if:

- Performance becomes a bottleneck
- Memory usage is critical
- Need to integrate with C++ trading libraries directly
- Want to reduce Python dependency

## Code Sharing with PWA

### Shared Data Models

Both Python TUI and PWA use the same data structures:

**Python** (`python/tui/models.py`):

```python
@dataclass
class SnapshotPayload:
    generated_at: str
    symbols: List[SymbolSnapshot]
    # ...
```

**TypeScript** (`web/src/types/snapshot.ts`):

```typescript
export interface SnapshotPayload {
  generated_at: string;
  symbols: SymbolSnapshot[];
  // ...
}
```

### Shared REST API

Both frontends consume the same REST API:

- Endpoint: `/api/snapshot` (or configurable)
- Format: JSON matching `SnapshotPayload` structure
- Updates: Polling (configurable interval)

## Documentation

- **Usage**: `python/tui/README.md`
- **Migration Notes**: This document
- **Code Comments**: All modules include migration notes

## Questions?

- See `python/tui/README.md` for usage
- Check code comments for migration notes
- Review `native/include/tui_data.h` for C++ structure reference
