# Python Environment Setup for PyO3 Compatibility

**Issue**: PyO3 0.21 supports Python up to 3.12, but system Python is 3.14
**Solution**: Use Python 3.12 virtual environment

## Quick Start

```bash
cd agents/backend
source scripts/activate_python_env.sh
cargo check -p backend_service
```

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

If you prefer manual setup:

```bash
cd agents/backend

# Create venv with Python 3.12
/usr/local/bin/python3.12 -m venv .venv

# Activate venv
source .venv/bin/activate

# Set PyO3 to use this Python
export PYO3_PYTHON="$(which python)"
export PYO3_PYTHON_VERSION="3.12"

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
- **Solution**: Use Python 3.12 in a virtual environment

## Future Options

1. **Upgrade PyO3**: When PyO3 0.22+ supports Python 3.14
2. **Use Poetry**: Could manage Python version via `pyproject.toml`
3. **Use pyenv**: Could manage multiple Python versions

For now, the virtual environment approach is the simplest and most reliable.

## Integration with CI/CD

For CI/CD pipelines, ensure Python 3.12 is available:

```yaml
# Example GitHub Actions
- name: Set up Python 3.12
  uses: actions/setup-python@v4
  with:
    python-version: '3.12'

- name: Create virtual environment
  run: |
    python -m venv .venv
    source .venv/bin/activate
    export PYO3_PYTHON="$(which python)"
```

## References

- [PyO3 Documentation](https://pyo3.rs/)
- [PyO3 Python Version Support](https://pyo3.rs/latest/building_and_distribution.html#python-version)
- [Python Virtual Environments](https://docs.python.org/3/tutorial/venv.html)
