# Dependency Security Scan Automation

**Date**: 2025-11-20
**Purpose**: Automate daily scanning of Python, Rust, and npm dependencies for known vulnerabilities

---

## Overview

This automation scans all project dependencies for known security vulnerabilities using multiple tools:
- **osv-scanner**: Multi-language vulnerability scanner (already in Trunk config)
- **pip-audit**: Python-specific vulnerability scanner
- **cargo-audit**: Rust-specific vulnerability scanner
- **npm audit**: npm-specific vulnerability scanner

---

## Features

### Multi-Language Support
- **Python**: Scans `requirements.txt` and `pyproject.toml` files
- **Rust**: Scans `Cargo.toml` files
- **npm**: Scans `package.json` files

### Vulnerability Detection
- Detects known CVEs and security advisories
- Categorizes by severity (critical, high, medium, low)
- Tracks trends over time
- Creates Todo2 tasks for high-priority vulnerabilities

### Reporting
- Generates detailed markdown reports
- Tracks vulnerability trends
- Provides actionable recommendations

---

## Installation

### Prerequisites

Install scanning tools (optional - script handles missing tools gracefully):

```bash
# osv-scanner (recommended - already in Trunk)
trunk install osv-scanner
# OR
brew install osv-scanner

# pip-audit (Python)
pip install pip-audit

# cargo-audit (Rust)
cargo install cargo-audit
```

**Note**: The script will work even if some tools are missing - it will use available tools and report which ones are unavailable.

### Setup

1. **Configure** (optional - defaults work):
   ```bash
   # Edit configuration if needed
   vim scripts/dependency_security_config.json
   ```

2. **Set up cron job** (recommended):
   ```bash
   ./scripts/setup_dependency_security_cron.sh
   ```

3. **Test manually**:
   ```bash
   python3 scripts/automate_dependency_security.py
   ```

---

## Usage

### Manual Execution

```bash
# Run with default config
python3 scripts/automate_dependency_security.py

# Run with custom config
python3 scripts/automate_dependency_security.py --config path/to/config.json

# Dry-run mode (no file changes)
python3 scripts/automate_dependency_security.py --dry-run
```

### Automated Execution

The cron job runs daily at 6 AM and:
1. Scans all dependencies
2. Generates report: `docs/DEPENDENCY_SECURITY_REPORT.md`
3. Creates Todo2 tasks for critical/high vulnerabilities
4. Logs to: `scripts/dependency_security_cron.log`

---

## Configuration

Edit `scripts/dependency_security_config.json`:

```json
{
  "scan_configs": {
    "python": {
      "enabled": true,
      "files": ["requirements.txt", "python/pyproject.toml"],
      "tools": {
        "osv_scanner": {"enabled": true},
        "pip_audit": {"enabled": true}
      }
    },
    "rust": {
      "enabled": true,
      "files": ["agents/backend/Cargo.toml"],
      "tools": {
        "cargo_audit": {"enabled": true},
        "osv_scanner": {"enabled": true}
      }
    },
    "npm": {
      "enabled": true,
      "files": ["web/package.json"],
      "tools": {
        "npm_audit": {"enabled": true},
        "osv_scanner": {"enabled": true}
      }
    }
  },
  "create_todo2_tasks": {
    "enabled": true,
    "min_severity": "high",
    "max_tasks": 10
  }
}
```

---

## Output

### Report File

`docs/DEPENDENCY_SECURITY_REPORT.md` contains:
- Executive summary
- Summary statistics (total, by severity, by language)
- Critical vulnerabilities (detailed)
- All vulnerabilities by language
- Recommendations

### Log Files

- `scripts/dependency_security.log`: General execution log
- `scripts/dependency_security_cron.log`: Cron execution log
- `scripts/dependency_security_cron_error.log`: Cron error log

### History File

`scripts/.dependency_security_history.json` tracks vulnerability trends over time (last 90 days).

---

## Todo2 Integration

The automation automatically creates Todo2 tasks for:
- **Critical** vulnerabilities (always)
- **High** vulnerabilities (configurable)
- Limited to 10 tasks per run (configurable)

Tasks include:
- Package name and version
- Vulnerability ID
- Severity level
- File location
- Description

---

## Troubleshooting

### Tools Not Found

If tools are missing, the script will:
- Log warnings
- Continue with available tools
- Report which tools are unavailable

**Solution**: Install missing tools (see Installation section).

### False Positives

Some vulnerability databases may have false positives:
- Review each vulnerability manually
- Check if updates are available
- Verify vulnerability applies to your usage

### Rate Limiting

Some vulnerability APIs may rate limit:
- Script includes timeouts
- Retry logic can be added if needed
- Consider caching results

---

## Best Practices

1. **Review reports regularly**: Check `docs/DEPENDENCY_SECURITY_REPORT.md` weekly
2. **Update dependencies**: Fix vulnerabilities by updating packages
3. **Monitor trends**: Watch for increasing vulnerability counts
4. **Prioritize critical**: Address critical vulnerabilities immediately
5. **Automate updates**: Consider automated dependency updates (separate automation)

---

## Integration with Other Automations

This automation works alongside:
- **Linter Automation**: Catches code quality issues
- **Test Coverage**: Ensures tests cover security-critical code
- **Build Health**: Monitors build system health

---

## Success Metrics

- **Vulnerability Detection**: 100% of known vulnerabilities detected
- **False Positive Rate**: < 5% (manual review)
- **Response Time**: Critical vulnerabilities addressed within 24 hours
- **Coverage**: All dependency files scanned

---

## Future Enhancements

Potential improvements:
1. **Automated Updates**: Create PRs for safe dependency updates
2. **License Scanning**: Check for license compliance issues
3. **Dependency Graph**: Visualize dependency relationships
4. **Custom Rules**: Define project-specific vulnerability rules
5. **Integration Alerts**: Send alerts to Slack/email for critical issues

---

*This automation is part of the comprehensive repository health monitoring system.*
