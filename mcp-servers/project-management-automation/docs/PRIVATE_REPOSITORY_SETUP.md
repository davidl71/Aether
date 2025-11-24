# Private/Local Repository Setup for Automa

**Date**: 2025-01-27
**Purpose**: Setup private/local package distribution before PyPI publication

---

## Options for Private/Local Distribution

### Option 1: Git-Based Installation (Recommended for Development)

**Pros:**
- ✅ No server setup required
- ✅ Works with any Git repository (GitHub, GitLab, private)
- ✅ Easy version control
- ✅ Can use private repositories

**Cons:**
- ⚠️ Requires Git access
- ⚠️ Slightly slower than PyPI

**Installation:**
```bash
# From GitHub (public or private)
pip install git+https://github.com/username/project-management-automation.git

# From specific branch/tag
pip install git+https://github.com/username/project-management-automation.git@v0.1.0

# From local repository
pip install git+file:///path/to/project-management-automation
```

---

### Option 2: Local File Installation (Best for Testing)

**Pros:**
- ✅ No network required
- ✅ Fastest for local development
- ✅ Easy to test changes

**Cons:**
- ⚠️ Only works on local machine
- ⚠️ Not suitable for distribution

**Installation:**
```bash
# Editable install (development mode)
cd mcp-servers/project-management-automation
pip install -e .

# Regular install
pip install .
```

---

### Option 3: Local PyPI Server (pypiserver)

**Pros:**
- ✅ Mimics PyPI structure
- ✅ Can serve multiple packages
- ✅ Good for team distribution

**Cons:**
- ⚠️ Requires server setup
- ⚠️ More complex

**Setup:**
```bash
# Install pypiserver
pip install pypiserver passlib

# Create package directory
mkdir -p ~/packages

# Build and upload package
cd mcp-servers/project-management-automation
python -m build
twine upload --repository-url http://localhost:8080 dist/*

# Install from local server
pip install --index-url http://localhost:8080/simple project-management-automation-mcp
```

---

### Option 4: GitHub Packages (Private)

**Pros:**
- ✅ Integrated with GitHub
- ✅ Private by default
- ✅ No server setup

**Cons:**
- ⚠️ Requires GitHub account
- ⚠️ Slightly more complex setup

**Setup:**
```bash
# Configure GitHub token
export GITHUB_TOKEN=your_token

# Build package
python -m build

# Upload to GitHub Packages
twine upload --repository-url https://upload.pypi.org/legacy/ dist/* \
  --username __token__ \
  --password $GITHUB_TOKEN

# Install from GitHub Packages
pip install project-management-automation-mcp \
  --index-url https://pypi.org/simple \
  --extra-index-url https://github.com/username/packages/simple
```

---

## Recommended Approach: Git-Based Installation

For initial development and testing, **Git-based installation** is recommended:

### Setup Steps

1. **Create Git Repository** (if not exists)
   ```bash
   cd mcp-servers/project-management-automation
   git init
   git add .
   git commit -m "Initial commit"
   ```

2. **Push to Private Repository**
   ```bash
   # GitHub private repo
   git remote add origin https://github.com/username/project-management-automation.git
   git push -u origin main

   # Or GitLab private repo
   git remote add origin https://gitlab.com/username/project-management-automation.git
   git push -u origin main
   ```

3. **Install from Git**
   ```bash
   # From private GitHub repo
   pip install git+https://github.com/username/project-management-automation.git

   # With authentication (for private repos)
   pip install git+https://token@github.com/username/project-management-automation.git
   ```

---

## Installation Script

Create a helper script for easy installation:

```bash
#!/bin/bash
# install_automa.sh

REPO_URL="${AUTOMA_REPO_URL:-https://github.com/username/project-management-automation.git}"
BRANCH="${AUTOMA_BRANCH:-main}"

echo "Installing automa from $REPO_URL (branch: $BRANCH)"

pip install git+$REPO_URL@$BRANCH

echo "Installation complete!"
```

**Usage:**
```bash
export AUTOMA_REPO_URL="https://github.com/username/project-management-automation.git"
export AUTOMA_BRANCH="main"
./install_automa.sh
```

---

## Version Tagging Strategy

For Git-based installation, use tags for versioning:

```bash
# Tag current version
git tag v0.1.0
git push origin v0.1.0

# Install specific version
pip install git+https://github.com/username/project-management-automation.git@v0.1.0

# Install latest from main
pip install git+https://github.com/username/project-management-automation.git@main
```

---

## Local Development Workflow

### Development Mode (Editable Install)

```bash
cd mcp-servers/project-management-automation
pip install -e .

# Make changes to code
# Changes are immediately available (no reinstall needed)
```

### Testing Installation

```bash
# Build package
python -m build

# Install from built package
pip install dist/project_management_automation_mcp-0.1.0-py3-none-any.whl
```

---

## Migration Path to PyPI

When ready for public release:

1. **Test with private repository first**
2. **Create PyPI account**
3. **Update pyproject.toml** (if needed)
4. **Publish to PyPI:**
   ```bash
   python -m build
   twine upload dist/*
   ```

---

## Configuration Files

### .pypirc (for twine)

```ini
[distutils]
index-servers =
    local
    github
    pypi

[local]
repository = http://localhost:8080
username =
password =

[github]
repository = https://github.com/username/packages
username = __token__
password = your_github_token

[pypi]
repository = https://upload.pypi.org/legacy/
username = your_pypi_username
password = your_pypi_password
```

---

## Recommended Setup for This Project

**Phase 1: Local Development**
- Use editable install: `pip install -e .`
- Test locally before distribution

**Phase 2: Private Git Repository**
- Push to private GitHub/GitLab repo
- Install via: `pip install git+https://...`
- Use tags for versioning

**Phase 3: Public PyPI (Future)**
- After testing and refinement
- Publish to PyPI for public distribution

---

## Quick Start: Git-Based Installation

```bash
# 1. Push to private repository
cd mcp-servers/project-management-automation
git add .
git commit -m "v0.1.0 - Initial release"
git tag v0.1.0
git push origin main --tags

# 2. Install on other machines
pip install git+https://github.com/username/project-management-automation.git@v0.1.0

# 3. Update .cursor/mcp.json
# Use absolute path to installed package or entry point
```

---

**Last Updated**: 2025-01-27
