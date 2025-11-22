# Todo2 Duplicate Task Detection Automation

**Date**: 2025-11-22
**Status**: ✅ Complete
**Script**: `scripts/automate_todo2_duplicate_detection.py`

---

## Overview

Automated script to detect duplicate tasks in Todo2 by analyzing:
- **Duplicate Task IDs** (critical data integrity issue)
- **Exact Name Matches** (tasks with identical names)
- **Similar Name Matches** (fuzzy matching, configurable threshold)
- **Similar Description Matches** (tasks with similar long descriptions)
- **Self-Dependencies** (tasks that depend on themselves - invalid)

---

## Quick Start

### Run Once

```bash
# Basic run with default settings
python3 scripts/automate_todo2_duplicate_detection.py

# Custom similarity threshold (0.0-1.0)
python3 scripts/automate_todo2_duplicate_detection.py --threshold 0.90

# Custom output path
python3 scripts/automate_todo2_duplicate_detection.py --output docs/my_report.md
```

### Setup Automated Daily Run

```bash
# Daily at 9 AM
./scripts/setup_todo2_duplicate_detection_cron.sh daily 09:00

# Weekly on Mondays at 9 AM
./scripts/setup_todo2_duplicate_detection_cron.sh weekly monday 09:00

# Monthly on the 1st at 9 AM
./scripts/setup_todo2_duplicate_detection_cron.sh monthly 1 09:00
```

---

## Configuration

**File**: `scripts/todo2_duplicate_config.json`

```json
{
  "output_path": "docs/TODO2_DUPLICATE_DETECTION_REPORT.md",
  "similarity_threshold": 0.85,
  "auto_fix": false
}
```

### Configuration Options

- **`output_path`**: Where to write the detection report
- **`similarity_threshold`**: Similarity threshold for fuzzy matching (0.0-1.0)
  - `0.85` = 85% similarity (default, good balance)
  - `0.90` = 90% similarity (stricter, fewer false positives)
  - `0.80` = 80% similarity (looser, more potential matches)
- **`auto_fix`**: Automatically fix duplicates (experimental, not recommended)

---

## What It Detects

### 1. Duplicate Task IDs ⚠️ CRITICAL

Multiple tasks with the same ID. This should never happen and indicates a data integrity issue.

**Example**:
```
Task ID: T-199 (appears 2 times)
- T-199: Set up dependency management mechanism (Status: Review)
- T-199: Research financial data sources (Status: Review)
```

### 2. Exact Name Matches

Tasks with identical names but different IDs (likely duplicates).

**Example**:
```
Name: "Define public/private boundaries" (2 tasks)
- T-198: Define public/private boundaries (Status: Review)
- T-208: Define public/private boundaries (Status: Review)
```

### 3. Similar Name Matches

Tasks with similar names (fuzzy matching based on similarity threshold).

**Example**:
```
Similarity: 87.5%
- T-207: Reorganize private monorepo to use extracted libraries
- T-221: Reorganize private monorepo to use extracted libraries
```

### 4. Similar Description Matches

Tasks with similar long descriptions (potential duplicates with different names).

### 5. Self-Dependencies

Tasks that depend on themselves (invalid - tasks cannot depend on themselves).

**Example**:
```
- T-199: Set up dependency management mechanism
  Dependencies: T-199 (invalid!)
```

---

## Report Format

The script generates a comprehensive markdown report with:

1. **Summary**: Total tasks analyzed, duplicates found
2. **Critical Issues**: Duplicate IDs (must fix immediately)
3. **Exact Matches**: Tasks with identical names
4. **Similar Matches**: Tasks with similar names/descriptions
5. **Self-Dependencies**: Invalid dependency patterns
6. **Recommendations**: Action items to fix issues
7. **How to Fix**: Step-by-step guide

---

## Integration with Project Housekeeping Tools

This script is part of the **project-housekeeping-tools** collection:

- ✅ Uses `IntelligentAutomationBase` for consistency
- ✅ Integrates with Tractatus/Sequential Thinking (optional)
- ✅ Creates Todo2 tasks for tracking
- ✅ Follows same patterns as other automation scripts
- ✅ Can be scheduled via cron
- ✅ Generates markdown reports

**Related Scripts**:
- `automate_todo2_alignment_v2.py` - Task alignment analysis
- `automate_todo_sync.py` - Task synchronization
- `automate_docs_health_v2.py` - Documentation health

---

## Usage Examples

### Manual Run

```bash
# Check for duplicates
python3 scripts/automate_todo2_duplicate_detection.py

# View report
cat docs/TODO2_DUPLICATE_DETECTION_REPORT.md
```

### Automated Daily Check

```bash
# Setup cron job
./scripts/setup_todo2_duplicate_detection_cron.sh daily 09:00

# View cron log
tail -f scripts/todo2_duplicate_detection_cron.log
```

### Custom Threshold

```bash
# Stricter matching (90% similarity)
python3 scripts/automate_todo2_duplicate_detection.py --threshold 0.90

# Looser matching (80% similarity)
python3 scripts/automate_todo2_duplicate_detection.py --threshold 0.80
```

---

## Troubleshooting

### Report Not Generated

Check that the output directory exists:
```bash
mkdir -p docs
python3 scripts/automate_todo2_duplicate_detection.py
```

### Too Many False Positives

Increase the similarity threshold:
```bash
python3 scripts/automate_todo2_duplicate_detection.py --threshold 0.90
```

### Too Few Matches

Decrease the similarity threshold:
```bash
python3 scripts/automate_todo2_duplicate_detection.py --threshold 0.80
```

### Cron Job Not Running

Check cron logs:
```bash
# View cron log
tail -f scripts/todo2_duplicate_detection_cron.log

# Verify cron job is installed
crontab -l | grep duplicate
```

---

## Best Practices

1. **Run Daily**: Catch duplicates early before they accumulate
2. **Review Reports**: Don't just run the script - review the findings
3. **Fix Critical Issues First**: Duplicate IDs should be fixed immediately
4. **Use Appropriate Threshold**: 0.85 is a good default, adjust based on your needs
5. **Don't Auto-Fix**: Review duplicates manually before deleting

---

## Future Enhancements

Potential improvements:
- [ ] Auto-fix mode (with confirmation)
- [ ] Integration with GitHub Actions
- [ ] Email/Slack notifications for critical issues
- [ ] Historical tracking (track duplicate trends over time)
- [ ] Dependency graph visualization for duplicates

---

**Last Updated**: 2025-11-22
**Maintainer**: Project Housekeeping Tools
