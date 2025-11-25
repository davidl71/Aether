# PyPI Publishing Setup Guide

**Date**: 2025-01-27
**Package**: `exarp-automation-mcp`
**Status**: Ready for PyPI Publishing

---

## Overview

This guide explains how to set up PyPI trusted publishing (Pending Publisher) for Exarp and publish the package to PyPI.

---

## Prerequisites

1. ✅ **PyPI Account**: You have a PyPI account
2. ✅ **Package Name**: `exarp-automation-mcp` (already configured in `pyproject.toml`)
3. ✅ **GitHub Repository**: Private repository at `davidl71/project-management-automation`
4. ✅ **GitHub Actions**: Enabled for the repository

---

## Step 1: Set Up PyPI Trusted Publishing (Pending Publisher)

### 1.1 Log in to PyPI

1. Go to [PyPI](https://pypi.org)
2. Log in to your account
3. Navigate to **Account Settings** → **API tokens** or **Manage account** → **Pending publishers**

### 1.2 Add Pending Publisher

1. Go to **Account Settings** → **Pending publishers** (or **API tokens** → **Add a pending publisher**)
2. Click **Add a pending publisher**
3. Fill in the form:
   - **PyPI project name**: `exarp-automation-mcp`
   - **Owner**: `davidl71` (your GitHub username)
   - **Repository name**: `project-management-automation`
   - **Workflow filename**: `.github/workflows/publish-pypi.yml`
   - **Environment name**: `pypi` (optional, but recommended)

4. Click **Add**
5. **Important**: Copy the **Publisher ID** (you'll need this for verification)

### 1.3 Verify Pending Publisher

PyPI will show the pending publisher status. It will be **"Pending"** until you:
1. ✅ Set up the GitHub Actions workflow (Already created)
2. ✅ Create the GitHub environment (see Step 2)
3. ✅ Run the workflow successfully (see Step 4)

**What "Pending" means**:
- The publisher is configured but not yet verified
- Verification happens automatically on first successful publish
- After first successful publish, status changes to **"Active"**

---

## Step 2: Set Up GitHub Environment

### 2.1 Create GitHub Environment

1. Go to your GitHub repository: `https://github.com/davidl71/project-management-automation`
2. Navigate to **Settings** → **Environments**
3. Click **New environment**
4. Name it: `pypi`
5. Click **Configure environment**

### 2.2 Configure Environment (Optional but Recommended)

**Protection rules** (optional):
- **Required reviewers**: Add yourself (optional)
- **Wait timer**: 0 minutes (or set if you want manual approval)
- **Deployment branches**: `main` only (or `main` and `release/*`)

**Environment secrets** (not needed for trusted publishing):
- Trusted publishing uses OIDC, so no secrets needed!

Click **Save protection rules**

---

## Step 3: Verify Workflow Configuration

### 3.1 Check Workflow File

The workflow file is already created at:
```
.github/workflows/publish-pypi.yml
```

**Key Features**:
- ✅ Uses `pypa/gh-action-pypi-publish@release/v1` (trusted publishing)
- ✅ Requires `id-token: write` permission (for OIDC)
- ✅ Uses `pypi` environment
- ✅ Triggers on release publication or manual workflow dispatch
- ✅ Builds package using `python -m build`
- ✅ Checks package with `twine check`
- ✅ Publishes to PyPI automatically

### 3.2 Verify pyproject.toml

Ensure `pyproject.toml` has correct package name:
```toml
[project]
name = "exarp-automation-mcp"
version = "0.2.0"
```

---

## Step 4: Test Publishing

### 4.1 Manual Test (Workflow Dispatch)

1. Go to **Actions** tab in GitHub
2. Select **Publish to PyPI** workflow
3. Click **Run workflow**
4. Fill in:
   - **Version**: `0.2.0` (or test version like `0.2.0.dev1`)
   - **Skip tests**: `false` (or `true` for quick test)
5. Click **Run workflow**

### 4.2 Release-Based Publishing

1. Create a new release:
   - Go to **Releases** → **Create a new release**
   - **Tag**: `v0.2.0` (or `v0.2.0-dev1` for test)
   - **Title**: `v0.2.0 - Initial PyPI Release`
   - **Description**: Release notes
   - Click **Publish release**

2. The workflow will automatically trigger and publish to PyPI

---

## Step 5: Verify Publication

### 5.1 Check PyPI

1. Go to: `https://pypi.org/project/exarp-automation-mcp/`
2. Verify the package is published
3. Check version matches your release

### 5.2 Test Installation

```bash
# Install from PyPI
pip install exarp-automation-mcp

# Verify installation
python3 -c "from exarp_project_management.server import main; print('✅ Installed successfully')"
```

---

## Troubleshooting

### Issue: "Publisher not found" or "Pending publisher not verified"

**Solution**:
1. Verify the **Publisher ID** in PyPI matches your repository
2. Ensure workflow file path is exactly: `.github/workflows/publish-pypi.yml`
3. Ensure environment name is exactly: `pypi`
4. Check that GitHub Actions has permission to access the repository

### Issue: "Permission denied" or "Authentication failed"

**Solution**:
1. Verify `id-token: write` permission is set in workflow
2. Ensure environment `pypi` exists in GitHub
3. Check that trusted publishing is enabled in PyPI

### Issue: "Package name already exists"

**Solution**:
1. Check if package name `exarp-automation-mcp` is available on PyPI
2. If taken, update `pyproject.toml` with a different name
3. Update pending publisher with new package name

### Issue: "Version already exists"

**Solution**:
1. Increment version in `pyproject.toml`
2. Create new release with new version
3. Re-run workflow

---

## Workflow Triggers

### Automatic (Release)
- **Trigger**: When a release is published
- **Action**: Automatically builds and publishes to PyPI

### Manual (Workflow Dispatch)
- **Trigger**: Manual workflow run
- **Action**: Allows testing before release
- **Inputs**:
  - `version`: Version to publish
  - `skip_tests`: Skip tests (for quick testing)

---

## Version Management

### Version Format

Follow [Semantic Versioning](https://semver.org/):
- **Major**: Breaking changes (1.0.0)
- **Minor**: New features (0.3.0)
- **Patch**: Bug fixes (0.2.1)

### Pre-release Versions

For testing:
- **Alpha**: `0.2.0a1`
- **Beta**: `0.2.0b1`
- **Release Candidate**: `0.2.0rc1`
- **Development**: `0.2.0.dev1`

---

## Security Best Practices

1. ✅ **Trusted Publishing**: Uses OIDC (no API tokens needed)
2. ✅ **Environment Protection**: Optional but recommended
3. ✅ **Branch Protection**: Only publish from `main` or `release/*`
4. ✅ **Package Verification**: `twine check` validates package before publishing

---

## Next Steps

1. ✅ Set up pending publisher in PyPI
2. ✅ Create `pypi` environment in GitHub
3. ✅ Test with workflow dispatch (version `0.2.0.dev1`)
4. ✅ Create first release (`v0.2.0`)
5. ✅ Verify publication on PyPI
6. ✅ Update installation instructions in README

---

## OIDC (OpenID Connect) Explanation

**What is OIDC?**
- OIDC is an authentication protocol that uses short-lived, cryptographically signed tokens
- **No API tokens needed** - GitHub automatically issues tokens
- **No secrets to manage** - Tokens are generated on-demand
- **More secure** - Tokens expire quickly and are repository-specific

**How it works:**
1. GitHub Actions workflow requests OIDC token from GitHub
2. GitHub issues short-lived token (~10 minutes)
3. Token contains claims (repository, workflow, environment)
4. PyPI verifies token signature and claims
5. If verified, package is published

**See**: [OIDC Explanation Guide](PYPI_OIDC_EXPLANATION.md) for detailed explanation

---

## References

- [PyPI Trusted Publishing Guide](https://docs.pypi.org/trusted-publishers/)
- [GitHub Actions OIDC](https://docs.github.com/en/actions/deployment/security-hardening-your-deployments/about-security-hardening-with-openid-connect)
- [GitHub Actions for PyPI](https://github.com/pypa/gh-action-pypi-publish)
- [Python Packaging Guide](https://packaging.python.org/)

---

**Status**: Ready for PyPI publishing setup
