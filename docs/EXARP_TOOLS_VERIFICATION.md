# Exarp MCP Tools Verification

**Date**: 2025-12-24
**Status**: ⚠️ Pending Cursor Restart

## Configuration Status

### ✅ Configuration Verified

**Current Configuration:**

```json
{
  "exarp": {
    "command": "uvx",
    "args": ["exarp", "--mcp"],
    "env": {
      "PROJECT_ROOT": "/Users/davidl/Projects/Trading/ib_box_spread_full_universal"
    }
  }
}
```

**Status**: ✅ Configuration is correct

### ⚠️ MCP Server Status

**Current**: MCP server not loaded (Cursor needs restart)

**Expected**: After restart, server should be available as "exarp"

## Available Tools (Expected)

Based on `.cursor/rules/project-automation.mdc`, the following tools should be available:

### Documentation Tools

1. **`check_documentation_health_tool`** ⚠️ PREFERRED
   - Analyze documentation structure
   - Find broken references
   - Identify issues
   - Create Todo2 tasks for issues

2. **`check_attribution`**
   - Verify proper attribution for third-party components
   - Check ATTRIBUTIONS.md compliance

### Task Management Tools

3. **`analyze_todo2_alignment_tool`** ⚠️ PREFERRED
   - Analyze task alignment with project goals
   - Find misaligned tasks
   - Create follow-up tasks

4. **`task_discovery`**
   - Discover tasks from TODO/FIXME comments
   - Find tasks in markdown files
   - Identify orphaned tasks

5. **`task_workflow`**
   - Sync TODO markdown tables ↔ Todo2
   - Approve/move tasks by status
   - Manage task clarifications

6. **`task_analysis`**
   - Find duplicate tasks
   - Consolidate/cleanup task tags
   - Analyze task hierarchy
   - Analyze dependencies
   - Identify parallelization opportunities

7. **`improve_task_clarity`**
   - Add time estimates
   - Rename tasks to start with action verbs
   - Remove unnecessary dependencies
   - Break down large tasks

8. **`estimate_task_duration`**
   - Estimate task duration using statistical methods
   - Historical task completion data

### Security Tools

9. **`security`**
   - Scan dependencies for vulnerabilities
   - Fetch GitHub Dependabot alerts
   - Combined security report

### Testing Tools

10. **`testing`**
    - Execute test suite
    - Analyze test coverage
    - Suggest test cases
    - Validate test structure

### Linting Tools

11. **`lint`**
    - Execute linter
    - Analyze problems
    - Auto-fix issues

### Health & Reporting Tools

12. **`health`**
    - Server operational status
    - Git working copy health
    - Documentation health score
    - Definition of done validation
    - CI/CD workflow validation

13. **`report`**
    - One-page project overview
    - Health metrics scorecard
    - Advisor wisdom summary
    - Product requirements document

### Memory Tools

14. **`memory`**
    - Save insights
    - Recall memories for tasks
    - Search memories by query

### Context Tools

15. **`context`**
    - Summarize data
    - Token budget analysis
    - Batch summarization

### Recommendation Tools

16. **`recommend`**
    - AI model recommendations
    - Workflow mode suggestions
    - Advisor wisdom

### Workflow Tools

17. **`workflow_mode`**
    - Manage workflow modes
    - Get mode suggestions
    - View usage statistics

### Configuration Tools

18. **`generate_config`**
    - Generate .cursor/rules/*.mdc files
    - Generate .cursorignore/.cursorindexingignore
    - Simplify existing rule files

19. **`setup_hooks`**
    - Install git hooks
    - Install pattern triggers

### Prompt Tracking

20. **`prompt_tracking`**
    - Log prompt iterations
    - Analyze prompt patterns

### Memory Maintenance

21. **`memory_maint`**
    - Memory system health
    - Garbage collect stale memories
    - Prune low-value memories
    - Consolidate similar memories
    - Reflect on memories

### Git-Inspired Tools

22. **`get_task_commits_tool`**
    - Get commit history for a task

23. **`get_branch_commits_tool`**
    - Get all commits for a branch

24. **`list_branches_tool`**
    - List all branches with statistics

25. **`get_branch_tasks_tool`**
    - Get all tasks in a branch

26. **`compare_task_diff_tool`**
    - Compare two versions of a task

27. **`generate_graph_tool`**
    - Generate commit graph visualization

28. **`merge_branch_tools_tool`**
    - Merge tasks from one branch to another

29. **`set_task_branch`**
    - Set branch for a task

### Session Management

30. **`exarp_session_handoff`**
    - End/resume sessions
    - Git sync integration
    - Multi-device coordination

### Auto-Priming

31. **`auto_prime_session`**
    - Auto-prime AI context at session start
    - Detect agent/time/mode

32. **`get_task_context`**
    - Get optimized context for working on a task

33. **`infer_session_mode`**
    - Infer current session mode from tool patterns

### Prompt Discovery

34. **`find_prompts`**
    - Find relevant prompts by mode/persona/category/keywords

### Task Assignment

35. **`task_assignee_tool`**
    - Manage task assignments across agents/humans/hosts

### Automation

36. **`run_daily_automation`**
    - Run daily checks (docs_health, alignment, duplicates, security)

37. **`run_nightly_automation`**
    - Process background-capable tasks automatically

38. **`run_sprint_automation`**
    - Full sprint automation with subtask extraction

39. **`run_discover_automation`**
    - Find automation opportunities in codebase

### Tool Catalog

40. **`tool_catalog`**
    - Unified tool catalog and help

## Verification Test Plan

### Step 1: Restart Cursor ⚠️ REQUIRED

**Action**: Restart Cursor to load the new MCP server configuration

**Expected**: MCP server "exarp" should be available

### Step 2: Verify Server Connection

**Test**: List MCP resources from exarp server

**Expected**: Should return list of available tools/resources

### Step 3: Test Core Tools

Test the following core tools:

1. **Health Check**

   ```python
   mcp_exarp_pma_health(
       action="server",
       workingDirectory="/Users/davidl/Projects/Trading/ib_box_spread_full_universal"
   )
   ```

2. **Documentation Health**

   ```python
   mcp_exarp_pma_check_documentation_health_tool(
       workingDirectory="/Users/davidl/Projects/Trading/ib_box_spread_full_universal",
       create_tasks=False
   )
   ```

3. **Task Alignment**

   ```python
   mcp_exarp_pma_analyze_todo2_alignment_tool(
       workingDirectory="/Users/davidl/Projects/Trading/ib_box_spread_full_universal",
       create_followup_tasks=False
   )
   ```

4. **Security Scan**

   ```python
   mcp_exarp_pma_security(
       action="report",
       workingDirectory="/Users/davidl/Projects/Trading/ib_box_spread_full_universal"
   )
   ```

### Step 4: Verify Environment Variables

**Test**: Verify `PROJECT_ROOT` environment variable is set correctly

**Expected**: Tools should use the correct project root

## Current Status

### ✅ Command-Line Tool Works

```bash
$ uvx exarp --version
Version: 0.2.1.dev1766596911+g8d0d4dc.dirty

$ uvx exarp --help
# Shows help with --mcp option
```

### ⚠️ MCP Server Not Loaded

**Reason**: Cursor needs to be restarted to load new MCP configuration

**Action Required**: Restart Cursor

## Troubleshooting

### If Tools Don't Work After Restart

1. **Check MCP Server Status**
   - Open Cursor Settings → MCP Servers
   - Verify "exarp" server is listed and active

2. **Check Environment Variables**
   - Verify `PROJECT_ROOT` is set correctly
   - Check MCP server logs for errors

3. **Verify uvx Installation**

   ```bash
   uvx --version
   uvx exarp --help
   ```

4. **Check Configuration**

   ```bash
   cat .cursor/mcp.json | python3 -m json.tool | grep -A 8 '"exarp"'
   ```

## Next Steps

1. ⚠️ **Restart Cursor** (required to load new MCP configuration)
2. ✅ **Verify MCP server is available** (check Cursor MCP settings)
3. ✅ **Test core tools** (health, documentation, tasks, security)
4. ✅ **Document any issues** found during testing

---

**Last Updated**: 2025-12-24
**Status**: Configuration Verified, Awaiting Cursor Restart
