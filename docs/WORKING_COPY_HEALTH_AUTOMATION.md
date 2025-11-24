# Working Copy Health Automation

**Status:** ✅ Integrated

**Purpose:** Automatically check git working copy status across all agents and runners before task execution.

---

## Overview

Working copy health checking ensures all agents have clean working copies and are in sync with the remote repository before starting work. This prevents conflicts and ensures everyone is working on the latest code.

---

## Available Tools

### 1. Command-Line Script

**Location:** `scripts/check_working_copy_status.sh`

**Usage:**
```bash
# Check all agents
bash scripts/check_working_copy_status.sh

# Output shows:
# - Uncommitted changes per agent
# - Branch status
# - Sync status (behind/ahead)
# - Recommendations
```

**What It Checks:**
- Local agent (current machine)
- Ubuntu agent (192.168.192.57)
- macOS M4 agent (192.168.192.141)

**Output:**
- ✅ Clean working copy
- ⚠️ Uncommitted changes
- ⚠️ Behind/ahead of remote
- ❌ Connection errors

---

### 2. MCP Tool

**Tool:** `check_working_copy_health_tool`

**Usage via Cursor Chat:**
```
"Check working copy status across all agents"
"Check if ubuntu agent has clean working copy"
"Verify all agents are in sync"
```

**Parameters:**
- `agent_name` (Optional): Specific agent to check (local/ubuntu/macos)
- `check_remote` (bool): Whether to check remote agents (default: true)

**Returns:**
```json
{
  "summary": {
    "total_agents": 3,
    "ok_agents": 2,
    "warning_agents": 1,
    "error_agents": 0,
    "agents_with_uncommitted_changes": 1,
    "agents_behind_remote": 0
  },
  "agents": {
    "local": {
      "status": "ok",
      "has_uncommitted_changes": false,
      "branch": "main",
      "behind_remote": 0,
      "ahead_remote": 0,
      "in_sync": true
    },
    ...
  },
  "recommendations": [
    "All agents have clean working copies and are in sync"
  ]
}
```

---

### 3. Integrated into Nightly Automation

**Location:** `mcp-servers/project-management-automation/tools/nightly_task_automation.py`

**What It Does:**
- Automatically checks working copy health before task execution
- Includes warnings in results if agents have issues
- Reports `working_copy_warnings` count in summary

**Results Include:**
```json
{
  "working_copy_status": {
    "summary": {...},
    "agents": {...},
    "recommendations": [...]
  },
  "summary": {
    "working_copy_warnings": 1,
    ...
  }
}
```

---

## Current Status Across Agents

### Local Agent
- **Status:** ⚠️ Has uncommitted changes
- **Sync:** ✅ In sync with origin/main
- **Branch:** main

### Ubuntu Agent
- **Status:** ⚠️ Has uncommitted changes
- **Sync:** ⚠️ Behind origin/main by 6 commits
- **Branch:** main
- **Uncommitted Files:**
  - Modified: `ansible/roles/devtools/tasks/main.yml`
  - Modified: `docs/PWA_FEATURE_PRIORITIZATION.md`
  - Modified: `setup_global_tools.sh`
  - Untracked: Several cursor-extension files

### macOS M4 Agent
- **Status:** ⚠️ Has uncommitted changes
- **Sync:** ⚠️ Behind origin/main by 38 commits
- **Branch:** main
- **Uncommitted Files:**
  - Untracked: `python/integration/__init__.py`

---

## Recommendations

### Immediate Actions

1. **Commit and Push Local Changes**
   ```bash
   git add -A
   git commit -m "Your commit message"
   git push
   ```

2. **Sync Ubuntu Agent**
   ```bash
   ssh david@192.168.192.57
   cd ~/ib_box_spread_full_universal
   git status  # Review changes
   git stash  # If needed, save uncommitted changes
   git pull   # Pull latest
   ```

3. **Sync macOS Agent**
   ```bash
   ssh davidl@192.168.192.141
   cd /Users/davidl/Projects/Trading/ib_box_spread_full_universal
   git status  # Review changes
   git stash  # If needed, save uncommitted changes
   git pull   # Pull latest
   ```

---

## Automation Integration

### Nightly Automation

The nightly automation now:
1. ✅ Checks working copy health before task execution
2. ✅ Reports warnings in results
3. ✅ Includes recommendations in output

### GitHub Actions

GitHub Actions workflows automatically:
- ✅ Checkout clean working copy
- ✅ Pull latest changes
- ✅ Run in isolated environment

---

## Best Practices

### Before Starting Work

1. **Check Working Copy Status**
   ```bash
   bash scripts/check_working_copy_status.sh
   ```

2. **Or Use MCP Tool**
   ```
   "Check working copy status"
   ```

3. **Fix Issues Before Starting**
   - Commit or stash uncommitted changes
   - Pull latest changes if behind
   - Push local commits if ahead

### During Work

- Commit frequently
- Push regularly to keep agents in sync
- Use `git status` before switching tasks

### After Work

- Commit and push all changes
- Verify all agents are in sync
- Run working copy health check

---

## Troubleshooting

### Agent Shows "Cannot Connect"

**Check:**
1. SSH connectivity: `ssh david@192.168.192.57`
2. Network connectivity
3. SSH keys configured

### Agent Shows "Error Checking Status"

**Check:**
1. Git repository exists at path
2. Permissions are correct
3. Git is installed on remote agent

### Agent Behind Remote

**Fix:**
```bash
ssh <agent>
cd <project_path>
git pull
```

### Agent Has Uncommitted Changes

**Options:**
1. Commit changes: `git add -A && git commit -m "message"`
2. Stash changes: `git stash`
3. Discard changes: `git restore .` (careful!)

---

## See Also

- `scripts/check_working_copy_status.sh` - Manual check script
- `mcp-servers/project-management-automation/tools/working_copy_health.py` - MCP tool
- `docs/AGENT_HOSTNAMES.md` - Agent connection details
- `docs/NIGHTLY_TASK_AUTOMATION.md` - Nightly automation details
