# Exarp Script Path Fix

**Date**: 2025-11-29
**Issue**: Exarp daily automation reports "Script not found" for all tasks
**Status**: Investigating

---

## Problem

Exarp daily automation (`mcp_exarp_run_daily_automation`) is skipping all tasks with "Script not found" errors:

- Documentation Health Check: Script not found
- Todo2 Alignment Analysis: Script not found
- Duplicate Task Detection: Script not found

---

## Investigation

### What Exarp Expects

Exarp daily automation looks for these scripts in the project:

1. `scripts/automate_docs_health_v2.py` - Documentation health check
2. `scripts/automate_todo2_alignment_v2.py` - Todo2 alignment analysis
3. `scripts/automate_todo2_duplicate_detection.py` - Duplicate detection

### Current Status

✅ **Scripts Created**: All three placeholder scripts created
❌ **Still Not Found**: Exarp still reports "Script not found"

### Possible Causes

1. **Script Location**: Exarp might look in a different directory
2. **Script Format**: Scripts might need specific structure or imports
3. **Python Path**: Scripts might need to be importable modules
4. **Configuration**: Exarp might need a config file specifying script paths
5. **MCP vs Local**: Exarp might expect to use MCP tools instead of local scripts

---

## Solutions Attempted

### Solution 1: Create Placeholder Scripts ✅

Created three placeholder scripts:

- `scripts/automate_docs_health_v2.py`
- `scripts/automate_todo2_alignment_v2.py`
- `scripts/automate_todo2_duplicate_detection.py`

**Result**: Scripts created but still not found by Exarp

---

## Recommended Solutions

### Option 1: Use MCP Tools Directly (Current Workaround) ✅

**Status**: Working
**Approach**: Call Exarp MCP tools directly instead of using daily automation

```python

# Instead of: mcp_exarp_run_daily_automation
# Use:

mcp_exarp_check_documentation_health()
mcp_exarp_analyze_todo2_alignment()
mcp_exarp_detect_duplicate_tasks()
```

**Pros**:

- ✅ Works immediately
- ✅ No script path issues
- ✅ Direct access to tools

**Cons**:

- ❌ Requires manual orchestration
- ❌ Not using daily automation feature

---

### Option 2: Investigate Exarp Source Code

**Action**: Check Exarp package source to understand:

- How it finds scripts
- What directory structure it expects
- What configuration it needs

**Command**:

```bash

# Find Exarp installation

python3 -c "import exarp_project_management; print(exarp_project_management.__file__)"

# Or check uvx cache

ls ~/.cache/uv/archive-v0/*/lib/exarp_project_management/
```

---

### Option 3: Create Proper Script Implementations

**Action**: Implement scripts that actually call Exarp functionality

**Requirements**:

- Scripts must be executable
- Must accept project directory as argument
- Must return proper exit codes
- Should call Exarp's internal functions or MCP tools

**Example**:

```python

#!/usr/bin/env python3

import sys
from pathlib import Path

# Try to import Exarp functions

try:
    from exarp_project_management.scripts import automate_docs_health_v2
    automate_docs_health_v2.main()
except ImportError:
    # Fallback: Use MCP tools or direct implementation
    pass
```

---

### Option 4: Configure Exarp Script Paths

**Action**: Create configuration file telling Exarp where to find scripts

**Possible Config Locations**:

- `.exarp/config.json`
- `exarp_config.json`
- `scripts/exarp_config.json`

**Possible Config Format**:

```json
{
  "scripts": {
    "docs_health": "scripts/automate_docs_health_v2.py",
    "todo2_alignment": "scripts/automate_todo2_alignment_v2.py",
    "duplicate_detection": "scripts/automate_todo2_duplicate_detection.py"
  }
}
```

---

## Current Workaround

**Use Individual MCP Tools**:

Instead of `mcp_exarp_run_daily_automation`, call tools individually:

```python

# Documentation health

mcp_exarp_check_documentation_health()

# Todo2 alignment

mcp_exarp_analyze_todo2_alignment()

# Duplicate detection

mcp_exarp_detect_duplicate_tasks(auto_fix=True)
```

**Or Use Our Daily Automation Script**:

```bash
./scripts/daily_automation_with_link_fixing.sh
```

This script runs:

1. Documentation link fixing
2. Format validation
3. TODO sync

---

## Next Steps

1. **Investigate Exarp Source**: Check how Exarp finds scripts
2. **Test Script Execution**: Verify scripts can be executed directly
3. **Check Configuration**: Look for Exarp config files
4. **Contact Exarp Maintainer**: If needed, ask about script path requirements
5. **Use Workaround**: Continue using individual MCP tools or our daily script

---

## Files Created

- `scripts/automate_docs_health_v2.py` - Placeholder script
- `scripts/automate_todo2_alignment_v2.py` - Placeholder script
- `scripts/automate_todo2_duplicate_detection.py` - Placeholder script
- `docs/EXARP_SCRIPT_PATH_FIX.md` - This documentation

---

**Last Updated**: 2025-11-29
**Status**: Scripts created, but Exarp still can't find them. Investigation ongoing.
