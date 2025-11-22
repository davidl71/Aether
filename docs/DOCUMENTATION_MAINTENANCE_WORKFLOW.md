# Documentation Maintenance Workflow

**Date**: 2025-01-27
**Purpose**: Standardized workflow for maintaining API documentation

---

## Overview

This document defines the workflow for maintaining `API_DOCUMENTATION_INDEX.md` and related documentation files. It includes checklists, validation procedures, and automation tools.

---

## Maintenance Checklist

### When Adding a New API

- [ ] **Check if API already exists** in `API_DOCUMENTATION_INDEX.md`
- [ ] **Follow entry template** from `API_DOCUMENTATION_ENTRY_TEMPLATE.md`
- [ ] **Include required fields**:
  - [ ] Website URL
  - [ ] Description
  - [ ] Relevance to Box Spread Trading
- [ ] **Include recommended fields**:
  - [ ] Key Features
  - [ ] API Types
  - [ ] Integration Considerations
  - [ ] Use Cases
- [ ] **Format URLs** using angle brackets: `<https://example.com>`
- [ ] **Add to appropriate section** (or create new section if needed)
- [ ] **Update comparison tables** if applicable
- [ ] **Run validation scripts**:
  - [ ] `./scripts/validate_docs_links.sh`
  - [ ] `./scripts/validate_docs_format.py`
- [ ] **Update summary document** if needed (`API_DOCUMENTATION_SUMMARY.md`)
- [ ] **Update topic indices** if applicable (`docs/indices/`)

### When Updating an Existing API

- [ ] **Check for version changes** (update version number if changed)
- [ ] **Update last-reviewed date** in metadata header
- [ ] **Verify links still work** (run `validate_docs_links.sh`)
- [ ] **Update comparison tables** if features changed
- [ ] **Check if deprecated** (add deprecation notice if applicable)
- [ ] **Update summary document** if significant changes

### Quarterly Review

- [ ] **Run all validation scripts**
- [ ] **Check for broken links** (fix or mark as deprecated)
- [ ] **Review deprecated APIs** (remove if no longer relevant)
- [ ] **Update version numbers** for all APIs
- [ ] **Review and update comparison tables**
- [ ] **Check metadata headers** are current
- [ ] **Update summary document** with any changes
- [ ] **Review topic indices** for accuracy

---

## Validation Scripts

### Link Validation

```bash
./scripts/validate_docs_links.sh
```

**What it does**:

- Checks all URLs in documentation files
- Validates HTTP status codes
- Reports broken links
- Skips local links, email links, and anchors

**When to run**:

- Before committing documentation changes
- During quarterly reviews
- When adding new APIs

### Format Validation

```bash
./scripts/validate_docs_format.py
```

**What it does**:

- Validates entry format against template
- Checks for required fields
- Warns about missing recommended fields
- Validates URL formatting

**When to run**:

- Before committing documentation changes
- When adding new APIs
- During code reviews

### Summary Table Generation

```bash
./scripts/generate_docs_summary_tables.py
```

**What it does**:

- Generates comparison tables from index
- Updates summary document
- Extracts provider information

**When to run**:

- After major documentation updates
- When adding multiple new APIs
- During quarterly reviews

---

## Pre-Commit Hooks

### Setup Pre-Commit Hooks

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash
# Pre-commit hook for documentation validation

DOCS_DIR="docs"
SCRIPTS_DIR="scripts"

# Check if documentation files changed
if git diff --cached --name-only | grep -q "$DOCS_DIR/API_DOCUMENTATION_INDEX.md"; then
  echo "🔍 Validating documentation..."

  # Run format validation
  if ! "$SCRIPTS_DIR/validate_docs_format.py"; then
    echo "❌ Documentation format validation failed"
    exit 1
  fi

  # Run link validation (non-blocking, just warn)
  if ! "$SCRIPTS_DIR/validate_docs_links.sh"; then
    echo "⚠️  Warning: Some documentation links may be broken"
    echo "   (This is non-blocking, but please fix before merging)"
  fi

  echo "✅ Documentation validation passed"
fi

exit 0
```

**Make executable**:

```bash
chmod +x .git/hooks/pre-commit
```

---

## CI/CD Integration

### GitHub Actions Example

Create `.github/workflows/docs-validation.yml`:

```yaml
name: Documentation Validation

on:
  pull_request:
    paths:
      - 'docs/API_DOCUMENTATION_INDEX.md'
      - 'docs/**/*.md'

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      - name: Validate Format
        run: ./scripts/validate_docs_format.py

      - name: Validate Links
        run: ./scripts/validate_docs_links.sh
        continue-on-error: true  # Non-blocking
```

---

## Version Tracking

### Adding Version Information

For APIs with version numbers, include in entry:

```markdown
### API Name

- **Version**: 1.2.3
- **Last Reviewed**: 2025-01-27
- **Status**: ✅ Active | ⚠️ Deprecated | ❌ Unmaintained
```

### Version Changelog

Consider maintaining a changelog for major API changes:

```markdown
### Version History

- **1.2.3** (2025-01-27): Added new endpoint
- **1.2.2** (2024-12-15): Bug fixes
- **1.2.1** (2024-11-01): Initial documentation
```

---

## Deprecation Process

### Marking APIs as Deprecated

1. **Add deprecation notice**:

   ```markdown
   - **Status**: ⚠️ Deprecated (as of 2025-01-27)
   - **Reason**: API no longer maintained / replaced by X
   - **Alternative**: [Link to alternative]
   ```

2. **Update comparison tables** to indicate deprecated status

3. **Move to deprecated section** (optional, after 6 months)

4. **Remove from active documentation** (after 12 months, if truly obsolete)

---

## Automation Opportunities

### Future Enhancements

1. **Automated Link Checking**
   - Scheduled job (weekly/monthly)
   - Email notifications for broken links
   - Auto-update status in documentation

2. **API Changelog Monitoring**
   - Monitor API changelogs for updates
   - Auto-generate update notifications
   - Suggest documentation updates

3. **Version Tracking**
   - Track API versions automatically
   - Alert on version changes
   - Generate version comparison reports

4. **Summary Table Auto-Generation**
   - Auto-generate comparison tables
   - Update summary document
   - Maintain consistency

---

## Maintenance Schedule

### Daily

- None (automated validation on commits)

### Weekly

- Review any broken link reports
- Check for new API announcements

### Monthly

- Run full validation suite
- Review deprecated APIs
- Update version numbers

### Quarterly

- Comprehensive documentation review
- Update comparison tables
- Review and update topic indices
- Archive or remove obsolete entries

---

## Troubleshooting

### Validation Scripts Fail

**Issue**: Format validation fails
**Solution**:

1. Check entry against template
2. Ensure required fields are present
3. Verify URL formatting

**Issue**: Link validation fails
**Solution**:

1. Check if URL is accessible
2. Verify URL format (angle brackets)
3. Check if URL requires authentication
4. Add to skip patterns if appropriate

### Documentation Conflicts

**Issue**: Merge conflicts in documentation
**Solution**:

1. Resolve conflicts manually
2. Run validation after resolution
3. Update summary if needed

---

## See Also

- **Entry Template**: `API_DOCUMENTATION_ENTRY_TEMPLATE.md`
- **Consolidation Plan**: `API_DOCUMENTATION_CONSOLIDATION_PLAN.md`
- **Status Review**: `DOCUMENTATION_STATUS_REVIEW.md`
- **Validation Scripts**: `scripts/validate_docs_*.sh` and `scripts/validate_docs_*.py`
