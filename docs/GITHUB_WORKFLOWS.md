# GitHub Actions Workflows

**Date**: 2025-11-30
**Status**: Active CI/CD Pipeline

---

## Overview

Exarp uses GitHub Actions for continuous integration, testing, linting, security scanning, and package publishing. All workflows are configured in `.github/workflows/`.

---

## Workflows

### 1. Test Workflow (`test.yml`)

**Purpose**: Run test suite across multiple Python versions and operating systems

**Triggers**:

- Push to `main` or `develop` branches
- Pull requests to `main` or `develop`
- Manual dispatch

**Matrix Strategy**:

- **OS**: Ubuntu Latest, macOS Latest
- **Python**: 3.9, 3.10, 3.11, 3.12

**Steps**:

1. Checkout repository
2. Set up Python (matrix version)
3. Install dependencies (`pip install -e ".[dev]"`)
4. Run tests (`pytest tests/ -v --tb=short`)
5. Upload coverage (Ubuntu + Python 3.11 only)

**Usage**:

```bash

# Tests run automatically on push/PR
# Manual trigger: GitHub Actions → Test → Run workflow
```

---

### 2. Lint Workflow (`lint.yml`)

**Purpose**: Check code style and quality

**Triggers**:

- Push to `main` or `develop` branches
- Pull requests to `main` or `develop`
- Manual dispatch

**Steps**:

1. Checkout repository
2. Set up Python 3.11
3. Install dependencies
4. Run Black (format check)
5. Run Ruff (linting)
6. Run MyPy (type checking)

**Usage**:

```bash

# Linting runs automatically on push/PR
# Manual trigger: GitHub Actions → Lint → Run workflow
```

**Tools**:

- **Black**: Code formatting check
- **Ruff**: Fast Python linter
- **MyPy**: Static type checking

---

### 3. Format Workflow (`format.yml`)

**Purpose**: Auto-format code and create PR with formatted changes

**Triggers**:

- Manual dispatch (recommended)
- Pull requests (optional)

**Steps**:

1. Checkout repository
2. Set up Python 3.11
3. Install dependencies
4. Run Black (format code)
5. Run Ruff (auto-fix)
6. Create Pull Request (if manual dispatch)

**Usage**:

```bash

# Manual trigger: GitHub Actions → Format → Run workflow
# Creates PR with auto-formatted code
```

---

### 4. Build Workflow (`build.yml`)

**Purpose**: Build Python package and verify build artifacts

**Triggers**:

- Push to `main` or `develop` branches
- Pull requests to `main` or `develop`
- Manual dispatch

**Steps**:

1. Checkout repository
2. Set up Python 3.11
3. Install build dependencies (`build`, `twine`)
4. Build package (`python -m build`)
5. Check package (`twine check dist/*`)
6. Upload build artifacts

**Usage**:

```bash

# Build runs automatically on push/PR
# Manual trigger: GitHub Actions → Build → Run workflow
```

**Artifacts**:

- `dist/` directory with built packages
- Retained for 7 days

---

### 5. Publish to PyPI (`publish-pypi.yml`)

**Purpose**: Publish package to PyPI using trusted publishing (OIDC)

**Triggers**:

- Release published (GitHub release)
- Manual dispatch (with version input)

**Steps**:

1. Checkout repository
2. Set up Python 3.9
3. Install build dependencies
4. Run tests (optional, can be skipped)
5. Build package
6. Check package (`twine check`)
7. Publish to PyPI (using OIDC trusted publishing)

**Environment**:

- `pypi` - PyPI trusted publishing environment

**Usage**:

```bash

# Automatic: Create GitHub release → triggers publish
# Manual: GitHub Actions → Publish to PyPI → Run workflow (with version)
```

**Setup**: This workflow documents the historical Python package publishing path. Keep it only if you still publish the legacy package.

**Reference**: [PyPI Trusted Publishing](https://docs.pypi.org/trusted-publishers/)

---

### 6. Security Scan (`security-scan.yml`)

**Purpose**: Scan for security vulnerabilities in dependencies and code

**Triggers**:

- Push to `main` or `develop` branches
- Pull requests to `main` or `develop`
- Weekly schedule (Mondays at midnight UTC)
- Manual dispatch

**Steps**:

1. Checkout repository
2. Set up Python 3.11
3. Install dependencies
4. Run Safety (dependency security scan)
5. Run Bandit (security linting)
6. Upload security reports

**Usage**:

```bash

# Runs automatically on push/PR and weekly
# Manual trigger: GitHub Actions → Security Scan → Run workflow
```

**Tools**:

- **Safety**: Dependency vulnerability scanning
- **Bandit**: Security linting for Python code

**Artifacts**:

- `bandit-report.json` - Security linting report
- Retained for 30 days

---

### 7. FastMCP Inspect (`fastmcp-inspect.yml`)

**Purpose**: Verify FastMCP configuration and server entrypoint

**Triggers**:

- Push to `main` or `develop` branches
- Pull requests to `main` or `develop`
- Manual dispatch

**Steps**:

1. Checkout repository
2. Set up Python 3.11
3. Install FastMCP CLI
4. Inspect FastMCP configuration (`fastmcp inspect fastmcp.json`)
5. Verify server entrypoint

**Usage**:

```bash

# Runs automatically on push/PR
# Manual trigger: GitHub Actions → FastMCP Inspect → Run workflow
```

**Purpose**: Ensures FastMCP configuration is valid and server can be discovered.

---

## Workflow Status Badges

The old `project-management-automation` badge examples below are obsolete. Use this repo's local workflow files under `.github/workflows/` instead.

Historical example:

```markdown
<!-- obsolete project-management-automation badge examples removed -->
```

---

## Workflow Dependencies

### Required for All Workflows

- **Python 3.11+** - Python workflow runtime
- **uv** - Python dependency/runtime manager
- **exarp-go** - MCP/task tooling used by this repo

### Development Dependencies

- **pytest** - Testing framework
- **pytest-mock** - Mocking for tests
- **black** - Code formatting
- **mypy** - Type checking
- **ruff** - Fast linting
- **safety** - Dependency security scanning
- **bandit** - Security linting
- **build** - Package building
- **twine** - Package publishing

---

## Workflow Best Practices

### 1. Matrix Testing

Test across multiple Python versions and operating systems to ensure compatibility:

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest]
    python-version: ['3.9', '3.10', '3.11', '3.12']
```

### 2. Caching

Use pip caching to speed up workflow runs:

```yaml

- uses: actions/setup-python@v5
  with:
    cache: 'pip'
```

### 3. Artifact Retention

Set appropriate retention periods for artifacts:

- Build artifacts: 7 days
- Security reports: 30 days

### 4. Conditional Steps

Use conditional steps to avoid unnecessary work:

```yaml

- name: Upload coverage
  if: matrix.os == 'ubuntu-latest' && matrix.python-version == '3.11'
```

### 5. Error Handling

Allow workflows to continue on non-critical failures:

```yaml

- name: Run MyPy
  run: |
    mypy exarp_project_management/ tools/ --ignore-missing-imports || true
```

---

## Workflow Triggers

### Automatic Triggers

- **Push to main/develop** - Test, Lint, Build, Security Scan, FastMCP Inspect
- **Pull requests** - Test, Lint, Build, Security Scan, FastMCP Inspect
- **Release published** - Publish to PyPI
- **Weekly schedule** - Security Scan (Mondays)

### Manual Triggers

All workflows support `workflow_dispatch` for manual execution:

- Test - Test specific scenarios
- Lint - Check code quality
- Format - Auto-format code
- Build - Verify package builds
- Security Scan - Run security checks
- FastMCP Inspect - Verify configuration
- Publish to PyPI - Manual release

---

## Workflow Status

### Current Status

- ✅ **Test** - Active, multi-version testing
- ✅ **Lint** - Active, code quality checks
- ✅ **Format** - Active, auto-formatting
- ✅ **Build** - Active, package verification
- ✅ **Publish to PyPI** - Active, trusted publishing
- ✅ **Security Scan** - Active, weekly + on push/PR
- ✅ **FastMCP Inspect** - Active, configuration verification

---

## Adding New Workflows

### Template

```yaml
name: Workflow Name

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]
  workflow_dispatch:

jobs:
  job-name:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.11'
          cache: 'pip'

      - name: Install dependencies
        run: |
          python -m pip install --upgrade pip
          pip install -e ".[dev]"

      - name: Run step
        run: |
          # Your commands here
```

---

## Troubleshooting

### Workflow Failures

1. **Test failures** - Check test output, verify Python version compatibility
2. **Lint failures** - Run `black` and `ruff` locally to fix issues
3. **Build failures** - Verify `pyproject.toml` is valid
4. **Publish failures** - Check PyPI trusted publishing configuration

### Common Issues

- **Import errors** - Verify package structure and imports
- **Type errors** - Run MyPy locally to identify issues
- **Security warnings** - Review Bandit and Safety reports

---

## Related Documentation

- [Contributing Guide](../CONTRIBUTING.md) - Development guidelines

---

**Last Updated**: 2025-11-30
