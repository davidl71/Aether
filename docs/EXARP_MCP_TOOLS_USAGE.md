# Exarp MCP Tools Usage Guide

**Date**: 2025-11-29  
**Status**: Active

---

## Overview

This guide documents how to use Exarp MCP tools as a workaround for the Exarp script discovery issue. While Exarp's aggregate daily automation tool cannot find scripts, individual MCP tools work perfectly.

---

## Available Tools

### 1. Documentation Health Check

**Tool**: `mcp_exarp_check_documentation_health`

**Purpose**: Checks documentation health, finds broken links, and validates documentation structure.

**Usage**:
```python
# Via MCP tool (in Cursor chat)
mcp_exarp_check_documentation_health()

# Via CLI
uvx exarp check-documentation-health /path/to/project [--dry-run]
```

**Output**: Documentation health report with broken links, format errors, and recommendations.

**Example**:
```bash
$ uvx exarp check-documentation-health . --dry-run
📚 Documentation Health Check
✅ Found 200+ documentation files
⚠️  26 broken links detected
📋 5 format errors found
```

---

### 2. Todo2 Alignment Analysis

**Tool**: `mcp_exarp_analyze_todo2_alignment`

**Purpose**: Analyzes Todo2 tasks for alignment with project goals and investment strategy framework.

**Usage**:
```python
# Via MCP tool (in Cursor chat)
mcp_exarp_analyze_todo2_alignment(create_followup_tasks=True)

# Via CLI
uvx exarp analyze-todo2-alignment /path/to/project [--dry-run]
```

**Output**: Alignment analysis report with misaligned tasks and recommendations.

**Example**:
```bash
$ uvx exarp analyze-todo2-alignment .
🎯 Todo2 Alignment Analysis
✅ Analyzed 307 tasks
⚠️  5 tasks misaligned with project goals
📋 Recommendations provided
```

---

### 3. Duplicate Task Detection

**Tool**: `mcp_exarp_detect_duplicate_tasks`

**Purpose**: Detects duplicate tasks in Todo2 system.

**Usage**:
```python
# Via MCP tool (in Cursor chat)
mcp_exarp_detect_duplicate_tasks(auto_fix=True, similarity_threshold=0.85)

# Via CLI
uvx exarp detect-duplicate-tasks /path/to/project [--auto-fix] [--dry-run]
```

**Output**: Duplicate detection report with duplicate groups and auto-fix results.

**Example**:
```bash
$ uvx exarp detect-duplicate-tasks . --auto-fix
🔍 Duplicate Task Detection
✅ Found 95 duplicate issues
🔄 Auto-fixed 12 duplicate tasks
📋 83 duplicates require manual review
```

---

## Wrapper Script

### Exarp Daily Automation Wrapper

**Script**: `scripts/exarp_daily_automation_wrapper.py`

**Purpose**: Orchestrates all three Exarp MCP tools in a single script.

**Usage**:
```bash
# Run all Exarp checks
python3 scripts/exarp_daily_automation_wrapper.py /path/to/project

# Dry-run mode
python3 scripts/exarp_daily_automation_wrapper.py /path/to/project --dry-run

# JSON output
python3 scripts/exarp_daily_automation_wrapper.py /path/to/project --json

# Auto-fix duplicates
python3 scripts/exarp_daily_automation_wrapper.py /path/to/project --auto-fix
```

**Features**:
- ✅ Calls all three Exarp tools
- ✅ Generates combined report
- ✅ Handles errors gracefully
- ✅ Supports dry-run mode
- ✅ JSON output option
- ✅ Proper exit codes

**Example Output**:
```
🚀 Starting Exarp daily automation...
Project directory: /home/david/ib_box_spread_full_universal

📚 Task 1: Checking documentation health...
✅ Documentation Health: Success

🎯 Task 2: Analyzing Todo2 alignment...
✅ Todo2 Alignment: Success

🔍 Task 3: Detecting duplicate tasks...
✅ Duplicate Detection: Success

======================================================================
📊 Summary:
   Tasks completed: 3
   Tasks succeeded: 3
   Tasks failed: 0
   ✅ All tasks completed successfully
======================================================================
```

---

## Integration with Daily Automation

### Enhanced Daily Automation Script

**Script**: `scripts/daily_automation_with_link_fixing.sh`

**Purpose**: Comprehensive daily automation including Exarp checks and documentation automation.

**Usage**:
```bash
# Run all automation tasks
./scripts/daily_automation_with_link_fixing.sh

# Dry-run mode
./scripts/daily_automation_with_link_fixing.sh . --dry-run

# Custom project directory
./scripts/daily_automation_with_link_fixing.sh /path/to/project
```

**Tasks Executed**:

**Phase 1: Exarp Checks**
1. Documentation health check
2. Todo2 alignment analysis
3. Duplicate task detection

**Phase 2: Documentation Automation**
4. Documentation link fixing
5. Documentation format validation
6. Shared TODO table synchronization

**Example Output**:
```
🚀 Starting daily automation tasks...
Project directory: /home/david/ib_box_spread_full_universal

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📦 Phase 1: Exarp Daily Automation Checks
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🚀 Starting Exarp daily automation...
...
✅ Exarp automation checks completed

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📚 Phase 2: Documentation Automation
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📝 Task 1: Fixing documentation links...
✅ Documentation links fixed

📋 Task 2: Validating documentation format...
✅ Documentation format validated

🔄 Task 3: Synchronizing shared TODO table...
✅ Shared TODO table synchronized

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 Daily Automation Summary
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ All tasks completed successfully!

Reports saved to:
  - /tmp/exarp_automation.log (Exarp checks)
  - /tmp/link_fix.log (Link fixing)
  - /tmp/format_validation.log (Format validation)
  - /tmp/todo_sync.log (TODO synchronization)
```

---

## Scheduling

### Cron Setup

To run daily automation automatically:

```bash
# Edit crontab
crontab -e

# Add daily automation (runs at 2 AM)
0 2 * * * /home/david/ib_box_spread_full_universal/scripts/daily_automation_with_link_fixing.sh /home/david/ib_box_spread_full_universal >> /tmp/daily_automation.log 2>&1
```

### Systemd Timer (Alternative)

Create `/etc/systemd/user/daily-automation.timer`:
```ini
[Unit]
Description=Daily Automation Timer

[Timer]
OnCalendar=daily
OnCalendar=02:00
Persistent=true

[Install]
WantedBy=timers.target
```

Create `/etc/systemd/user/daily-automation.service`:
```ini
[Unit]
Description=Daily Automation Service

[Service]
Type=oneshot
ExecStart=/home/david/ib_box_spread_full_universal/scripts/daily_automation_with_link_fixing.sh /home/david/ib_box_spread_full_universal
```

---

## Troubleshooting

### Issue: "uvx or exarp not found"

**Solution**: Install `uvx`:
```bash
# Install uv (includes uvx)
curl -LsSf https://astral.sh/uv/install.sh | sh

# Verify installation
uvx --version
```

### Issue: "Script not found" errors

**Solution**: Use individual MCP tools or wrapper script instead of aggregate daily automation:
```bash
# ✅ Works: Use wrapper script
python3 scripts/exarp_daily_automation_wrapper.py .

# ✅ Works: Use individual tools
uvx exarp check-documentation-health .
uvx exarp analyze-todo2-alignment .
uvx exarp detect-duplicate-tasks .

# ❌ Doesn't work: Aggregate daily automation
# mcp_exarp_run_daily_automation()  # Still has script discovery issue
```

### Issue: Tasks timing out

**Solution**: Increase timeout in wrapper script or run tasks individually:
```python
# In scripts/exarp_daily_automation_wrapper.py
timeout=600  # Increase from 300 to 600 seconds
```

---

## Best Practices

1. **Use Wrapper Script**: Prefer `exarp_daily_automation_wrapper.py` over calling tools individually
2. **Dry-Run First**: Always test with `--dry-run` before applying changes
3. **Monitor Logs**: Check `/tmp/exarp_automation.log` for detailed output
4. **Schedule Regularly**: Set up cron or systemd timer for daily execution
5. **Review Reports**: Check summary reports after each run

---

## Related Documentation

- `docs/EXARP_SCRIPT_PATH_ISSUE_RESOLVED.md` - Issue resolution and workarounds
- `docs/EXARP_SCRIPT_PATH_TODO2_PLAN.md` - Todo2 execution plan
- `docs/DAILY_AUTOMATION_SETUP_COMPLETE.md` - Daily automation setup guide

---

**Last Updated**: 2025-11-29  
**Status**: Active
