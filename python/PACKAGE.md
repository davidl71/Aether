# Package Installation Guide

## Quick Start

```bash
# From project root
cd python

# Install the package
pip install -e .

# Install with development dependencies
pip install -e ".[dev]"

# Build Cython bindings (separate package)
cd bindings
pip install -e .
```

## Package Structure

The package is organized as follows:

### Main Package (`python/`)

- **Integration modules** (`integration/`): NautilusTrader integration
- **Wrapper modules** (`wrapper/`): Bridge between Python and C++
- **Standalone modules**: `nautilus_strategy.py`, `config_adapter.py`

### Bindings Package (`python/bindings/`)

- **Cython bindings** for C++ calculations
- Must be built separately due to C++ compilation requirements

## Installation Methods

### Method 1: Development Installation (Recommended)

```bash
# Install main package
pip install -e .

# Install bindings
cd bindings && pip install -e . && cd ..
```

### Method 2: Production Installation

```bash
# Build wheel
python setup.py bdist_wheel

# Install from wheel
pip install dist/ib_box_spread_generator-*.whl

# Install bindings separately
cd bindings
python setup.py bdist_wheel
pip install dist/box_spread_bindings-*.whl
```

### Method 3: Using pyproject.toml

```bash
pip install .
```

## Usage After Installation

### Command-Line

```bash
# Use the installed entry point
ib-box-spread-nautilus --config ../config/config.json --dry-run
```

### Python Import

```python
# Import integration modules
from integration.nautilus_client import NautilusClient
from integration.strategy_runner import StrategyRunner
from integration.config_adapter import ConfigAdapter

# Import wrapper
from wrapper.nautilus_bridge import NautilusBridge

# Import bindings (if installed)
try:
    from bindings.box_spread_bindings import PyOptionContract, PyBoxSpreadLeg
except ImportError:
    print("Bindings not installed - run: cd bindings && pip install -e .")
```

## Dependencies

### Runtime Dependencies

- `nautilus_trader>=2.0.0`
- `numpy>=1.24.0`

### Build Dependencies (for bindings)

- `cython>=3.0.0`
- C++ compiler with C++20 support
- CMake (optional, for CMake integration)

### Development Dependencies

- `pytest>=7.4.0`
- `pytest-cov>=4.1.0`

## Troubleshooting

### Import Errors

If you get import errors, ensure the package is installed:

```bash
pip install -e .
```

### Bindings Not Found

Build and install the bindings:

```bash
cd bindings
pip install -e .
```

### Entry Point Not Found

Reinstall the package:

```bash
pip install -e .
```

## Building Distribution Packages

```bash
# Build source distribution
python setup.py sdist

# Build wheel
python setup.py bdist_wheel

# Build both
python setup.py sdist bdist_wheel
```

The built packages will be in `dist/` directory.


