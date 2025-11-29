# Automation Integration Guide

**Date**: 2025-11-29  
**Purpose**: Guide for integrating automation scripts with Exarp MCP tools

---

## Overview

This guide explains how to integrate project automation scripts with Exarp MCP server for automated task execution.

---

## ✅ Available Automation Scripts

### 1. Documentation Link Fixing

**Script**: `scripts/automate_documentation_link_fixing.py`  
**Purpose**: Automatically fixes broken documentation links using path-based and name-based matching

**Usage**:
```bash
# Dry run (default)
python3 scripts/automate_documentation_link_fixing.py

# Apply fixes
python3 scripts/automate_documentation_link_fixing.py --apply

# Generate JSON report
python3 scripts/automate_documentation_link_fixing.py --apply --output report.json
```

**Features**:
- Combines both link fixing approaches
- Dry-run mode for safety
- JSON report generation
- Statistics tracking

**Integration Points**:
- Git hooks (pre-commit/post-commit)
- File watchers (on docs changes)
- Daily automation (scheduled)

---

### 2. Documentation Format Validation

**Script**: `scripts/validate_docs_format.py`  
**Purpose**: Validates API documentation entry format

**Usage**:
```bash
python3 scripts/validate_docs_format.py
```

**Features**:
- Validates required fields
- Checks recommended fields
- Validates URL format
- Color-coded output

**Integration Points**:
- Git hooks (pre-commit)
- CI/CD pipelines
- File watchers (on API docs changes)

---

## 🔌 Exarp Integration Options

### Option 1: Direct Script Execution (Current)

**How it works**:
- Exarp tools can call external scripts via subprocess
- Scripts return JSON reports
- Exarp processes and displays results

**Example Integration**:
```python
# In Exarp tool implementation
import subprocess
import json

def fix_documentation_links(dry_run=True):
    cmd = [
        'python3',
        'scripts/automate_documentation_link_fixing.py',
        '--dry-run' if dry_run else '--apply',
        '--output', '/tmp/link_fix_report.json'
    ]
    result = subprocess.run(cmd, capture_output=True, text=True)
    with open('/tmp/link_fix_report.json') as f:
        report = json.load(f)
    return report
```

---

### Option 2: Wrapper Script for Exarp

**How it works**:
- Create wrapper scripts that Exarp can call directly
- Wrappers handle Exarp-specific formatting
- Return structured data for Exarp processing

**Example Wrapper**:
```python
#!/usr/bin/env python3
# scripts/exarp_wrappers/fix_documentation_links.py

import sys
import json
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from automate_documentation_link_fixing import DocumentationLinkFixer

def main():
    dry_run = '--apply' not in sys.argv
    docs_dir = Path('docs')
    
    fixer = DocumentationLinkFixer(docs_dir, dry_run=dry_run)
    report = fixer.run()
    
    # Output JSON for Exarp
    print(json.dumps(report, indent=2))
    sys.exit(0 if report['status'] == 'success' else 1)

if __name__ == '__main__':
    main()
```

---

### Option 3: Exarp Custom Tool (Requires Exarp Modification)

**How it works**:
- Add new tool to Exarp MCP server
- Tool calls project scripts internally
- Provides Exarp-native interface

**Requirements**:
- Access to Exarp source code
- Ability to modify Exarp tools
- Exarp supports custom tool registration

**Example Tool Definition**:
```python
# In Exarp tool registry
{
    "name": "fix_documentation_links",
    "description": "Automatically fix broken documentation links",
    "parameters": {
        "dry_run": {"type": "boolean", "default": True},
        "apply": {"type": "boolean", "default": False}
    },
    "script": "scripts/automate_documentation_link_fixing.py"
}
```

---

## 🎯 Recommended Integration Approach

### Phase 1: Script-Based Integration (Immediate)

**Use**: Direct script execution via Exarp's existing infrastructure

**Steps**:
1. ✅ Scripts already exist and work
2. Create wrapper functions in Exarp (if possible)
3. Add to daily automation workflow
4. Test integration

**Pros**:
- No Exarp modification needed
- Quick to implement
- Works with current Exarp version

**Cons**:
- Requires Exarp to support external script execution
- Less integrated than native tools

---

### Phase 2: Git Hooks Integration (Short-Term)

**Use**: Git hooks to trigger automation

**Implementation**:
```bash
# .git/hooks/pre-commit
#!/bin/bash
python3 scripts/validate_docs_format.py || exit 1
python3 scripts/automate_documentation_link_fixing.py --dry-run
```

**Pros**:
- Catches issues before commit
- No Exarp dependency
- Works for all developers

**Cons**:
- Requires manual hook setup
- Can slow down commits

---

### Phase 3: File Watcher Integration (Medium-Term)

**Use**: File watchers to trigger automation on changes

**Implementation**:
```python
# Watch docs/ directory for changes
# On change: Run link fixing (dry-run)
# On save: Run format validation
```

**Pros**:
- Real-time feedback
- Automatic execution
- No manual intervention

**Cons**:
- Requires file watcher setup
- May impact performance

---

## 📋 Integration Checklist

### Documentation Link Fixing

- [x] Create unified automation script
- [ ] Test script with Exarp (if possible)
- [ ] Add to daily automation workflow
- [ ] Set up git hooks (optional)
- [ ] Set up file watchers (optional)
- [ ] Document integration

### Documentation Format Validation

- [x] Script exists and works
- [ ] Test with Exarp (if possible)
- [ ] Add to pre-commit hooks
- [ ] Add to CI/CD pipeline
- [ ] Document integration

### Shared TODO Table Sync

- [ ] Create sync script
- [ ] Test with Exarp
- [ ] Add to daily automation
- [ ] Set up file watcher (on Todo2 changes)
- [ ] Document integration

---

## 🚀 Quick Start

### Test Documentation Link Fixing

```bash
# Dry run
python3 scripts/automate_documentation_link_fixing.py

# Apply fixes
python3 scripts/automate_documentation_link_fixing.py --apply

# With report
python3 scripts/automate_documentation_link_fixing.py --apply --output report.json
```

### Test Format Validation

```bash
python3 scripts/validate_docs_format.py
```

### Integrate with Daily Automation

Add to daily automation script or Exarp daily automation:
```python
# Run link fixing (apply mode)
subprocess.run([
    'python3',
    'scripts/automate_documentation_link_fixing.py',
    '--apply'
])

# Run format validation
subprocess.run([
    'python3',
    'scripts/validate_docs_format.py'
])
```

---

## 📊 Expected Results

### Documentation Link Fixing

**Before Automation**:
- 186 broken links
- Manual fixing required
- Links accumulate

**After Automation**:
- < 50 broken links maintained
- Automatic fixing
- No manual intervention

**Frequency**: Daily or on documentation changes

### Format Validation

**Before Automation**:
- Format errors discovered late
- Manual validation
- Inconsistent format

**After Automation**:
- Early error detection
- Automatic validation
- Consistent format

**Frequency**: On documentation changes or pre-commit

---

## 🔧 Troubleshooting

### Script Not Found

**Issue**: Exarp can't find scripts  
**Solution**: Use absolute paths or ensure scripts are in PATH

### Permission Denied

**Issue**: Scripts not executable  
**Solution**: `chmod +x scripts/automate_documentation_link_fixing.py`

### JSON Parse Error

**Issue**: Report format incorrect  
**Solution**: Check script output format, ensure valid JSON

### Integration Not Working

**Issue**: Exarp doesn't support external scripts  
**Solution**: Use git hooks or file watchers instead

---

## 📝 Notes

- All scripts support `--dry-run` mode for safety
- Reports are generated in JSON format for programmatic access
- Scripts can be run independently of Exarp
- Integration is optional but recommended

---

**Last Updated**: 2025-11-29  
**Status**: Ready for integration
