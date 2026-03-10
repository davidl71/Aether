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

- **Integration modules** (`integration/`): broker/bank/rates integrations
- **TUI modules** (`tui/`): active Textual terminal client
- **Legacy Nautilus scaffolding**: deprecated; not part of the active supported runtime

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

### Python Import

Import the active integration, TUI, or bindings modules from the current package layout.

## Dependencies

### Runtime Dependencies

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

