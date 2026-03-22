# Documentation Health Automation Guide

**Date**: 2025-11-20
**Purpose**: Guide for setting up and using automated documentation health monitoring

---

## Overview

The automated documentation health system monitors documentation quality across multiple dimensions:

- **Link Validity**: Internal and external links
- **Format Compliance**: Markdown syntax and template adherence
- **Content Completeness**: Required sections and fields
- **Date Currency**: Stale document detection
- **Cross-Reference Integrity**: Reference validation and orphan detection

This runs on a schedule (cron) to keep documentation healthy and catch issues early.

---

## Quick Start

### 1. Manual Run

Run the analysis script manually:

```bash

# Basic run

python3 scripts/automate_docs_health.py

# With custom config

python3 scripts/automate_docs_health.py --config scripts/docs_health_config.json

# With custom output path

python3 scripts/automate_docs_health.py --output docs/my_health_report.md
```

### 2. Scheduled Automation

Set up a cron job to run automatically:

```bash

# Run weekly on Tuesday at 2:00 AM (default)

./scripts/setup_docs_health_cron.sh weekly tuesday 02:00

# Run daily at 4:00 AM

./scripts/setup_docs_health_cron.sh daily 04:00

# Run monthly on the 1st at 3:00 AM

./scripts/setup_docs_health_cron.sh monthly 1 03:00
```

**View cron jobs:**

```bash
crontab -l
```

**Remove cron job:**

```bash
crontab -l | grep -v 'run_docs_health_cron.sh' | crontab -
```

**Logs:**

- Success logs: `scripts/docs_health.log`
- Error logs: `scripts/docs_health_errors.log`

---

## Configuration

### Configuration File: `scripts/docs_health_config.json`

```json
{
  "stale_threshold_days": 90,
  "output_path": "docs/DOCUMENTATION_HEALTH_REPORT.md"
}
```

**Options:**

- `stale_threshold_days`: Number of days before a document is considered stale (default: 90)
- `output_path`: Path to write the health report

---

## What the Script Does

### 1. Link Validation

- **Internal Links**: Checks that referenced files exist
- **External Links**: Validates HTTP accessibility (with timeout)
- **Skipped**: Email links, anchors, local paths

**Output**: List of broken internal and external links

### 2. Format Validation

- **API Documentation**: Validates entries in `API_DOCUMENTATION_INDEX.md`
- **Required Fields**: Checks for minimum required fields
- **Recommended Fields**: Warns about missing recommended fields
- **URL Format**: Validates URL formatting (angle brackets)

**Output**: Format errors and missing fields

### 3. Date Currency

- **Last Updated Dates**: Parses "Last Updated" or "Date:" fields
- **Stale Detection**: Flags documents older than threshold (default: 90 days)
- **Missing Dates**: Identifies key documents without dates

**Output**: List of stale documents and missing dates

### 4. Cross-Reference Validation

- **Reference Resolution**: Validates all internal document references
- **Orphan Detection**: Identifies documents with no incoming links
- **Broken References**: Flags references to non-existent files

**Output**: Broken references and orphaned files

### 5. Trend Tracking

- **Historical Data**: Stores results from each run
- **Score Trends**: Tracks health score over time
- **Issue Trends**: Monitors broken links, format errors, etc.
- **Comparison**: Compares current vs previous run

**Output**: Trend analysis in report

### 6. Health Score Calculation

**Overall Score (0-100%)** based on:

- **40%**: Link Health (broken links ratio)
- **20%**: Format Health (format errors)
- **20%**: Date Currency (stale documents)
- **20%**: Cross-Reference Health (broken/orphaned references)

**Target: 80%+ health score**

---

## Output

The script generates/updates `docs/DOCUMENTATION_HEALTH_REPORT.md` with:

- Executive summary with health score
- Link validation results
- Format validation results
- Date currency analysis
- Cross-reference validation
- Trend analysis
- Recommendations for improvement

### Report Sections

1. **Executive Summary**: Overall health score and key metrics
2. **Link Validation**: Broken internal and external links
3. **Format Validation**: Format errors and missing fields
4. **Date Currency**: Stale documents and missing dates
5. **Cross-Reference Validation**: Broken references and orphaned files
6. **Trends**: Historical comparison and trends
7. **Recommendations**: Actionable steps to improve health

---

## Troubleshooting

### Script Takes Too Long

**Issue**: External link checking is slow

**Solutions**:

- External links are checked with 10-second timeout
- Consider running with limited external link checking
- Network issues may cause delays

**Workaround**: Run with timeout:

```bash
timeout 300 python3 scripts/automate_docs_health.py
```

### False Positives for External Links

**Issue**: Some external links fail but are actually valid

**Causes**:

- Rate limiting
- Temporary network issues
- Sites requiring authentication

**Solution**: Review failed links manually, add to skip patterns if needed

### Missing History File

**Issue**: `scripts/.docs_health_history.json` not found

**Solution**: File is created automatically on first run. If missing, script will start fresh.

### Format Validation Errors

**Issue**: Format validation reports errors

**Solution**:

- Review `scripts/validate_docs_format.py` output
- Fix entries in `API_DOCUMENTATION_INDEX.md`
- Ensure entries follow template in `API_DOCUMENTATION_ENTRY_TEMPLATE.md`

---

## Best Practices

1. **Review Reports Regularly**: Check health reports weekly
2. **Fix Issues Promptly**: Address broken links and format errors quickly
3. **Update Dates**: Keep "Last Updated" dates current
4. **Monitor Trends**: Watch for declining health scores
5. **Archive Stale Docs**: Remove or update outdated content
6. **Maintain Cross-References**: Keep document references accurate

---

## Integration with Other Tools

### Combine with Other Automations

Run multiple health checks together:

```bash

# Create combined script

cat > scripts/run_all_health_checks.sh << 'EOF'

#!/bin/bash

python3 scripts/automate_pwa_review.py
python3 scripts/automate_todo2_alignment.py
python3 scripts/automate_docs_health.py
EOF
chmod +x scripts/run_all_health_checks.sh
```

### Pre-commit Hook

Add to pre-commit to check documentation before commits:

```bash

# In .git/hooks/pre-commit

python3 scripts/automate_docs_health.py --output /tmp/docs_check.md

# Check health score and warn if below threshold
```

---

## Advanced Usage

### Custom Validation Rules

Modify `scripts/automate_docs_health.py` to:

- Add custom validation rules
- Check additional file types
- Validate specific document structures
- Integrate with other tools

### Integration with Notification Systems

**Combine with notification systems:**

- Add email notifications on low health scores
- Send Slack messages with summary
- Create GitHub issues for broken links

**Example:**

```python

# In automate_docs_health.py

if health_score < 70:
    send_notification(f"Documentation health dropped to {health_score}%")
```

---

## References

- `scripts/automate_docs_health.py` - Main analysis script
- `scripts/docs_health_config.json` - Configuration file
- `scripts/setup_docs_health_cron.sh` - Cron setup script
- `scripts/validate_docs_links.sh` - Link validation script (integrated)
- `scripts/validate_docs_format.py` - Format validation script (integrated)
- `docs/DOCUMENTATION_HEALTH_REPORT.md` - Generated health report
- `docs/API_DOCUMENTATION_ENTRY_TEMPLATE.md` - API documentation template

---

*This automation helps keep documentation healthy and maintainable automatically.*
