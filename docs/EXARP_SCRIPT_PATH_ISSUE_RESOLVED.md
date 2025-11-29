# Exarp Script Path Issue - Resolution

**Date**: 2025-11-29  
**Status**: Workaround Implemented

---

## Problem Summary

Exarp daily automation (`mcp_exarp_run_daily_automation`) reports "Script not found" for all tasks, even though:
- Scripts exist in `scripts/` directory
- Scripts are executable
- Scripts can be run directly

---

## Root Cause Analysis

### Issue
Exarp runs as an MCP server via `uvx exarp --mcp`, which means:
1. **Different Execution Context**: MCP server runs in its own context, not the project directory
2. **Script Discovery**: Exarp may look for scripts in the Exarp package itself, not the project
3. **Path Resolution**: Script paths may be resolved relative to MCP server location, not project root

### Scripts Expected
Exarp daily automation expects these scripts:
- `scripts/automate_docs_health_v2.py`
- `scripts/automate_todo2_alignment_v2.py`
- `scripts/automate_todo2_duplicate_detection.py`

### Scripts Created ✅
All three scripts have been created with:
- Proper shebang (`#!/usr/bin/env python3`)
- Executable permissions
- Fallback implementations
- Error handling

---

## Solution Implemented

### Option 1: Use Individual MCP Tools (Recommended) ✅

**Status**: Working perfectly

Instead of using the aggregate `mcp_exarp_run_daily_automation`, call tools individually:

```python
# Documentation health
mcp_exarp_check_documentation_health()

# Todo2 alignment
mcp_exarp_analyze_todo2_alignment()

# Duplicate detection
mcp_exarp_detect_duplicate_tasks(auto_fix=True)
```

**Pros**:
- ✅ Works immediately
- ✅ No script path issues
- ✅ Direct access to results
- ✅ Can be called from any context

**Cons**:
- ❌ Requires manual orchestration
- ❌ Not using daily automation aggregate feature

---

### Option 2: Use Our Daily Automation Script ✅

**Status**: Working perfectly

Use our custom daily automation script:

```bash
./scripts/daily_automation_with_link_fixing.sh
```

This script runs:
1. Documentation link fixing (apply mode)
2. Documentation format validation
3. Shared TODO table synchronization

**Pros**:
- ✅ Works immediately
- ✅ Includes additional automation (link fixing)
- ✅ Can be scheduled via cron
- ✅ Comprehensive logging

---

### Option 3: Scripts Created (For Future Use)

**Status**: Scripts exist but Exarp can't find them

Created scripts:
- `scripts/automate_docs_health_v2.py` - Documentation health wrapper
- `scripts/automate_todo2_alignment_v2.py` - Alignment analysis wrapper
- `scripts/automate_todo2_duplicate_detection.py` - Duplicate detection wrapper

**Implementation**:
- Try to import Exarp package functions
- Fallback to `uvx exarp` subprocess calls
- Graceful error handling

**Note**: These scripts work when executed directly but Exarp daily automation still can't find them.

---

## Configuration Attempts

### Created Config File
Created `.exarp/config.json` with script paths:
```json
{
  "scripts": {
    "docs_health": "scripts/automate_docs_health_v2.py",
    "todo2_alignment": "scripts/automate_todo2_alignment_v2.py",
    "duplicate_detection": "scripts/automate_todo2_duplicate_detection.py"
  }
}
```

**Status**: Config file created but Exarp may not use it (unknown if Exarp reads this config)

---

## Current Status

### What Works ✅
1. **Individual MCP Tools**: All work perfectly
   - `mcp_exarp_check_documentation_health()` ✅
   - `mcp_exarp_analyze_todo2_alignment()` ✅
   - `mcp_exarp_detect_duplicate_tasks()` ✅

2. **Our Daily Script**: Works perfectly
   - `./scripts/daily_automation_with_link_fixing.sh` ✅

3. **Scripts Exist**: All scripts created and executable ✅

### What Doesn't Work ❌
1. **Exarp Daily Automation Aggregate**: Still reports "Script not found"
   - Likely an Exarp internal issue with script discovery
   - May require Exarp package modification or configuration

---

## Recommendations

### Immediate Solution
**Use individual MCP tools** or **our daily automation script**:
- Both work perfectly
- No script path issues
- Can be automated via cron or CI/CD

### Long-Term Solution
1. **Investigate Exarp Source**: Check how Exarp discovers scripts
2. **Contact Exarp Maintainer**: Report script discovery issue
3. **Use MCP Tools**: Continue using individual tools (they work great)
4. **Monitor Exarp Updates**: Check if future versions fix the issue

---

## Files Created

### Scripts
- `scripts/automate_docs_health_v2.py` - Documentation health wrapper
- `scripts/automate_todo2_alignment_v2.py` - Alignment analysis wrapper
- `scripts/automate_todo2_duplicate_detection.py` - Duplicate detection wrapper

### Configuration
- `.exarp/config.json` - Script paths configuration (may not be used by Exarp)

### Documentation
- `docs/EXARP_SCRIPT_PATH_FIX.md` - Initial investigation
- `docs/EXARP_SCRIPT_PATH_ISSUE_RESOLVED.md` - This document

---

## Verification

### Test Individual Tools
```python
# All work perfectly
mcp_exarp_check_documentation_health()  # ✅
mcp_exarp_analyze_todo2_alignment()     # ✅
mcp_exarp_detect_duplicate_tasks()      # ✅
```

### Test Daily Script
```bash
./scripts/daily_automation_with_link_fixing.sh  # ✅
```

### Test Scripts Directly
```bash
python3 scripts/automate_docs_health_v2.py .  # ✅ Works
python3 scripts/automate_todo2_alignment_v2.py .  # ✅ Works
python3 scripts/automate_todo2_duplicate_detection.py .  # ✅ Works
```

### Test Exarp Daily Automation
```python
mcp_exarp_run_daily_automation()  # ❌ Still reports "Script not found"
```

---

## Conclusion

**Status**: Workaround implemented and working

While Exarp daily automation aggregate tool has script discovery issues, we have:
1. ✅ Working individual MCP tools
2. ✅ Working custom daily automation script
3. ✅ Scripts created for future use
4. ✅ Comprehensive documentation

**Recommendation**: Continue using individual MCP tools or our custom daily script. The aggregate daily automation tool can be used once Exarp fixes the script discovery issue.

---

**Last Updated**: 2025-11-29  
**Status**: Workaround implemented, issue documented
