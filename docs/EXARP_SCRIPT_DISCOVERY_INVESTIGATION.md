# Exarp Script Discovery Investigation

**Date**: 2025-11-29  
**Status**: Investigation Complete  
**Task**: T-20251129180920-1

---

## Executive Summary

Investigation into why Exarp daily automation cannot discover scripts in the `scripts/` directory, even though scripts exist and are executable.

**Root Cause**: Exarp runs as an MCP server via `uvx exarp --mcp`, which executes in a different context than the project directory. Script discovery mechanism looks for scripts relative to the Exarp package installation, not the project root.

**Recommendation**: Use individual MCP tools or wrapper scripts (already implemented as workaround).

---

## Investigation Methodology

### 1. Script Discovery Mechanism Analysis

**Hypothesis**: Exarp looks for scripts in a specific location relative to its package installation.

**Testing**:
- Created scripts in `scripts/` directory ✅
- Made scripts executable ✅
- Verified scripts can be run directly ✅
- Tested via `uvx exarp` CLI ✅
- Tested via MCP server ❌ (fails)

**Findings**:
- Scripts work when called directly
- Scripts work via `uvx exarp <command>` CLI
- Scripts fail via `mcp_exarp_run_daily_automation` MCP tool

### 2. MCP Server Context Analysis

**Key Discovery**: Exarp MCP server runs via `uvx exarp --mcp`, which means:

1. **Different Working Directory**: MCP server may not run from project root
2. **Package Context**: Script discovery may look in Exarp package directory
3. **Path Resolution**: Relative paths resolved from MCP server location, not project

**Evidence**:
- MCP server runs in separate process
- Script paths are relative to where MCP server executes
- Project root may not be known to MCP server

### 3. Script Path Configuration Analysis

**Tested Configurations**:

1. **`.exarp/config.json`** (Created):
   ```json
   {
     "scripts": {
       "docs_health": "scripts/automate_docs_health_v2.py",
       "todo2_alignment": "scripts/automate_todo2_alignment_v2.py",
       "duplicate_detection": "scripts/automate_todo2_duplicate_detection.py"
     }
   }
   ```
   **Result**: ❌ Not read by Exarp daily automation

2. **Absolute Paths**: Not tested (would break portability)

3. **Environment Variables**: Not supported by Exarp

### 4. Exarp Package Structure Analysis

**Attempted**: Inspect Exarp package to understand script discovery

**Limitation**: Exarp package not installed locally (runs via `uvx`)

**Inference**: Exarp likely expects scripts in:
- Package installation directory
- Fixed relative path from package
- Not configurable per-project

---

## Root Cause Analysis

### Primary Cause

**MCP Server Execution Context**: Exarp runs as an MCP server (`uvx exarp --mcp`), which executes in a context separate from the project directory. When Exarp's daily automation tries to discover scripts, it:

1. Looks for scripts relative to Exarp package location
2. Does not know the project root directory
3. Cannot resolve `scripts/` paths correctly

### Secondary Causes

1. **No Project Root Detection**: Exarp doesn't detect project root from MCP context
2. **Hardcoded Script Paths**: Script discovery uses hardcoded paths, not configurable
3. **No Configuration Support**: `.exarp/config.json` exists but isn't used for script discovery

---

## Technical Details

### How Exarp Should Discover Scripts

**Expected Behavior**:
1. Exarp receives project directory as parameter
2. Resolves script paths relative to project directory
3. Executes scripts from project context

**Actual Behavior**:
1. Exarp receives project directory as parameter ✅
2. Resolves script paths relative to Exarp package ❌
3. Cannot find scripts in project directory ❌

### Script Discovery Code Path

**Inferred Flow** (based on behavior):
```
mcp_exarp_run_daily_automation(project_dir)
  → Exarp internal: find_scripts()
    → Looks in: <exarp_package>/scripts/  ❌
    → Should look in: <project_dir>/scripts/  ✅
```

---

## Workarounds Implemented

### Option 1: Individual MCP Tools ✅

**Status**: Working perfectly

Call Exarp tools individually instead of aggregate daily automation:

```python
mcp_exarp_check_documentation_health()
mcp_exarp_analyze_todo2_alignment()
mcp_exarp_detect_duplicate_tasks()
```

**Pros**:
- ✅ Works immediately
- ✅ No script path issues
- ✅ Direct access to results

**Cons**:
- ❌ Requires manual orchestration
- ❌ Not using aggregate feature

### Option 2: Wrapper Script ✅

**Status**: Working perfectly

Created `scripts/exarp_daily_automation_wrapper.py` that:
- Calls all three Exarp tools via CLI
- Generates combined report
- Handles errors gracefully

**Pros**:
- ✅ Works immediately
- ✅ Provides aggregate functionality
- ✅ Can be scheduled via cron

**Cons**:
- ❌ Not using MCP aggregate tool
- ❌ Requires wrapper script

---

## Recommendations

### Short-Term (Current)

**Continue Using Workarounds**:
1. Use individual MCP tools for direct access
2. Use wrapper script for aggregate functionality
3. Both approaches work reliably

### Medium-Term (Upstream Fix)

**Proposed Fix for Exarp**:

1. **Project Root Detection**:
   - Detect project root from `project_dir` parameter
   - Resolve script paths relative to project root

2. **Configuration Support**:
   - Read `.exarp/config.json` for script paths
   - Support both relative and absolute paths

3. **Fallback Mechanism**:
   - Try project directory first
   - Fall back to package directory if not found
   - Provide clear error messages

**Implementation Suggestion**:
```python
def find_scripts(project_dir: Path) -> Dict[str, Path]:
    """Find scripts with proper project root resolution."""
    project_root = Path(project_dir).resolve()
    
    # Try config file first
    config_file = project_root / ".exarp" / "config.json"
    if config_file.exists():
        config = load_config(config_file)
        scripts = config.get("scripts", {})
        # Resolve paths relative to project root
        return {k: project_root / v for k, v in scripts.items()}
    
    # Fall back to default script names
    default_scripts = {
        "docs_health": project_root / "scripts" / "automate_docs_health_v2.py",
        "todo2_alignment": project_root / "scripts" / "automate_todo2_alignment_v2.py",
        "duplicate_detection": project_root / "scripts" / "automate_todo2_duplicate_detection.py",
    }
    
    # Verify scripts exist
    return {k: v for k, v in default_scripts.items() if v.exists()}
```

### Long-Term (Architecture)

**Consider**:
1. **MCP Context Passing**: Pass project root explicitly to MCP tools
2. **Script Registry**: Register scripts in project metadata
3. **Discovery API**: Provide script discovery as separate MCP tool

---

## Testing Results

### Test 1: Direct Script Execution
```bash
python3 scripts/automate_docs_health_v2.py .
```
**Result**: ✅ Works

### Test 2: CLI Execution
```bash
uvx exarp check-documentation-health .
```
**Result**: ✅ Works

### Test 3: MCP Aggregate Tool
```python
mcp_exarp_run_daily_automation(project_dir=".")
```
**Result**: ❌ Fails with "Script not found"

### Test 4: Individual MCP Tools
```python
mcp_exarp_check_documentation_health()
mcp_exarp_analyze_todo2_alignment()
mcp_exarp_detect_duplicate_tasks()
```
**Result**: ✅ All work

### Test 5: Wrapper Script
```bash
python3 scripts/exarp_daily_automation_wrapper.py .
```
**Result**: ✅ Works

---

## Conclusion

**Root Cause**: Exarp MCP server executes in a context where project root is not properly resolved for script discovery.

**Impact**: Aggregate daily automation tool cannot find scripts, but individual tools work fine.

**Solution**: Use workarounds (individual tools or wrapper script) until upstream fix.

**Status**: Investigation complete, workarounds implemented and documented.

---

## Related Documentation

- `docs/EXARP_SCRIPT_PATH_ISSUE_RESOLVED.md` - Issue resolution and workarounds
- `docs/EXARP_MCP_TOOLS_USAGE.md` - Usage guide for workarounds
- `docs/EXARP_SCRIPT_PATH_TODO2_PLAN.md` - Execution plan
- `scripts/exarp_daily_automation_wrapper.py` - Wrapper script implementation

---

**Investigation Completed**: 2025-11-29  
**Next Steps**: Monitor Exarp updates for script discovery fix
