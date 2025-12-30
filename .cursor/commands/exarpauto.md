# Exarp Daily Automation

Run daily automation checks including documentation health, task alignment, duplicate detection, and security scanning.

## Usage

Run `/exarp/auto` in Cursor chat or use the command palette.

**Note**: If MCP server isn't responding, run manually:
```bash
cd ~/Projects/project-management-automation
PROJECT_ROOT=/Users/davidl/Projects/Trading/ib_box_spread_full_universal \
  python3 -m project_management_automation.scripts.automate_daily \
  --tasks docs_health todo2_alignment duplicate_detection \
  --output-path /Users/davidl/Projects/Trading/ib_box_spread_full_universal/docs/DAILY_AUTOMATION_REPORT.md
```

## What It Does

1. **Documentation Health Check** - Validates documentation structure, broken links, format issues
2. **Task Alignment Analysis** - Evaluates Todo2 task alignment with project goals
3. **Duplicate Task Detection** - Finds and reports duplicate tasks
4. **Security Scanning** - Scans dependencies for vulnerabilities (use `--include-slow`)

## Output

- Creates reports in `docs/` directory
- Optionally creates follow-up tasks for issues found
- Provides summary of findings

## Requirements

- Exarp MCP server must be running (or run manually from project-management-automation repo)
- Working directory: `/Users/davidl/Projects/Trading/ib_box_spread_full_universal`
- Scripts must be accessible from project-management-automation repository
