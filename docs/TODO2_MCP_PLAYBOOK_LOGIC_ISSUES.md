# Todo2 MCP Playbook Logic Issues and Fixes

**Date**: 2025-12-24
**Status**: ✅ Issues Identified and Fixed

## Issues Found

### 1. ❌ Missing Error Handling for JSON Parsing

**Location**: Line 54-57 (Parse existing MCP configuration)

**Problem**:

- If `mcp.json` has invalid JSON, `from_json` filter will fail
- Playbook will stop with error
- No graceful fallback

**Fix Applied**: ✅

- Added `block/rescue` structure
- Falls back to empty config if JSON invalid
- Displays warning message

### 2. ❌ Missing Error Handling for Deduplication Script

**Location**: Line 76-84 (Remove duplicate MCP servers)

**Problem**:

- If Python script fails, `stdout` might not be valid JSON
- `from_json` filter would fail in later tasks
- Playbook would stop

**Fix Applied**: ✅

- Added `failed_when: false` to prevent playbook failure
- Added `rc == 0` check before using output
- Added `default(0)` for count query
- Safe variable access throughout

### 3. ⚠️ Variable Undefined Edge Case

**Location**: Multiple tasks referencing `deduplicated_result`

**Problem**:

- If deduplication task is skipped, variable might not be registered
- Accessing `.changed` on undefined variable could fail

**Fix Applied**: ✅

- All accesses use `default(false)` pattern
- Task only runs when needed (file exists + has servers)
- Safe fallback values used

### 4. ✅ Write Condition Logic (Verified Correct)

**Location**: Line 123-124

**Test Results**: All scenarios work correctly

- File doesn't exist → Writes ✅
- Todo2 not configured → Writes ✅
- Todo2 configured, no dupes → Doesn't write ✅
- Todo2 configured, has dupes → Writes ✅

## Logic Flow Verification

### Scenario 1: New Project (No mcp.json)

1. ✅ Check Todo2 directory → Found
2. ✅ Create `.cursor/` directory
3. ✅ Create empty `mcp.json` with `{}`
4. ✅ Initialize `mcp_config = {'mcpServers': {}}`
5. ⏭️ Skip deduplication (no file existed, now empty)
6. ✅ Check Todo2 → Not configured
7. ✅ Add Todo2 to config
8. ✅ Write config → **Writes** (correct)

### Scenario 2: Existing Project (Has mcp.json, No Todo2)

1. ✅ Check Todo2 directory → Found
2. ✅ `.cursor/` exists
3. ✅ Read existing `mcp.json`
4. ✅ Parse JSON → Success
5. ✅ Run deduplication (if has servers)
6. ✅ Check Todo2 → Not configured
7. ✅ Add Todo2 to config
8. ✅ Write config → **Writes** (correct)

### Scenario 3: Existing Project (Has Todo2, Has Duplicates)

1. ✅ Check Todo2 directory → Found
2. ✅ Read existing `mcp.json`
3. ✅ Parse JSON → Success
4. ✅ Run deduplication → Finds duplicates
5. ✅ Update config with deduplicated servers
6. ✅ Check Todo2 → Already configured
7. ⏭️ Skip adding Todo2
8. ✅ Write config → **Writes** (to save deduplication) ✅

### Scenario 4: Existing Project (Has Todo2, No Duplicates)

1. ✅ Check Todo2 directory → Found
2. ✅ Read existing `mcp.json`
3. ✅ Parse JSON → Success
4. ✅ Run deduplication → No duplicates found
5. ✅ Check Todo2 → Already configured
6. ⏭️ Skip adding Todo2
7. ⏭️ Skip writing (no changes) → **Doesn't write** ✅

### Scenario 5: Invalid JSON in Existing File

1. ✅ Check Todo2 directory → Found
2. ✅ Read existing `mcp.json` (invalid JSON)
3. ❌ Parse JSON → **Fails**
4. ✅ Rescue block → Sets empty config
5. ✅ Display warning
6. ✅ Continue with empty config
7. ✅ Add Todo2
8. ✅ Write config → **Writes** (correct)

## Test Results

### Deduplication Script ✅

```
Test: Remove 2 duplicates
Result: ✅ Correct
  - Removed: ['fs (duplicate of filesystem)', 'git2 (duplicate of git)']
  - Remaining: ['filesystem', 'git', 'unique']
```

### Write Condition Logic ✅

All 5 test cases pass:

- ✅ File doesn't exist → Writes
- ✅ Todo2 not configured, no dupes → Writes
- ✅ Todo2 not configured, has dupes → Writes
- ✅ Todo2 configured, no dupes → Doesn't write
- ✅ Todo2 configured, has dupes → Writes

## Summary

### ✅ Logic is Correct

**All scenarios handled:**

- New projects
- Existing projects
- Invalid JSON
- Script failures
- Duplicate removal
- Todo2 configuration

### ✅ Error Handling Added

- JSON parsing errors → Graceful fallback
- Script failures → Safe defaults
- Undefined variables → Safe access patterns

### ✅ Ready for Production

The playbook is **safe and correct** to run on all projects.

---

**Last Updated**: 2025-12-24
**Status**: Logic Verified, Issues Fixed, Production Ready
