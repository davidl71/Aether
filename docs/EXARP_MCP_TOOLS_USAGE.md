# Exarp MCP Tools Usage Guide

**Date**: 2025-11-29
**Status**: Active

> Exarp in this repo is now provided by `exarp-go`. The old `uvx exarp` workflow is legacy.
> Use MCP chat tools or `./scripts/run_exarp_go_tool.sh <tool> <json_args>`.

---

## Overview

This guide documents the current `exarp-go` tool flow for this repo. Prefer MCP chat tools in Cursor or the local `scripts/run_exarp_go_tool.sh` wrapper for CLI usage.

---

## Available Tools

### 1. Documentation Health Check

**Tool**: `health`

**Purpose**: Checks documentation health, finds broken links, and validates documentation structure.

**Usage**:

```text
MCP chat: run the exarp-go `health` tool with `{"action":"docs"}`
CLI: ./scripts/run_exarp_go_tool.sh health '{"action":"docs"}'
```

**Output**: Documentation health report with broken links, format errors, and recommendations.

**Example**:

```bash
./scripts/run_exarp_go_tool.sh health '{"action":"docs"}'
```

---

### 2. Todo2 Alignment Analysis

**Tool**: `analyze_alignment`

**Purpose**: Analyzes Todo2 tasks for alignment with project goals and investment strategy framework.

**Usage**:

```text
MCP chat: run the exarp-go `analyze_alignment` tool
CLI: ./scripts/run_exarp_go_tool.sh analyze_alignment '{}'
```

**Output**: Alignment analysis report with misaligned tasks and recommendations.

**Example**:

```bash
./scripts/run_exarp_go_tool.sh analyze_alignment '{}'
```

---

### 3. Duplicate Task Detection

**Tool**: `task_analysis`

**Purpose**: Detects duplicate tasks in Todo2 system.

**Usage**:

```text
MCP chat: run the exarp-go `task_analysis` tool with `{"action":"duplicates"}`
CLI: ./scripts/run_exarp_go_tool.sh task_analysis '{"action":"duplicates"}'
```

**Output**: Duplicate detection report with duplicate groups and auto-fix results.

**Example**:

```bash
./scripts/run_exarp_go_tool.sh task_analysis '{"action":"duplicates"}'
```

---

## Current CLI Flow

Use the local wrapper directly:

```bash
./scripts/run_exarp_go_tool.sh health '{"action":"docs"}'
./scripts/run_exarp_go_tool.sh analyze_alignment '{}'
./scripts/run_exarp_go_tool.sh task_analysis '{"action":"duplicates"}'
./scripts/run_exarp_go_tool.sh scan_dependency_security '{}'
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

Reports should be written under repo-local artifact paths such as `build/` or `out/`, not `/tmp`.
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

1. **Use the repo wrapper**: Prefer `scripts/run_exarp_go_tool.sh` over legacy wrappers
2. **Dry-run or read-only first**: Start with non-mutating tool actions before applying changes
3. **Keep outputs repo-local**: Write reports under `out/` or `build/`, not `/tmp`
4. **Schedule regularly**: Set up cron or systemd timer for recurring checks if needed
5. **Review reports**: Check generated summaries after each run

---

## Related Documentation

- `docs/EXARP_GO_MIGRATION_LEFTOVERS.md` - Remaining migration cleanup notes
- `docs/DAILY_AUTOMATION_SETUP_COMPLETE.md` - Daily automation setup guide

---

**Last Updated**: 2025-11-29
**Status**: Active
