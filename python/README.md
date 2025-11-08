# IB Box Spread Generator - Python Package

Python integration package for IB Box Spread Generator with NautilusTrader support.

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
- NautilusTrader 2.0+
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
├── integration/           # NautilusTrader integration
│   ├── __init__.py
│   ├── nautilus_client.py
│   ├── market_data_handler.py
│   ├── execution_handler.py
│   └── strategy_runner.py
├── wrapper/               # Bridge modules
│   ├── __init__.py
│   └── nautilus_bridge.py
├── config_adapter.py
├── nautilus_strategy.py
├── setup.py
└── pyproject.toml
```

## Usage

After installation, use the command-line entry point:

```bash
ib-box-spread-nautilus --config config/config.json --dry-run
```

Or import in Python:

```python
from python.integration.nautilus_client import NautilusClient
from python.wrapper.nautilus_bridge import NautilusBridge

# Use the modules...
```

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


