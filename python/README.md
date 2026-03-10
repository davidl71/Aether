# IB Box Spread Generator - Python Package

Python integration package for the active Textual TUI, service integrations, and bindings. Old NautilusTrader references are deprecated scaffold/history, not an active supported runtime.

## Installation

### From Source

```bash
# Install the main package
pip install -e .

# Install with development dependencies
pip install -e ".[dev]"

# Build Cython bindings separately
cd bindings
pip install -e .
```

### Dependencies

The package requires:
- Python 3.11+
- Cython 3.0+ (for building bindings)
- NumPy 1.24+

## Package Structure

```
python/
├── bindings/              # Cython bindings package
│   ├── __init__.py
│   ├── box_spread_bindings.pxd
│   ├── box_spread_bindings.pyx
│   └── setup.py
├── bindings/              # Cython bindings package
├── integration/           # Broker/bank/rates integrations
├── tui/                   # Active Textual TUI
├── tests/
└── pyproject.toml
```

## Usage

After installation, use the TUI and integration modules from the active `python/` package layout.

## Development

```bash
# Install in development mode
pip install -e ".[dev]"

# Run tests
pytest tests/python/

# Build bindings
cd bindings && pip install -e .
```

## License

MIT License - see parent project LICENSE file.

