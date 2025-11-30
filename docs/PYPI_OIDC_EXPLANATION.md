# PyPI Trusted Publishing with OIDC

**Date**: 2025-01-27
**Topic**: OpenID Connect (OIDC) for PyPI Publishing

---

## What is OIDC?

**OpenID Connect (OIDC)** is an authentication protocol built on OAuth 2.0 that enables secure, token-based authentication without storing long-lived credentials.

### Key Concepts

1. **Identity Provider (IdP)**: GitHub (in our case)
2. **Relying Party (RP)**: PyPI (the service we're authenticating to)
3. **OIDC Token**: Short-lived, cryptographically signed token proving identity

---

## How OIDC Works for PyPI Publishing

### Traditional Method (API Tokens) ❌

**Old Way**:

1. Generate API token in PyPI
2. Store token as GitHub secret
3. Use token in workflow
4. **Risk**: Token can be leaked, stolen, or misused

**Problems**:

- Long-lived credentials
- Must be stored as secrets
- Can be compromised
- Hard to rotate
- No automatic expiration

### OIDC Method (Trusted Publishing) ✅

**New Way**:

1. Configure "Pending Publisher" in PyPI
2. GitHub Actions requests OIDC token from GitHub
3. GitHub issues short-lived token (valid for ~10 minutes)
4. Token is used to authenticate to PyPI
5. PyPI verifies token signature and claims
6. **No secrets needed!**

**Benefits**:

- ✅ No long-lived credentials
- ✅ No secrets to manage
- ✅ Automatic token rotation
- ✅ Short-lived tokens (expire quickly)
- ✅ Cryptographically verified
- ✅ Auditable

---

## OIDC Flow for PyPI Publishing

```
┌─────────────────┐
│  GitHub Actions │
│   (Workflow)    │
└────────┬────────┘
         │
         │ 1. Request OIDC token
         ▼
┌─────────────────┐
│     GitHub       │
│  (IdP - Issues   │
│   OIDC Token)    │
└────────┬────────┘
         │
         │ 2. OIDC Token
         │    (signed, short-lived)
         ▼
┌─────────────────┐
│  GitHub Actions │
│   (Workflow)    │
└────────┬────────┘
         │
         │ 3. Use token to publish
         ▼
┌─────────────────┐
│      PyPI       │
│  (RP - Verifies │
│   Token Claims) │
└─────────────────┘
```

### Step-by-Step Process

1. **Workflow Triggers**: Release published or manual dispatch
2. **GitHub Issues Token**: GitHub automatically issues OIDC token
3. **Token Contains Claims**:
   - Repository: `davidl71/project-management-automation`
   - Workflow: `.github/workflows/publish-pypi.yml`
   - Environment: `pypi` (if specified)
   - Actor: GitHub Actions
4. **PyPI Verifies**: Checks token signature and claims match pending publisher
5. **Publish**: If verified, package is published

---

## Configuration in Our Workflow

### Required Permissions

```yaml
permissions:
  id-token: write  # Required for OIDC token
  contents: read   # Required to read repository
```

**`id-token: write`**:

- Allows workflow to request OIDC token from GitHub
- **Critical**: Without this, OIDC won't work

### Environment Configuration

```yaml
environment:
  name: pypi
  url: https://pypi.org/p/exarp-automation-mcp
```

**Environment**:

- Optional but recommended
- Allows protection rules (approvals, branch restrictions)
- Must match environment name in PyPI pending publisher (if specified)

### Publishing Action

```yaml

- name: Publish to PyPI
  uses: pypa/gh-action-pypi-publish@release/v1
  with:
    packages-dir: dist/
    print-hash: true
```

**`pypa/gh-action-pypi-publish@release/v1`**:

- Automatically uses OIDC if configured
- No `username` or `password` needed
- No `__token__` needed
- Detects OIDC from environment configuration

---

## PyPI Pending Publisher Configuration

### What PyPI Stores

When you create a pending publisher, PyPI stores:

- **Repository**: `davidl71/project-management-automation`
- **Workflow**: `.github/workflows/publish-pypi.yml`
- **Environment**: `pypi` (if specified)
- **Project**: `exarp-automation-mcp`

### How PyPI Verifies

1. **Receives OIDC Token**: From GitHub Actions workflow
2. **Verifies Signature**: Checks token is signed by GitHub
3. **Checks Claims**: Verifies repository, workflow, environment match
4. **Authorizes**: If all match, allows publish

### Pending → Active

- **Pending**: Configured but not yet verified
- **Active**: Successfully published at least once
- **Status Change**: Automatic on first successful publish

---

## Security Benefits

### 1. No Secrets Management

**Before (API Token)**:

```yaml

- name: Publish
  env:
    TWINE_USERNAME: __token__
    TWINE_PASSWORD: ${{ secrets.PYPI_API_TOKEN }}  # ❌ Secret
```

**After (OIDC)**:

```yaml

- name: Publish
  uses: pypa/gh-action-pypi-publish@release/v1
  # ✅ No secrets needed!
```

### 2. Automatic Token Rotation

- OIDC tokens are generated fresh for each workflow run
- No manual rotation needed
- Old tokens automatically expire

### 3. Scope Limitation

- Token only valid for specific repository
- Token only valid for specific workflow
- Token only valid for specific environment (if specified)
- Cannot be reused elsewhere

### 4. Audit Trail

- PyPI logs show which repository/workflow published
- GitHub Actions logs show token usage
- Full traceability

---

## Troubleshooting OIDC

### Issue: "Permission denied" or "Authentication failed"

**Check**:

1. ✅ `id-token: write` permission in workflow
2. ✅ Environment name matches PyPI pending publisher
3. ✅ Repository name matches PyPI pending publisher
4. ✅ Workflow filename matches PyPI pending publisher

### Issue: "Publisher not found"

**Check**:

1. ✅ Pending publisher created in PyPI
2. ✅ Repository owner matches (case-sensitive)
3. ✅ Workflow path is exactly `.github/workflows/publish-pypi.yml`

### Issue: "Token validation failed"

**Check**:

1. ✅ GitHub Actions is enabled for repository
2. ✅ Workflow file is in correct location
3. ✅ Environment exists in GitHub (if specified)

---

## OIDC Token Details

### Token Lifetime

- **Valid for**: ~10 minutes
- **Auto-expires**: Cannot be reused
- **Scope**: Limited to specific repository/workflow

### Token Claims

Example OIDC token claims:

```json
{
  "iss": "https://token.actions.githubusercontent.com",
  "sub": "repo:davidl71/project-management-automation:ref:refs/heads/main",
  "aud": "pypi",
  "repository": "davidl71/project-management-automation",
  "workflow": ".github/workflows/publish-pypi.yml",
  "environment": "pypi"
}
```

### Token Verification

PyPI verifies:

1. **Signature**: Token signed by GitHub
2. **Issuer**: `https://token.actions.githubusercontent.com`
3. **Audience**: `pypi`
4. **Repository**: Matches pending publisher
5. **Workflow**: Matches pending publisher
6. **Environment**: Matches pending publisher (if specified)

---

## Comparison: API Token vs OIDC

| Aspect | API Token | OIDC |
|--------|-----------|------|
| **Storage** | GitHub Secret | No storage needed |
| **Lifetime** | Until revoked | ~10 minutes |
| **Rotation** | Manual | Automatic |
| **Security** | Can be leaked | Cryptographically signed |
| **Scope** | Full account access | Repository-specific |
| **Audit** | Limited | Full traceability |
| **Setup** | Generate token | Configure pending publisher |

---

## Best Practices

### 1. Use Environment Protection

```yaml
environment:
  name: pypi
```

**Benefits**:

- Can require approvals
- Can restrict to specific branches
- Can add wait timers

### 2. Restrict to Main Branch

In GitHub environment settings:

- **Deployment branches**: `main` only
- Prevents accidental publishes from feature branches

### 3. Use Release-Based Publishing

```yaml
on:
  release:
    types: [published]
```

**Benefits**:

- Only publishes on official releases
- Prevents accidental publishes
- Clear version tracking

### 4. Verify Before Publishing

```yaml

- name: Check package
  run: twine check dist/*
```

**Benefits**:

- Catches errors before publishing
- Validates package structure
- Prevents broken releases

---

## References

- [PyPI Trusted Publishing Guide](https://docs.pypi.org/trusted-publishers/)
- [GitHub Actions OIDC](https://docs.github.com/en/actions/deployment/security-hardening-your-deployments/about-security-hardening-with-openid-connect)
- [OIDC Specification](https://openid.net/specs/openid-connect-core-1_0.html)

---

## Summary

**OIDC for PyPI Publishing**:

- ✅ **Secure**: No long-lived credentials
- ✅ **Simple**: No secrets to manage
- ✅ **Automatic**: Token rotation handled automatically
- ✅ **Verifiable**: Cryptographically signed tokens
- ✅ **Recommended**: PyPI's preferred method

**Our Setup**:

- ✅ Workflow configured with `id-token: write`
- ✅ Environment configured (`pypi`)
- ✅ Using `pypa/gh-action-pypi-publish@release/v1`
- ✅ Ready for pending publisher setup

---

**Status**: ✅ OIDC configuration complete - Ready for PyPI pending publisher setup
