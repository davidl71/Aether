# TODO2 Task Update Methods

**Overview**: Three ways to update TODO2 tasks - choose the right tool for the job.

---

## Method Comparison

| Method | Use Case | When to Use | Example |
|--------|----------|-------------|---------|
| **Batch Script** | Batch operations, automation | Multiple tasks, command-line workflows | `scripts/batch_update_todos.py approve --status Review --clarification-none` |
| **MCP Tools** | Interactive, single tasks | Cursor chat, one-off updates | `update_todos` via MCP |
| **Python Heredoc** | Quick one-liners | Temporary scripts, throwaway code | `python3 << 'EOF' ... EOF` |

---

## 1. Batch Update Script (Recommended for Batch Operations)

**Location**: `scripts/batch_update_todos.py`

**Best For:**
- ✅ Batch approving multiple tasks
- ✅ Command-line automation
- ✅ Reusable workflows
- ✅ CI/CD integration

**Quick Examples:**
```bash
# Approve all Review tasks with no clarification needed
python3 scripts/batch_update_todos.py approve --status Review --clarification-none --yes

# List tasks needing decisions
python3 scripts/batch_update_todos.py list --status Review

# Update specific tasks
python3 scripts/batch_update_todos.py update-status --task-ids T-156,T-157 --new-status Todo
```

**Documentation**: See `docs/BATCH_TODO_UPDATE_SCRIPT.md` for full documentation.

---

## 2. MCP Tools (Recommended for Interactive Use)

**Location**: `mcp-servers/project-management-automation/`

**Available Tools:**
- `update_todos` - Update task status, content, tags, dependencies
- `add_comments` - Add research, result, note, or manualsetup comments
- `list_todos` - View filtered task lists
- `get_todo_details` - Retrieve full task details

**Best For:**
- ✅ Interactive Cursor chat workflows
- ✅ Single task updates
- ✅ Complex updates with multiple fields
- ✅ Integration with AI assistant

**Example (via Cursor chat):**
```
"Update task T-156 to Todo status and add a note comment"
```

**Documentation**: See `mcp-servers/project-management-automation/TOOLS_STATUS.md`

---

## 3. Python Heredoc (Not Recommended)

**Best For:**
- ❌ Quick one-off scripts (but script is better)
- ❌ Temporary throwaway code (but script is better)

**Why Not Recommended:**
- ❌ Not reusable
- ❌ Hard to maintain
- ❌ No documentation
- ❌ Error-prone

**Example (Old Way - Don't Use):**
```bash
python3 << 'EOF'
import json
# ... 100+ lines ...
EOF
```

**Better Alternative**: Use the batch script instead.

---

## Recommended Workflow

### For Batch Operations (Most Common)

```bash
# Step 1: See what needs approval
python3 scripts/batch_update_todos.py list --status Review --clarification-none

# Step 2: Approve them
python3 scripts/batch_update_todos.py approve --status Review --clarification-none --yes
```

### For Interactive Updates

Use MCP tools via Cursor chat:
```
"Show me all Review tasks"
"Approve task T-156"
"Add a comment to T-156 saying it's approved"
```

### For Automation Scripts

```bash
# In your automation script
python3 scripts/batch_update_todos.py approve --status Review --clarification-none --yes
```

---

## Quick Reference

| Task | Method | Command |
|------|--------|---------|
| Batch approve research tasks | Script | `scripts/batch_update_todos.py approve --status Review --clarification-none --yes` |
| List tasks needing decisions | Script | `scripts/batch_update_todos.py list --status Review` |
| Update single task | MCP | Use `update_todos` via Cursor chat |
| Add comment to task | MCP | Use `add_comments` via Cursor chat |
| Complex batch operations | Script | `scripts/batch_update_todos.py update-status ...` |

---

## Migration Guide

### Replacing Python Heredocs

**Before:**
```bash
python3 << 'EOF'
import json
from pathlib import Path
# ... 100+ lines of code ...
EOF
```

**After:**
```bash
python3 scripts/batch_update_todos.py approve --status Review --clarification-none --yes
```

**Benefits:**
- ✅ 1 line instead of 100+
- ✅ Reusable
- ✅ Documented
- ✅ Maintainable

---

## See Also

- `docs/BATCH_TODO_UPDATE_SCRIPT.md` - Full script documentation
- `docs/BATCH_TODO_QUICK_REFERENCE.md` - Quick command reference
- `mcp-servers/project-management-automation/TOOLS_STATUS.md` - MCP tools documentation
