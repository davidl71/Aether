# Todo2 Update Tool JSON Error - Diagnostic

## Issue
The `mcp_Todo2_todo2-extension-todo2_update_todos` tool is failing with:
```
Error: Expected property name or '}' in JSON at position 14 (line 1 column 15)
```

## Root Cause
The tool requires an `updates` parameter (array of update objects), but there appears to be a JSON parsing issue when the parameter is provided through the MCP interface.

## Workaround Options

### Option 1: Manual Update via Todo2 Interface
Update the 48 tasks manually through the Todo2 UI/interface.

### Option 2: Direct JSON File Edit
The Todo2 state file is located at: `.todo2/state.todo2.json`

You can manually edit this file to change task statuses from `"In Progress"` to `"Review"` for the following 48 tasks:
- T-1, T-2, T-9, T-14, T-15, T-22, T-48, T-56, T-57, T-58
- T-59, T-85, T-86, T-87, T-88, T-89, T-90, T-91, T-93, T-94
- T-96, T-97, T-139, T-162, T-163, T-164, T-167, T-169, T-171, T-172
- T-173, T-174, T-175, T-176, T-177, T-178, T-179, T-180, T-185, T-186
- T-187, T-188, T-189, T-191, T-192, T-194, T-197, T-208

**⚠️ Warning**: Back up the file before editing!

### Option 3: Use Python Script to Update State File
A Python script could directly modify the `.todo2/state.todo2.json` file to update task statuses.

## Generated Resources
- `scripts/update_tasks_to_review.py` - Generates proper update commands
- `docs/analysis/TASK_UPDATE_COMMANDS.json` - Contains all 48 task updates in batches

## Next Steps
1. Check Todo2 MCP server configuration
2. Verify MCP server is running correctly
3. Try updating tasks one at a time to isolate the issue
4. Consider using direct file manipulation as a workaround
