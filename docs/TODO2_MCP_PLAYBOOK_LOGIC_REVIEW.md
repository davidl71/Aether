# Todo2 MCP Playbook Logic Review

**Date**: 2025-12-24
**Status**: ✅ Logic Reviewed and Improved

## Logic Flow Analysis

### 1. Initial Checks ✅

**Task**: Check if project has Todo2 directory
- ✅ Correctly checks for `.todo2/` directory
- ✅ Skips playbook if not found (correct behavior)

### 2. MCP Config File Handling ✅

**Tasks**:
- Create `.cursor/` directory if missing
- Check if `mcp.json` exists
- Initialize empty config if missing
- Read existing config if present

**Logic**: ✅ Correct
- Handles both new and existing files
- Creates empty structure if needed

### 3. Duplicate Removal Logic ✅ (Improved)

**Task**: Remove duplicate MCP servers

**Original Logic:**
- Only runs when file exists AND has servers
- Uses Python script for deduplication
- Updates config if duplicates found

**Issues Found:**
1. ❌ No error handling if script fails
2. ❌ No validation of script output JSON
3. ⚠️ Variable might be undefined if task skipped

**Fixes Applied:**
1. ✅ Added `failed_when: false` to handle script failures gracefully
2. ✅ Added `rc == 0` check before using output
3. ✅ Added `default(0)` for count query
4. ✅ Safe variable access with `default(false)`

### 4. Todo2 Configuration ✅

**Task**: Add Todo2 MCP server

**Logic**: ✅ Correct
- Checks if Todo2 already configured
- Only adds if not present
- Merges with existing servers

### 5. Write Condition Logic ✅ (Verified)

**Task**: Write updated MCP configuration

**Condition**: `not todo2_configured or (deduplicated_result.changed | default(false) | bool)`

**Test Cases:**

| Scenario                          | todo2_configured | deduplicated.changed | Write? | Correct?  |
| --------------------------------- | ---------------- | -------------------- | ------ | --------- |
| File doesn't exist                | False            | undefined → False    | ✅ Yes  | ✅ Correct |
| File exists, no Todo2, no dupes   | False            | False                | ✅ Yes  | ✅ Correct |
| File exists, no Todo2, has dupes  | False            | True                 | ✅ Yes  | ✅ Correct |
| File exists, has Todo2, no dupes  | True             | False                | ❌ No   | ✅ Correct |
| File exists, has Todo2, has dupes | True             | True                 | ✅ Yes  | ✅ Correct |

**Result**: ✅ All test cases pass

## Edge Cases Handled

### 1. Invalid JSON in Existing File ✅ (Fixed)

**Issue**: If `mcp.json` has invalid JSON, `from_json` would fail

**Fix**: Added error handling with `failed_when: false` and rescue block
- Falls back to empty config if JSON invalid
- Displays warning message

### 2. Deduplication Script Failure ✅ (Fixed)

**Issue**: If script fails, `stdout` might not be valid JSON

**Fix**:
- Added `failed_when: false` to prevent playbook failure
- Added `rc == 0` check before using output
- Safe defaults for all variable accesses

### 3. Variable Undefined ✅ (Handled)

**Issue**: `deduplicated_result` might not exist if task skipped

**Fix**:
- Uses `default(false)` for all accesses
- Task only runs when needed (file exists + has servers)
- Safe fallback values throughout

### 4. Empty Config ✅ (Handled)

**Issue**: What if file exists but `mcpServers` is empty?

**Fix**:
- Deduplication task correctly skips (no duplicates to remove)
- Todo2 still gets added
- Write condition works correctly

## Remaining Considerations

### 1. Project Root Detection

**Current**: `playbook_dir/../..`

**Assumption**: Playbook is in `ansible/playbooks/`

**Status**: ✅ Works for current structure
- Could be more robust (check for `.git` or `.todo2`)
- But current approach is fine for intended use

### 2. JSON Validation

**Current**: Relies on Ansible's `to_json` and `from_json` filters

**Status**: ✅ Sufficient
- Ansible handles JSON validation
- Added error handling for edge cases

### 3. Script Path

**Current**: `{{ project_root }}/scripts/deduplicate_mcp_servers.py`

**Status**: ✅ Correct
- Uses absolute path from project root
- Script is executable

## Test Results

### Deduplication Script Test ✅

```
Test Results:
  Duplicates removed: 2
  Removed: ['fs (duplicate of filesystem)', 'git2 (duplicate of git)']
  Remaining servers: ['filesystem', 'git', 'unique']
  Expected: filesystem, git, unique
```

**Result**: ✅ Script works correctly

### Write Condition Tests ✅

All test cases pass:
- ✅ File doesn't exist → Writes
- ✅ Todo2 not configured, no dupes → Writes
- ✅ Todo2 not configured, has dupes → Writes
- ✅ Todo2 configured, no dupes → Doesn't write (correct)
- ✅ Todo2 configured, has dupes → Writes (correct)

## Summary

### ✅ Logic is Correct

**Flow:**
1. Check Todo2 directory → Skip if missing
2. Ensure `.cursor/` directory exists
3. Read/create MCP config
4. Remove duplicates (if file exists + has servers)
5. Add Todo2 (if not present)
6. Write config (if Todo2 added OR duplicates removed)

### ✅ Edge Cases Handled

- Invalid JSON in existing file
- Script failures
- Undefined variables
- Empty configurations
- Missing files

### ✅ Improvements Made

1. Added error handling for JSON parsing
2. Added validation for script output
3. Added safe variable access patterns
4. Added validation before writing

## Recommendations

### Current Status: ✅ Ready to Use

The playbook logic is **correct and safe** to run. All edge cases are handled, and error conditions are managed gracefully.

### Optional Enhancements (Future)

1. **Project root detection**: Could check for `.git` or `.todo2` markers
2. **Backup creation**: Could create backup before writing
3. **Dry-run mode**: Could add `--check` support
4. **Verbose logging**: Could add more detailed progress messages

---

**Last Updated**: 2025-12-24
**Status**: Logic Reviewed, Issues Fixed, Ready for Production Use
