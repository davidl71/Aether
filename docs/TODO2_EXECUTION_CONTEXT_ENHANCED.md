# TODO2 Execution Context - Enhanced with Best Mode & Location Type ✅

**Date:** 2025-01-20
**Status:** ✅ **Enhanced - Best Mode & Location Type Added**

---

## Summary

Execution context metadata has been enhanced to include:
1. **Best Mode:** Cursor AI mode (Agent/Plan/Ask)
2. **Location Type:** Execution location (Local/Worktree)

This enables better task execution planning and optimal Cursor AI mode selection.

---

## New Fields Added

### Best Mode (Cursor AI Mode)

**Purpose:** Indicates the optimal Cursor AI mode for task execution

**Values:**
- **Agent:** Full AI agent mode - makes decisions, implements changes autonomously
- **Plan:** Planning mode - creates detailed plan, requires approval before execution
- **Ask:** Question mode - asks clarifying questions, minimal autonomous action

**Usage:**
- **Agent mode:** For well-defined implementation tasks (most MCP extension tasks)
- **Plan mode:** For complex tasks requiring coordination (e.g., MCP-EXT-8, MCP-EXT-10)
- **Ask mode:** For tasks requiring clarification or human input

### Location Type

**Purpose:** Indicates where to execute the task within the repository

**Values:**
- **Local:** Execute on local machine directly (main repository)
- **Worktree:** Execute in isolated git worktree (parallel development, isolation)

**Usage:**
- **Local:** For quick tasks, single-file changes, documentation updates
- **Worktree:** For feature development, large refactoring, parallel work streams

---

## Updated Execution Context Format

```markdown
📋 **Execution Context:**
- **Location:** `any` | `remote` | `local` (which agent can run it)
- **Location Type:** `local` | `worktree` (where to execute)
- **Best Mode:** `Agent` | `Plan` | `Ask` (Cursor AI mode)
- **Mode:** `background` | `foreground` | `interactive` | `automated`
- **Resources:** `network`, `cpu-intensive`, etc. (optional)
- **Remote Agent:** `any` | `ubuntu-agent` | `macos-m4-agent`
- **Background:** `yes` | `no` | `optional`
- **Local Interaction:** `required` | `not-required` | `optional`
```

---

## Updated Tags

### New Tags Added

**Best Mode Tags:**
- `execution-mode-cursor-agent` - Use Cursor Agent mode
- `execution-mode-cursor-plan` - Use Cursor Plan mode
- `execution-mode-cursor-ask` - Use Cursor Ask mode

**Location Type Tags:**
- `execution-location-type-local` - Execute in local repository
- `execution-location-type-worktree` - Execute in git worktree

---

## MCP-EXT Tasks Updated

All 10 MCP-EXT tasks now include:

### Best Mode Distribution

| Best Mode | Count | Tasks |
|-----------|-------|-------|
| **Agent** | 8 | MCP-EXT-1, MCP-EXT-2, MCP-EXT-3, MCP-EXT-4, MCP-EXT-5, MCP-EXT-6, MCP-EXT-7, MCP-EXT-9 |
| **Plan** | 2 | MCP-EXT-8, MCP-EXT-10 |

**Rationale:**
- **Agent mode (8 tasks):** Implementation tasks - clear requirements, autonomous execution
- **Plan mode (2 tasks):** Analysis/coordination tasks - benefit from planning before execution

### Location Type Distribution

| Location Type | Count | Tasks |
|---------------|-------|-------|
| **Local** | 10 | All MCP-EXT tasks |

**Rationale:**
- **Local:** All MCP extension tasks are tool implementations that work on the main repository
- **Worktree:** Not needed for these tasks (can be used for feature branches if desired)

---

## Decision Guidelines

### When to Use Each Best Mode

#### Agent Mode
**Use when:**
- Task has clear, well-defined requirements
- Implementation approach is straightforward
- Autonomous execution is safe
- No major architectural decisions needed

**Examples:**
- Tool implementation (MCP-EXT-1 through MCP-EXT-7, MCP-EXT-9)
- Standard patterns to follow
- Clear acceptance criteria

#### Plan Mode
**Use when:**
- Task requires coordination or analysis
- Multiple approaches possible
- Benefits from structured planning
- Complex task requiring approval

**Examples:**
- Task distribution analysis (MCP-EXT-8)
- Coordination reports (MCP-EXT-10)
- Architecture decisions
- Multi-component changes

#### Ask Mode
**Use when:**
- Requirements need clarification
- Multiple valid approaches exist
- Human input/decision required
- Task scope is ambiguous

**Examples:**
- Configuration decisions
- User preference choices
- Ambiguous requirements
- Tasks needing human approval

### When to Use Each Location Type

#### Local
**Use when:**
- Quick tasks (< 1 hour)
- Single-file changes
- Documentation updates
- Configuration changes
- Tool implementations (like MCP extensions)

**Advantages:**
- Direct access to repository
- No worktree overhead
- Faster execution

#### Worktree
**Use when:**
- Feature development
- Large refactoring (> 5 files)
- Parallel work streams
- Isolation needed
- Branch-specific changes

**Advantages:**
- Isolation from main branch
- Parallel development safe
- Easy to discard if needed

---

## Examples

### Example 1: Agent Mode + Local

```markdown
📋 **Execution Context:**
- **Location:** `any` (can run on any agent)
- **Location Type:** `local` (where to execute)
- **Best Mode:** `Agent` (Cursor AI mode: Agent/Plan/Ask)
- **Mode:** `automated` | `background`
- **Resources:** None
- **Remote Agent:** `any`
- **Background:** `yes`
- **Local Interaction:** `not-required`
```

**Task:** MCP-EXT-1 (validate_ci_cd_workflow_tool implementation)

**Reasoning:**
- Clear requirements → Agent mode
- Tool implementation → Local (main repository)
- Well-defined pattern to follow

### Example 2: Plan Mode + Local

```markdown
📋 **Execution Context:**
- **Location:** `any` (can run on any agent)
- **Location Type:** `local` (where to execute)
- **Best Mode:** `Plan` (Cursor AI mode: Agent/Plan/Ask)
- **Mode:** `automated` | `background`
- **Resources:** None
- **Remote Agent:** `any`
- **Background:** `yes`
- **Local Interaction:** `not-required`
```

**Task:** MCP-EXT-8 (analyze_agent_task_distribution_tool)

**Reasoning:**
- Analysis task → Plan mode (benefits from structured planning)
- Tool implementation → Local (main repository)
- Requires coordination analysis

### Example 3: Ask Mode + Worktree

```markdown
📋 **Execution Context:**
- **Location:** `local` (local machine)
- **Location Type:** `worktree` (where to execute)
- **Best Mode:** `Ask` (Cursor AI mode: Agent/Plan/Ask)
- **Mode:** `interactive`
- **Resources:** None
- **Remote Agent:** `not-applicable`
- **Background:** `no`
- **Local Interaction:** `required`
```

**Task:** Feature development requiring architectural decisions

**Reasoning:**
- Ambiguous requirements → Ask mode (needs clarification)
- Feature development → Worktree (isolation needed)
- Requires human input

---

## Integration Benefits

### For Cursor AI

1. **Optimal Mode Selection:**
   - Automatically select best Cursor AI mode
   - Agent mode for autonomous tasks
   - Plan mode for complex tasks
   - Ask mode for clarification needs

2. **Execution Location:**
   - Automatically use worktree for parallel work
   - Use local for quick tasks
   - Prevent conflicts with parallel agents

### For Parallel Agents

1. **Task Delegation:**
   - Match location type to agent setup
   - Worktree tasks → Isolated execution
   - Local tasks → Direct execution

2. **Mode Coordination:**
   - Plan mode tasks → Require coordination
   - Agent mode tasks → Can run independently
   - Ask mode tasks → Need human availability

---

## Filtering Examples

### Filter by Best Mode

**Agent Mode Tasks:**
```bash
grep -l "execution-mode-cursor-agent" .todo2/state.todo2.json
```

**Plan Mode Tasks:**
```bash
grep -l "execution-mode-cursor-plan" .todo2/state.todo2.json
```

**Ask Mode Tasks:**
```bash
grep -l "execution-mode-cursor-ask" .todo2/state.todo2.json
```

### Filter by Location Type

**Local Tasks:**
```bash
grep -l "execution-location-type-local" .todo2/state.todo2.json
```

**Worktree Tasks:**
```bash
grep -l "execution-location-type-worktree" .todo2/state.todo2.json
```

### Combined Filters

**Agent Mode + Local:**
- Quick autonomous tasks
- Can execute immediately

**Plan Mode + Worktree:**
- Complex feature work
- Requires planning and isolation

---

## Files Updated

### Scripts

- ✅ `scripts/add_execution_context_to_todos.py` - Enhanced with Best Mode and Location Type
- ✅ All MCP-EXT tasks updated in `.todo2/state.todo2.json`

### Documentation

- ✅ `docs/TODO2_EXECUTION_CONTEXT.md` - Updated with new fields
- ✅ `docs/TODO2_EXECUTION_CONTEXT_ENHANCED.md` - This document

---

## Next Steps

1. ✅ **Enhancement Complete** - All tasks updated
2. 📋 **Use in Workflow** - Apply Best Mode and Location Type when executing tasks
3. 📋 **Refine Guidelines** - Learn from execution patterns and refine recommendations
4. 📋 **Create Automation** - Auto-select Cursor mode based on execution context

---

**Status:** ✅ **Enhanced - Best Mode & Location Type Added to All Tasks**

**Result:** All TODO2 tasks now include comprehensive execution context for optimal parallel agent workflows!
