# MCP Prompt Names - Shortened for Cursor Popup

## Summary

Prompt names have been shortened to fit better in Cursor's chat popup (triggered by `/`). All prompts are now 10 characters or less.

---

## Name Changes

| Old Name | New Name | Length | Description |
|----------|----------|--------|-------------|
| `doc_health_check` | `doc_check` | 9 | Documentation health check |
| `doc_quick_check` | `doc_quick` | 9 | Quick doc check (no tasks) |
| `task_alignment` | `align` | 5 | Task alignment analysis |
| `duplicate_cleanup` | `dups` | 4 | Find duplicate tasks |
| `task_sync` | `sync` | 4 | Sync tasks across systems |
| `security_scan_all` | `scan` | 4 | Scan all dependencies |
| `security_scan_python` | `scan_py` | 7 | Scan Python dependencies |
| `security_scan_rust` | `scan_rs` | 7 | Scan Rust dependencies |
| `automation_discovery` | `auto` | 4 | Discover automation opportunities |
| `automation_high_value` | `auto_high` | 9 | High-value automation only |
| `pwa_review` | `pwa` | 3 | Review PWA configuration |
| `pre_sprint_cleanup` | `pre_sprint` | 9 | Pre-sprint cleanup workflow |
| `post_implementation_review` | `post_impl` | 9 | Post-implementation review |
| `weekly_maintenance` | `weekly` | 6 | Weekly maintenance workflow |

---

## Usage

### In Chat

Type `/` in Cursor chat to see all available prompts. The shortened names make them easier to scan and select.

**Examples:**
- `/doc_check` - Full documentation health check
- `/align` - Check task alignment
- `/dups` - Find duplicate tasks
- `/scan` - Security scan all dependencies
- `/auto` - Discover automation opportunities
- `/weekly` - Run weekly maintenance

### Via AI Chat

You can still reference prompts by their old names or new names:
- "Use the doc_check prompt" ✅
- "Use the doc_health_check prompt" ✅ (still works via description matching)

---

## Benefits

1. **Fits in Popup:** All names ≤ 10 characters, perfect for Cursor's popup
2. **Easy to Type:** Shorter names are faster to type
3. **Still Clear:** Names remain descriptive enough to understand purpose
4. **Backward Compatible:** Old names still work via description matching

---

## Prompt Reference

### Documentation
- `/doc_check` - Full documentation health analysis
- `/doc_quick` - Quick documentation check

### Tasks
- `/align` - Task alignment analysis
- `/dups` - Duplicate task detection
- `/sync` - Task synchronization

### Security
- `/scan` - Scan all dependencies
- `/scan_py` - Scan Python only
- `/scan_rs` - Scan Rust only

### Automation
- `/auto` - Discover automation opportunities
- `/auto_high` - High-value automation only

### PWA
- `/pwa` - Review PWA configuration

### Workflows
- `/pre_sprint` - Pre-sprint cleanup
- `/post_impl` - Post-implementation review
- `/weekly` - Weekly maintenance

---

**Status:** ✅ All prompt names shortened to ≤ 10 characters
