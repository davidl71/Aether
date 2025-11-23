# Nightly Task Automation - Quick Start

**Date:** 2025-01-20
**Status:** ✅ **Ready to Use**

---

## What It Does

The Nightly Task Automation tool automatically:
1. ✅ **Identifies** background-capable tasks (MCP extensions, research, implementation, etc.)
2. ✅ **Assigns** them to remote agents (Ubuntu + macOS) in parallel
3. ✅ **Moves** interactive tasks to Review status (requires user input)
4. ✅ **Proceeds** to next tasks automatically

---

## Quick Usage

### Via MCP Tool (Recommended)

```
Run nightly task automation with default settings (5 tasks per host, 10 total)
```

Or with parameters:
```
Run nightly task automation: max 10 tasks per host, 20 total parallel tasks, high priority only, dry run mode
```

---

## Default Behavior

- **Max Tasks Per Host:** 5
- **Max Parallel Tasks:** 10
- **Priority Filter:** None (all priorities)
- **Tag Filter:** None (all tags)
- **Dry Run:** False (actually executes)

---

## Example: Dry Run (Preview First)

```
Run nightly task automation in dry run mode to preview what would happen
```

This shows:
- Which tasks would be assigned
- Which tasks would be moved to Review
- No actual changes made

---

## Example: High Priority Only

```
Run nightly task automation with high priority filter, max 20 parallel tasks
```

This processes only high-priority background tasks.

---

## Example: MCP Extensions Only

```
Run nightly task automation with tag filter ["mcp"], max 10 parallel tasks
```

This processes only MCP extension tasks.

---

## What Gets Assigned

**Background-Capable Tasks:**
- ✅ MCP extension tasks (MCP-EXT-*)
- ✅ Research tasks
- ✅ Implementation tasks
- ✅ Testing tasks
- ✅ Documentation tasks
- ✅ Configuration tasks

**Excluded:**
- ❌ Design tasks (need decisions)
- ❌ Tasks requiring clarification
- ❌ Tasks needing user input
- ❌ Already in progress or done

---

## What Gets Moved to Review

**Interactive Tasks:**
- Tasks with "clarification required"
- Tasks needing user input
- Design decision tasks
- Strategy/planning tasks

**Result:** Status changed to "Review" with explanatory comment

---

## Output

The tool returns:
- **Summary:** Statistics (tasks found, assigned, moved, hosts used)
- **Assigned Tasks:** List of tasks assigned to each host
- **Moved to Review:** List of task IDs moved to Review
- **Remaining:** Count of background tasks not yet assigned

---

## Scheduling (Future)

### Via GitHub Actions

```yaml
on:
  schedule:
    - cron: '0 2 * * *'  # 2 AM daily
```

### Via Cron

```bash
0 2 * * * cd /path/to/project && python3 -m tools.nightly_task_automation
```

---

## Safety

### ✅ Dry Run Mode

Always test first:
```
Run nightly task automation in dry run mode
```

### ✅ Automatic Backups

State file automatically backed up before changes (`.todo2/state.todo2.json.bak`)

### ✅ Audit Trail

All changes logged in:
- Task comments (why moved to Review)
- Task status changes
- TODO2 state file

---

## Troubleshooting

### No Tasks Assigned

**Check:**
1. Tasks are in "Todo" status
2. Tasks are background-capable
3. Host limits not exceeded
4. Priority/tag filters not too restrictive

### Too Many Tasks in Review

**Expected:** Interactive tasks are intentionally moved to Review

**Action:** Review moved tasks and provide clarifications/input

---

## Next Steps

1. **Test with Dry Run:**
   ```
   Run nightly task automation in dry run mode
   ```

2. **Execute First Run:**
   ```
   Run nightly task automation with default settings
   ```

3. **Review Results:**
   - Check assigned tasks
   - Review tasks moved to Review
   - Provide input for Review tasks

---

## Related Documentation

- [Full Documentation](./NIGHTLY_TASK_AUTOMATION.md)
- [Execution Mode Analysis](./TODO2_EXECUTION_MODE_ANALYSIS.md)
- [Agent Hostnames](./AGENT_HOSTNAMES.md)

---

**Status:** ✅ **Ready - Test with Dry Run First!**
