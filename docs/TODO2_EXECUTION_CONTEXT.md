# TODO2 Execution Context Guidelines

**Date:** 2025-01-20
**Purpose:** Standardize execution context metadata for TODO2 tasks in parallel agent workflows
**Status:** ✅ **Active Standard**

---

## Overview

Execution context metadata helps parallel agents understand:

1. **Where** tasks can be executed (local, remote agent, either)
2. **How** tasks can be executed (background, foreground, interactive)
3. **What** resources are required (specific agent type, local tools, network access)

This enables better task delegation and parallel execution in multi-agent workflows.

---

## Execution Context Tags

### Location Tags

**Tag Format:** `execution-location-[type]`

| Tag | Meaning | Use Case |
|-----|---------|----------|
| `execution-location-local` | Task requires local machine | Local file access, local tools, macOS-specific |
| `execution-location-remote` | Task can run on remote agent | Network tasks, agent-specific work |
| `execution-location-any` | Task can run anywhere | Cloud-based, agent-agnostic |
| `execution-location-ubuntu` | Task requires Ubuntu agent | Linux-specific tools, Ubuntu environment |
| `execution-location-macos` | Task requires macOS agent | Apple-specific, macOS tools, Apple Intelligence |

### Mode Tags

**Tag Format:** `execution-mode-cursor-[type]` and `execution-mode-type-[type]`

| Tag | Meaning | Use Case |
|-----|---------|----------|
| `execution-mode-cursor-agent` | Use Cursor Agent mode | Full autonomous implementation |
| `execution-mode-cursor-plan` | Use Cursor Plan mode | Requires approval before execution |
| `execution-mode-cursor-ask` | Use Cursor Ask mode | Ask questions, minimal autonomous action |
| `execution-location-type-local` | Execute in local repository | Direct work on main branch |
| `execution-location-type-worktree` | Execute in git worktree | Isolated parallel development |

### Execution Mode Tags

**Tag Format:** `execution-mode-[type]`

| Tag | Meaning | Use Case |
|-----|---------|----------|
| `execution-mode-background` | Can run in background | Long-running tasks, monitoring, automated |
| `execution-mode-foreground` | Requires foreground execution | Interactive tasks, user input |
| `execution-mode-interactive` | Requires user interaction | Manual setup, approval, human decisions |
| `execution-mode-automated` | Fully automated | Scripts, automated tests, CI/CD |

### Resource Tags

**Tag Format:** `execution-resource-[type]`

| Tag | Meaning | Use Case |
|-----|---------|----------|
| `execution-resource-network` | Requires network access | API calls, downloads, remote connections |
| `execution-resource-cpu-intensive` | CPU-intensive task | Compilation, heavy processing |
| `execution-resource-disk-intensive` | Disk-intensive task | Large file operations, builds |
| `execution-resource-memory-intensive` | Memory-intensive task | Large data processing |

---

## Description Format

Add execution context section to task long_description:

```markdown
📋 **Execution Context:**

- **Location:** `local` | `remote` | `any` | `ubuntu` | `macos` | `ubuntu-or-macos`
- **Location Type:** `local` | `worktree` (where to execute)
- **Best Mode:** `Agent` | `Plan` | `Ask` (Cursor AI mode)
- **Mode:** `background` | `foreground` | `interactive` | `automated`
- **Resources:** `network`, `cpu-intensive`, `disk-intensive`, `memory-intensive` (optional)
- **Remote Agent:** `ubuntu-agent` | `macos-m4-agent` | `any` (if applicable)
- **Background:** `yes` | `no` | `optional`
- **Local Interaction:** `required` | `not-required` | `optional`
```

### Mode Definitions

**Best Mode (Cursor AI Mode):**

- **Agent:** Full AI agent mode - makes decisions, implements changes autonomously
- **Plan:** Planning mode - creates detailed plan, requires approval before execution
- **Ask:** Question mode - asks clarifying questions, minimal autonomous action

**Location Type:**

- **Local:** Execute on local machine directly (main repository)
- **Worktree:** Execute in isolated git worktree (parallel development, isolation)

---

## Examples

### Example 1: Background Task on Remote Agent

```json
{
  "id": "T-XXX",
  "name": "Collect system information from Ubuntu agent",
  "tags": [
    "execution-location-ubuntu",
    "execution-mode-background",
    "execution-mode-automated",
    "execution-resource-network"
  ],
  "long_description": "...\n\n📋 **Execution Context:**\n- **Location:** `remote` (ubuntu-agent)\n- **Location Type:** `local` (where to execute)\n- **Best Mode:** `Agent` (Cursor AI mode: Agent/Plan/Ask)\n- **Mode:** `background` | `automated`\n- **Resources:** `network` (SSH connection)\n- **Remote Agent:** `ubuntu-agent`\n- **Background:** `yes`\n- **Local Interaction:** `not-required`\n..."
}
```

### Example 2: Interactive Local Task

```json
{
  "id": "T-YYY",
  "name": "Configure GitHub Actions runner",
  "tags": [
    "execution-location-local",
    "execution-mode-interactive",
    "execution-resource-network"
  ],
  "long_description": "...\n\n📋 **Execution Context:**\n- **Location:** `local` (requires manual setup)\n- **Location Type:** `local` (where to execute)\n- **Best Mode:** `Ask` (Cursor AI mode: Agent/Plan/Ask)\n- **Mode:** `interactive`\n- **Resources:** `network` (GitHub API access)\n- **Remote Agent:** `not-applicable`\n- **Background:** `no`\n- **Local Interaction:** `required` (runner token input)\n..."
}
```

### Example 3: Parallel Agent Task

```json
{
  "id": "T-ZZZ",
  "name": "Validate CI/CD workflow",
  "tags": [
    "execution-location-any",
    "execution-mode-automated",
    "execution-mode-background"
  ],
  "long_description": "...\n\n📋 **Execution Context:**\n- **Location:** `any` (can run on any agent)\n- **Location Type:** `local` (where to execute)\n- **Best Mode:** `Agent` (Cursor AI mode: Agent/Plan/Ask)\n- **Mode:** `background` | `automated`\n- **Resources:** None\n- **Remote Agent:** `any` (ubuntu-agent or macos-m4-agent)\n- **Background:** `yes`\n- **Local Interaction:** `not-required`\n..."
}
```

---

## Tag Combinations

### Common Patterns

| Pattern | Tags | Description |
|---------|------|-------------|
| **Remote Background** | `execution-location-remote`, `execution-mode-background` | Can run in background on remote agent |
| **Local Interactive** | `execution-location-local`, `execution-mode-interactive` | Requires local user interaction |
| **Parallel Automated** | `execution-location-any`, `execution-mode-automated` | Can run on any agent, fully automated |
| **Agent-Specific** | `execution-location-ubuntu` or `execution-location-macos` | Requires specific agent type |
| **Resource-Heavy** | `execution-resource-cpu-intensive`, `execution-mode-background` | Heavy processing, run in background |

---

## Integration with Parallel Agent Workflow

### Task Delegation Logic

**For Agent Assignment:**

1. Check `execution-location-*` tags
2. Match agent type (ubuntu/macos/any)
3. Assign to appropriate agent

**For Execution Planning:**

1. Check `execution-mode-*` tags
2. Background tasks → Schedule first, run in parallel
3. Interactive tasks → Require human availability
4. Automated tasks → Can run anytime

**For Resource Management:**

1. Check `execution-resource-*` tags
2. Heavy tasks → Schedule during low-usage periods
3. Network tasks → Ensure connectivity
4. CPU-intensive → May need dedicated time slots

---

## Task Creation Script Updates

### Updated Task Template

When creating tasks, include execution context:

```python
task = {
    "id": "T-XXX",
    "name": "Task name",
    "tags": [
        # Existing tags...
        "execution-location-ubuntu",  # Add location tag
        "execution-mode-background",  # Add mode tag
        "execution-resource-network"  # Add resource tags if needed
    ],
    "long_description": """
    ... existing description ...

    📋 **Execution Context:**
    - **Location:** `remote` (ubuntu-agent)
    - **Mode:** `background` | `automated`
    - **Resources:** `network` (SSH connection)
    - **Remote Agent:** `ubuntu-agent`
    - **Background:** `yes`
    - **Local Interaction:** `not-required`
    """
}
```

---

## Benefits

### For Parallel Agents

1. **Automatic Task Delegation:** Agents can auto-assign tasks based on execution context
2. **Optimal Scheduling:** Background tasks can run in parallel
3. **Resource Management:** Heavy tasks can be scheduled appropriately

### For Workflow Coordination

1. **Clear Requirements:** Know what each task needs upfront
2. **Better Planning:** Schedule interactive vs automated tasks appropriately
3. **Efficient Execution:** Run background tasks in parallel

### For Developers

1. **Clear Expectations:** Understand task requirements at a glance
2. **Better Planning:** Know which tasks can be delegated vs require local work
3. **Faster Execution:** Background tasks can start immediately

---

## Migration Plan

### Update Existing Tasks

1. **High Priority Tasks First:**
   - Tasks in "In Progress" status
   - Tasks assigned to agents
   - Recent tasks

2. **Batch Updates:**
   - Create script to add execution context to existing tasks
   - Focus on tasks without execution context
   - Validate with sample tasks first

3. **New Task Standard:**
   - All new tasks include execution context
   - Template includes execution context section
   - Validation checks for execution context

---

## Tools for Managing Execution Context

### Task Filtering

**By Location:**

- Filter: `execution-location-ubuntu` → Show Ubuntu tasks
- Filter: `execution-location-macos` → Show macOS tasks
- Filter: `execution-location-any` → Show parallelizable tasks

**By Mode:**

- Filter: `execution-mode-background` → Show background tasks
- Filter: `execution-mode-interactive` → Show interactive tasks

**By Resource:**

- Filter: `execution-resource-cpu-intensive` → Show CPU-heavy tasks
- Filter: `execution-resource-network` → Show network-required tasks

---

## Examples in Practice

### MCP Extension Tasks

**Tool 1: validate_ci_cd_workflow_tool**

- **Location:** `any` (can run on any agent)
- **Mode:** `automated` | `background`
- **Background:** `yes`
- **Local Interaction:** `not-required`

**Tool 2: validate_agent_coordination_tool**

- **Location:** `any` (can run on any agent)
- **Mode:** `automated` | `background`
- **Background:** `yes`
- **Local Interaction:** `not-required`

**Tool 3: collect_agent_environment_tool**

- **Location:** `remote` (requires SSH to remote agent)
- **Mode:** `automated` | `background`
- **Resources:** `network` (SSH connection)
- **Background:** `yes`
- **Local Interaction:** `not-required`

---

## Next Steps

1. ✅ **Document Standard** - This document
2. 📋 **Update Task Creation Script** - Add execution context to template
3. 📋 **Update Existing Tasks** - Add execution context to MCP-EXT-* tasks
4. 📋 **Create Filtering Tools** - Filter tasks by execution context
5. 📋 **Integrate with Agent Workflow** - Use execution context for task delegation

---

**Status:** ✅ **Standard Defined - Ready for Implementation**

**Next Action:** Update MCP-EXT-* tasks with execution context, then update task creation script.
