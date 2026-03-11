# Python Bindings Installation Guide

## Prerequisites

1. **Python 3.11 or higher**
2. **Cython 3.0+**
3. **CMake** (for building C++ dependencies)
4. **C++ Compiler** (Clang/GCC with C++20 support)

## Installation Steps

### 1. Install Python Dependencies (Core)

```bash
# From project root
uv sync --project python --extra dev --extra tui
```

This installs the pinned dependencies captured in `python/pyproject.toml` and `python/uv.lock`. Core packages include NumPy, Cython, pytest/pytest-cov, requests, urllib3, and Textual.

To refresh the lockfile after updating direct dependencies, run:

```bash
uv lock --project python
```

The repo standard is `uv`; avoid maintaining a separate root `requirements.txt`.

For Homebrew installations, the native CLI and Python entrypoints also look for user configuration
at `$HOME/.config/ib_box_spread/config.json` (or `~/Library/Application Support/ib_box_spread/config.json`
on macOS). Copy the packaged example to that directory or pass an explicit `--config /path/to/config.json`
when launching the tooling. The native binary can scaffold a starter file as well:

```bash
ib_box_spread --init-config                # writes ~/.config/ib_box_spread/config.json
ib_box_spread --init-config ./custom.json  # writes to a custom path
```

### 2. Build Cython Bindings

#### Option A: Using setuptools (Recommended)

```bash
cd python/bindings
uv pip install --python ../.venv/bin/python -e .
```

This will:
- Compile the `.pyx` file to C++
- Link with C++ source files
- Install the `box_spread_bindings` module

#### Option B: Using CMake

```bash
# From project root
cmake -B build -DENABLE_PYTHON_BINDINGS=ON
cmake --build build --target python_bindings
```

### 3. Verify Installation

```bash
# Test Python imports
uv run --project python python -c "from python.bindings.box_spread_bindings import PyOptionContract; print('Bindings OK')"

# Run Python tests
uv run --project python pytest python/tests/
```

## Troubleshooting

### Cython Not Found

```bash
uv sync --project python --extra dev --extra tui
```

### Compilation Errors

1. Check that C++ headers are in `include/` directory
2. Verify C++ source files are in `src/` directory
3. Ensure C++ compiler supports C++20

### Import Errors

```bash
# Make sure you're in the project root or have PYTHONPATH set
export PYTHONPATH="${PYTHONPATH}:$(pwd)/python"
```

## Development

For development with automatic recompilation:

```bash
cd python/bindings
uv pip install --python ../.venv/bin/python -e . --no-build-isolation
```

Then edit `.pyx` files and restart Python to see changes.

## Project Structure

```
python/
├── bindings/           # Cython bindings
│   ├── box_spread_bindings.pxd  # Cython declarations
│   ├── box_spread_bindings.pyx  # Cython implementation
│   └── setup.py        # Build configuration
├── integration/        # Broker/bank/rates integrations
├── tui/                # Active Textual TUI
├── bindings/           # Cython bindings
└── tests/
```
