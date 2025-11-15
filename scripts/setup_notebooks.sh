#!/bin/bash
# Setup script for Jupyter notebooks environment

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "📓 Setting up Jupyter notebooks environment..."
echo ""

# Check Python version
python_version=$(python3 --version 2>&1 | awk '{print $2}')
echo "Python version: $python_version"

# Install notebook dependencies
echo ""
echo "Installing notebook dependencies..."
pip install -r "$PROJECT_ROOT/requirements-notebooks.txt"

# Create output directory
mkdir -p "$PROJECT_ROOT/notebooks/output"

# Check for JupyterLab
if command -v jupyter &> /dev/null; then
    echo ""
    echo "✅ Jupyter installed"
    jupyter --version
else
    echo "❌ Jupyter not found. Install with: pip install jupyter jupyterlab"
    exit 1
fi

# Generate Jupyter config if it doesn't exist
JUPYTER_CONFIG_DIR="$HOME/.jupyter"
if [ ! -f "$JUPYTER_CONFIG_DIR/jupyter_lab_config.py" ]; then
    echo ""
    echo "Generating JupyterLab config..."
    jupyter lab --generate-config
fi

echo ""
echo "✅ Notebook environment setup complete!"
echo ""
echo "To start JupyterLab:"
echo "  cd $PROJECT_ROOT"
echo "  jupyter lab"
echo ""
echo "Or use VS Code with Jupyter extension:"
echo "  code notebooks/"
echo ""
echo "See notebooks/README.md for usage instructions."
