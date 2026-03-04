# Batch TODO2 Task Update Script

**Location**: `scripts/batch_update_todos.py`

**Purpose**: Provides a command-line interface for batch operations on TODO2 tasks, replacing the need for Python heredocs in terminal commands.

---

## Quick Start

### Batch Approve Tasks (Most Common Use Case)

```bash

# Approve all Review tasks with no clarification needed

python3 scripts/batch_update_todos.py approve --status Review --clarification-none

# Approve with auto-confirmation (skip prompt)

python3 scripts/batch_update_todos.py approve --status Review --clarification-none --yes
```

### List Tasks Matching Criteria

```bash

# List all Review tasks

python3 scripts/batch_update_todos.py list --status Review

# List Review tasks with no clarification needed

python3 scripts/batch_update_todos.py list --status Review --clarification-none

# List tasks by tag

python3 scripts/batch_update_todos.py list --filter-tag research
```

### Update Task Status

```bash

# Update specific tasks

python3 scripts/batch_update_todos.py update-status --task-ids T-156,T-157 --new-status Todo

# Update all Review tasks to Todo

python3 scripts/batch_update_todos.py update-status --status Review --new-status Todo --yes
```

### Add Comments

```bash

# Add comment to specific tasks


python3 scripts/batch_update_todos.py add-comment --task-ids T-156 --comment "Approved for execution"
```

---

## Commands

### `approve`

Approve tasks (typically move Review → Todo).

**Options:**

- `--status`: Current status to filter (default: `Review`)
- `--new-status`: New status (default: `Todo`)
- `--clarification-none`: Only approve tasks with no clarification needed
- `--filter-tag`: Filter by tag (e.g., `research`)
- `--task-ids`: Comma-separated list of specific task IDs
- `--comment`: Custom comment (default: auto-generated)
- `--yes` / `-y`: Skip confirmation prompt

**Examples:**

```bash


# Approve all Review tasks with no clarification needed

python3 scripts/batch_update_todos.py approve --status Review --clarification-none

# Approve specific tasks

python3 scripts/batch_update_todos.py approve --task-ids T-156,T-157,T-158 --yes

# Approve research tasks only


python3 scripts/batch_update_todos.py approve --status Review --filter-tag research --clarification-none
```

### `update-status`

Update status for tasks matching criteria.

**Options:**

- `--status`: Current status to filter
- `--new-status`: New status (required)
- `--clarification-none`: Only tasks with no clarification needed

- `--filter-tag`: Filter by tag
- `--task-ids`: Comma-separated list of task IDs
- `--comment`: Comment to add
- `--yes` / `-y`: Skip confirmation

**Examples:**

```bash

# Move all Review tasks to Todo

python3 scripts/batch_update_todos.py update-status --status Review --new-status Todo --yes

# Update specific tasks

python3 scripts/batch_update_todos.py update-status --task-ids T-156 --new-status "In Progress"
```

### `add-comment`

Add comments to specific tasks.

**Options:**

- `--task-ids`: Comma-separated list of task IDs (required)
- `--comment`: Comment content (required)
- `--comment-type`: Comment type (default: `note`)

**Examples:**

```bash

# Add a note comment

python3 scripts/batch_update_todos.py add-comment --task-ids T-156 --comment "Approved for execution"

# Add a result comment

python3 scripts/batch_update_todos.py add-comment --task-ids T-156 --comment "Task completed" --comment-type result
```

### `list`

List tasks matching criteria (read-only, no changes).

**Options:**

- `--status`: Filter by status
- `--filter-tag`: Filter by tag
- `--clarification-none`: Only tasks with no clarification needed
- `--task-ids`: Comma-separated list of task IDs

**Examples:**

```bash

# List all Review tasks

python3 scripts/batch_update_todos.py list --status Review

# List tasks needing clarification

python3 scripts/batch_update_todos.py list --status Review --clarification-none
```

---

## Common Workflows

### 1. Batch Approve Research Tasks

```bash


# Step 1: List tasks that can be approved

python3 scripts/batch_update_todos.py list --status Review --clarification-none

# Step 2: Approve them

python3 scripts/batch_update_todos.py approve --status Review --clarification-none --yes
```

### 2. Review and Approve Specific Tasks

```bash

# Step 1: List specific tasks

python3 scripts/batch_update_todos.py list --task-ids T-156,T-157,T-158

# Step 2: Approve them

python3 scripts/batch_update_todos.py approve --task-ids T-156,T-157,T-158 --yes

```

### 3. Move Tasks to Review Status

```bash

# Move specific tasks to Review


python3 scripts/batch_update_todos.py update-status --task-ids T-156 --new-status Review --comment "Needs user input"
```

---

## Integration with MCP Tools

This script complements the MCP tools available in the Project Automation MCP Server:

- **MCP Tools**: Use for interactive operations via Cursor chat
- **This Script**: Use for batch operations and automation

**When to use each:**

- **MCP Tools**: Single task updates, interactive workflows, Cursor chat integration
- **This Script**: Batch operations, automation scripts, command-line workflows

---

## Error Handling

The script will:

- Validate that `.todo2/state.todo2.json` exists
- Show confirmation prompts (unless `--yes` is used)
- Display errors clearly if operations fail

- Preserve existing task data (only updates specified fields)

---

## Examples from Recent Work

### Batch Approval (What We Just Did)

Instead of:

```bash
python3 << 'EOF'

# ... 100+ lines of Python code ...

EOF
```

We can now use:

```bash
python3 scripts/batch_update_todos.py approve --status Review --clarification-none --yes
```

### List Tasks Needing Clarification

```bash
python3 scripts/batch_update_todos.py list --status Review
```

This shows all Review tasks, making it easy to see which ones need your input.

---

## Future Enhancements

Potential additions:

- `--dry-run` flag to preview changes
- JSON output format for scripting
- Filter by priority
- Filter by date ranges
- Bulk tag operations
- Export/import task lists

---

**See Also:**

- `docs/TODO2_EXECUTION_CONTEXT.md` - Task execution context metadata
- exarp-go MCP (see docs/MCP_REQUIRED_SERVERS.md) for task automation
