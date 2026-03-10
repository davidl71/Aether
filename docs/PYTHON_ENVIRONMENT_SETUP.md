# Python Environment Setup

This guide covers Python virtual environment setup for the project, including PyO3 compatibility and modern tooling with `uv`.

**Issue**: PyO3 0.21 supports Python up to 3.12, but system Python is 3.14
**Solution**: Use Python 3.12 virtual environment

**Modern Tooling**: The project now supports `uv` (fast Python package manager) with automatic fallback to standard `venv` and `pip`.

## Quick Start

### Using `uv` (Recommended - Faster)

```bash

# Install uv if not already installed

pip install uv

# Create virtual environment with uv (faster)

cd agents/backend
uv venv .venv
source .venv/bin/activate

# Set PyO3 to use this Python

export PYO3_PYTHON="$(which python)"
cargo check -p backend_service
```

### Using Standard `venv` (Fallback)

```bash
cd agents/backend
source scripts/activate_python_env.sh
cargo check -p backend_service
```

**Note**: Scripts automatically use `uv` when available, falling back to standard `venv` for compatibility.

## Detailed Setup

### Prerequisites

- Python 3.12 installed (via Homebrew: `brew install python@3.12`)
- Located at `/usr/local/bin/python3.12`

### Automatic Setup

The activation script handles everything:

```bash
cd agents/backend
source scripts/activate_python_env.sh
```

**What it does:**

1. Checks for Python 3.12
2. Creates `.venv/` if it doesn't exist
3. Activates the virtual environment
4. Sets `PYO3_PYTHON` environment variable
5. Installs Python dependencies (if needed)

### Manual Setup

#### Using `uv` (Recommended)

```bash
cd agents/backend

# Install uv if not already installed

pip install uv

# Create venv with uv (faster than standard venv)

uv venv .venv --python python3.12

# Activate venv

source .venv/bin/activate

# Set PyO3 to use this Python

export PYO3_PYTHON="$(which python)"
export PYO3_PYTHON_VERSION="3.12"

# Install dependencies with uv (faster)

uv pip install -r requirements.txt

# Verify

python --version  # Should show Python 3.12.x
echo $PYO3_PYTHON  # Should show path to .venv/bin/python
```

#### Using Standard `venv` (Fallback)

```bash
cd agents/backend

# Create venv with Python 3.12

/usr/local/bin/python3.12 -m venv .venv

# Activate venv

source .venv/bin/activate

# Set PyO3 to use this Python

export PYO3_PYTHON="$(which python)"
export PYO3_PYTHON_VERSION="3.12"

# Install dependencies

pip install -r requirements.txt

# Verify

python --version  # Should show Python 3.12.x
echo $PYO3_PYTHON  # Should show path to .venv/bin/python
```

### Using direnv (Optional)

If you have `direnv` installed:

```bash

# Install direnv

brew install direnv

# Add to shell config (~/.zshrc or ~/.bashrc)

echo 'eval "$(direnv hook zsh)"' >> ~/.zshrc

# Allow direnv in backend directory

cd agents/backend
direnv allow
```

The `.envrc` file will automatically:

- Activate the virtual environment
- Set PyO3 environment variables

## Verification

After activation, verify the setup:

```bash

# Check Python version

python --version

# Should show: Python 3.12.x

# Check PyO3 environment variable

echo $PYO3_PYTHON

# Should show: /path/to/agents/backend/.venv/bin/python

# Test Rust build

cargo check -p backend_service

# Should succeed without pyo3-ffi errors
```

## Troubleshooting

### Python 3.12 Not Found

```bash

# Install Python 3.12

brew install python@3.12

# Verify installation

/usr/local/bin/python3.12 --version
```

### PyO3 Still Using System Python

```bash

# Explicitly set PYO3_PYTHON

export PYO3_PYTHON="$(which python)"

# Verify

echo $PYO3_PYTHON

# Should point to .venv/bin/python, not system python
```

### Virtual Environment Issues

```bash

# Remove and recreate

cd agents/backend
rm -rf .venv
source scripts/activate_python_env.sh
```

### Cargo Still Fails

```bash

# Clean build cache

cargo clean

# Rebuild with correct Python

source scripts/activate_python_env.sh
cargo check -p backend_service
```

## Why This Is Needed

- **PyO3 0.21** supports Python 3.8-3.12
- **System Python 3.14** is too new
- **Solution**: Use Python 3.12 in a virtual environment **or** use the forward-compat option below

### Building with system Python 3.14+ (no venv)

If you only need to build the Rust backend (e.g. `cargo build`, `cargo test`) and do not need to run the PyO3/Python bridge, the project sets **`PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1`** in `agents/backend/.cargo/config.toml`. That lets PyO3 build against Python 3.14+ using the stable ABI, so you can run `cargo build` without activating a Python 3.12 venv.

- **When to use**: Local Rust-only builds, CI that doesn't run the Python bridge.
- **When to use a 3.12 venv**: Running the active Python integration/TUI stack, or if you see ABI/runtime issues.
- **Removing it**: When upgrading to a PyO3 version that supports your system Python, you can delete the `[env]` block from `agents/backend/.cargo/config.toml`.

## Modern Tooling: `uv` Support

The project now supports `uv` (fast Python package manager) with automatic fallback to standard tools.

### Benefits of `uv`

- **10-100x faster** package installation
- **Faster virtual environment creation**
- **Better dependency resolution**
- **Compatible** with existing `requirements.txt` files

### Installation

```bash

# Install uv

pip install uv

# Or via Homebrew (macOS)

brew install uv

# Verify installation

uv --version
```

### Usage

```bash

# Create virtual environment (faster)

uv venv .venv

# Install packages (faster)

uv pip install -r requirements.txt

# Or use uv sync with pyproject.toml

uv sync
```

**Note**: All project scripts automatically use `uv` when available, falling back to standard `venv` and `pip` for compatibility.

## Future Options

1. **Upgrade PyO3**: When PyO3 0.22+ supports Python 3.14 (then remove `PYO3_USE_ABI3_FORWARD_COMPATIBILITY` from `agents/backend/.cargo/config.toml`).
2. **Use Poetry**: Could manage Python version via `pyproject.toml`
3. **Use pyenv**: Could manage multiple Python versions
4. **Full `uv` Migration**: Consider migrating to `pyproject.toml` with `uv sync`

For now, the virtual environment approach with `uv` support is the simplest and most reliable. For Rust-only builds, the `.cargo/config.toml` forward-compat setting avoids needing a 3.12 venv.

## Integration with CI/CD

For CI/CD pipelines, ensure Python 3.12 is available. Using `uv` is recommended for faster builds:

```yaml

# Example GitHub Actions with uv (Recommended)
- name: Install uv
  run: pip install uv

- name: Set up Python 3.12
  uses: actions/setup-python@v4
  with:
    python-version: '3.12'

- name: Create virtual environment and install dependencies
  run: |
    uv venv .venv
    source .venv/bin/activate
    uv pip install -r requirements.txt
    export PYO3_PYTHON="$(which python)"
```

**Fallback** (without `uv`):

```yaml

# Example GitHub Actions (Standard venv)
- name: Set up Python 3.12
  uses: actions/setup-python@v4
  with:
    python-version: '3.12'

- name: Create virtual environment
  run: |
    python -m venv .venv
    source .venv/bin/activate
    pip install -r requirements.txt
    export PYO3_PYTHON="$(which python)"
```

## References

- [PyO3 Documentation](https://pyo3.rs/)
- [PyO3 Python Version Support](https://pyo3.rs/latest/building_and_distribution.html#python-version)
- [Python Virtual Environments](https://docs.python.org/3/tutorial/venv.html)
- [uv Documentation](https://github.com/astral-sh/uv) - Fast Python package manager
- [Project Migration Plan](./PYTHON_UV_MIGRATION_PLAN.md) - Detailed migration guide
- [Standardization Analysis](./PYTHON_VENV_STANDARDIZATION_ANALYSIS.md) - Analysis and recommendations
