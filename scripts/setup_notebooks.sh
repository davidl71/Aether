#!/bin/bash
# Setup script for Jupyter notebooks environment

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VENV_DIR="${PROJECT_ROOT}/.cache/venvs/notebooks"

export XDG_CACHE_HOME="${XDG_CACHE_HOME:-$PROJECT_ROOT/.cache}"
export UV_CACHE_DIR="${UV_CACHE_DIR:-$XDG_CACHE_HOME/uv}"
export PIP_CACHE_DIR="${PIP_CACHE_DIR:-$XDG_CACHE_HOME/pip}"
export JUPYTER_CONFIG_DIR="${JUPYTER_CONFIG_DIR:-$XDG_CACHE_HOME/jupyter}"
mkdir -p "$XDG_CACHE_HOME" "$UV_CACHE_DIR" "$PIP_CACHE_DIR" "$JUPYTER_CONFIG_DIR"

echo "📓 Setting up Jupyter notebooks environment..."
echo ""

# Check Python version
python_version=$(python3 --version 2>&1 | awk '{print $2}')
echo "Python version: $python_version"

# Install notebook dependencies
echo ""
echo "Installing notebook dependencies..."
if command -v uv >/dev/null 2>&1; then
    uv venv "$VENV_DIR"
    uv pip install --python "$VENV_DIR/bin/python" -r "$PROJECT_ROOT/requirements-notebooks.txt"
    JUPYTER_BIN="$VENV_DIR/bin/jupyter"
else
    python3 -m venv "$VENV_DIR"
    "$VENV_DIR/bin/python" -m pip install --upgrade pip
    "$VENV_DIR/bin/python" -m pip install -r "$PROJECT_ROOT/requirements-notebooks.txt"
    JUPYTER_BIN="$VENV_DIR/bin/jupyter"
fi

# Create output directory
mkdir -p "$PROJECT_ROOT/notebooks/output"

# Check for JupyterLab
if [ -x "$JUPYTER_BIN" ]; then
    echo ""
    echo "✅ Jupyter installed"
    "$JUPYTER_BIN" --version
else
    echo "❌ Jupyter not found in $VENV_DIR"
    exit 1
fi

# Generate Jupyter config if it doesn't exist
if [ ! -f "$JUPYTER_CONFIG_DIR/jupyter_lab_config.py" ]; then
    echo ""
    echo "Generating JupyterLab config..."
    "$JUPYTER_BIN" lab --generate-config
fi

echo ""
echo "✅ Notebook environment setup complete!"
echo ""
echo "To start JupyterLab:"
echo "  cd $PROJECT_ROOT"
echo "  ${JUPYTER_BIN} lab"
echo ""
echo "Or use VS Code with Jupyter extension:"
echo "  code notebooks/"
echo ""
echo "See notebooks/README.md for usage instructions."
