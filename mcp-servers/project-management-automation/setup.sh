#!/bin/bash
# Quick setup script for MCP server

set -e

echo "🔧 Setting up Project Management Automation MCP Server..."
echo ""

# Check Python version
echo "📋 Checking Python version..."
python3 --version

# Install MCP package
echo ""
echo "📦 Installing MCP package..."
pip3 install mcp

# Verify installation
echo ""
echo "✅ Verifying installation..."
python3 -c "import mcp; print('MCP version:', getattr(mcp, '__version__', 'installed'))" || echo "MCP installed but version not available"

# Test server import
echo ""
echo "🧪 Testing server imports..."
cd "$(dirname "$0")"
python3 -c "
import sys
sys.path.insert(0, '.')
sys.path.insert(0, '../..')
try:
    from error_handler import ErrorCode
    from tools.docs_health import check_documentation_health
    print('✅ All imports successful')
except Exception as e:
    print(f'❌ Import error: {e}')
    sys.exit(1)
"

echo ""
echo "🎉 Setup complete!"
echo ""
echo "Next steps:"
echo "1. Restart Cursor completely"
echo "2. Check Cursor Settings → MCP Servers"
echo "3. Verify 'project-management-automation' appears in the list"
echo ""
