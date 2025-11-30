# PyPI Publishing Quick Start

**Quick reference for publishing Exarp to PyPI**

---

## Setup (One-Time)

### 1. PyPI Pending Publisher

1. Go to [PyPI Account Settings](https://pypi.org/manage/account/)
2. Navigate to **Pending publishers** (or **API tokens** → **Add a pending publisher**)
3. Add publisher:
   - **PyPI project**: `exarp-automation-mcp`
   - **Owner**: `davidl71`
   - **Repository**: `project-management-automation`
   - **Workflow**: `.github/workflows/publish-pypi.yml`
   - **Environment**: `pypi` (optional)

### 2. GitHub Environment

1. Go to repository **Settings** → **Environments**
2. Create environment: `pypi`
3. (Optional) Add protection rules

---

## Publishing

### Method 1: Release-Based (Recommended)

1. Update version in `pyproject.toml`:

   ```toml
   version = "0.2.0"
   ```

2. Commit and push:

   ```bash
   git add pyproject.toml
   git commit -m "Bump version to 0.2.0"
   git push
   ```

3. Create GitHub release:
   - Go to **Releases** → **Create a new release**
   - **Tag**: `v0.2.0`
   - **Title**: `v0.2.0`
   - **Publish release**

4. Workflow automatically publishes to PyPI

### Method 2: Manual Workflow

1. Go to **Actions** → **Publish to PyPI**
2. Click **Run workflow**
3. Enter version (e.g., `0.2.0.dev1` for testing)
4. Click **Run workflow**

---

## Verify

```bash

# Check PyPI

open https://pypi.org/project/exarp-automation-mcp/

# Test installation

pip install exarp-automation-mcp
python3 -c "from exarp_project_management.server import main; print('✅ OK')"
```

---

## Troubleshooting

- **"Publisher not found"**: Verify pending publisher in PyPI matches repository
- **"Permission denied"**: Check `id-token: write` permission in workflow
- **"Version exists"**: Increment version in `pyproject.toml`

---

**Full Guide**: See [PYPI_PUBLISHING_SETUP.md](PYPI_PUBLISHING_SETUP.md)
