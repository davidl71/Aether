# Installation Guide

**Quick Fix for Import Errors**

---

## Issue

When running the server directly, you may see:
- `Error handling module not available`
- `Some tools not available: attempted relative import with no known parent package`
- `MCP not installed`

---

## Solution

### Step 1: Install MCP Package

```bash
pip3 install mcp
```

Or if using a virtual environment:
```bash
python3 -m pip install mcp
```

### Step 2: Verify Installation

```bash
python3 -c "import mcp; print('MCP installed:', mcp.__version__ if hasattr(mcp, '__version__') else 'yes')"
```

### Step 3: Test Server Import

```bash
cd mcp-servers/project-management-automation
python3 -c "
import sys
sys.path.insert(0, '.')
sys.path.insert(0, '../..')
from error_handler import ErrorCode
from tools.docs_health import check_documentation_health
print('✅ All imports work')
"
```

---

## Alternative: Run as Module

If direct execution has import issues, you can run as a module:

```bash
cd /path/to/project
python3 -m mcp_servers.project_management_automation.server
```

But this requires proper package structure. The direct file execution approach is simpler.

---

## Troubleshooting

### Import Errors

If you see relative import errors:
1. Ensure `server.py` is being run from the correct directory
2. Check that all `__init__.py` files exist
3. Verify Python path includes project root

### MCP Not Found

If MCP package is not found:
1. Install: `pip3 install mcp`
2. Verify: `python3 -c "import mcp"`
3. Check Python version: `python3 --version` (needs 3.9+)

---

**After installation, restart Cursor completely to activate the MCP server.**
