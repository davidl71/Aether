# Python Virtual Environment Migration Plan: Standardizing on `uv`

**Date**: 2025-11-29
**Status**: In Progress
**Goal**: Migrate from `python3 -m venv` + `pip` to `uv venv` + `uv pip` for better performance and modern tooling

---

## Executive Summary

This migration plan outlines the gradual adoption of `uv` (the fast Python package manager) across the project while maintaining backward compatibility with standard `venv` and `pip`.

**Benefits**:

- **10-100x faster** package installation
- **Faster virtual environment creation**
- **Better dependency resolution**
- **Consistent toolchain** (uv venv, uv pip, uvx)

**Approach**: Gradual migration with automatic fallback to standard tools

---

## Current State

### Virtual Environment Usage

- **13+ scripts** use `setup_venv()` from `scripts/include/python_utils.sh`
- **Standard `python3 -m venv`** is the current method
- **`uvx` already in use** for MCP servers (exarp, notebooklm)
- **`uv` installed** but not used for venv management

### Scripts Using `setup_venv()`

1. `web/scripts/run-alpaca-service.sh`
2. `web/scripts/run-ib-service.sh`
3. `web/scripts/run-tastytrade-service.sh`
4. `web/scripts/run-tradestation-service.sh`
5. `web/scripts/run-discount-bank-service.sh`
6. `web/scripts/run-risk-free-rate-service.sh`
7. `scripts/start_rust_backend.sh`
9. And more...

---

## Migration Phases

### ✅ Phase 1: Update Core Utilities (COMPLETE)

**Status**: ✅ Complete
**Date**: 2025-11-29

**Changes Made**:

1. Updated `scripts/include/python_utils.sh`:
   - `setup_venv()` now prefers `uv venv` when available
   - Falls back to `python3 -m venv` for compatibility
   - Sets `USE_UV` variable to track tool usage
2. Updated `install_python_packages()`:
   - Prefers `uv pip install` when `uv` is available
   - Falls back to `pip install` for compatibility

**Testing**:

- ✅ Scripts continue to work with standard `venv`
- ✅ Scripts automatically use `uv` when available
- ✅ No breaking changes for existing workflows

---

### ⏳ Phase 2: Update Service Scripts (OPTIONAL)

**Status**: Pending
**Estimated Time**: 1-2 hours

**Goal**: Update service scripts to explicitly prefer `uv` and provide better feedback

**Changes**:

1. Add `uv` detection messages to service scripts
2. Document `uv` usage in script comments
3. Add `uv` installation instructions to error messages

**Scripts to Update**:

- `web/scripts/run-*.sh` (all service scripts)
- `scripts/start_rust_backend.sh`

**Example Update**:

```bash

# Before

setup_venv "${PYTHON_DIR}" || exit 1

# After (with better feedback)

if command -v uv >/dev/null 2>&1; then
  echo "ℹ️  Using uv for faster virtual environment management" >&2
fi
setup_venv "${PYTHON_DIR}" || exit 1
```

---

### ⏳ Phase 3: Update CI/CD (RECOMMENDED)

**Status**: Pending
**Estimated Time**: 30 minutes

**Goal**: Use `uv` in CI/CD pipelines for faster builds

**Changes**:

1. Install `uv` in GitHub Actions workflows
2. Use `uv venv` and `uv pip install` in CI scripts
3. Document `uv` usage in CI/CD documentation

**Example GitHub Actions Update**:

```yaml

# Before
- name: Set up Python
  uses: actions/setup-python@v4
  with:
    python-version: '3.12'

- name: Create virtual environment
  run: |
    python -m venv .venv
    source .venv/bin/activate
    pip install -r requirements.txt

# After
- name: Install uv
  run: pip install uv

- name: Set up Python
  uses: actions/setup-python@v4
  with:
    python-version: '3.12'

- name: Create virtual environment and install dependencies
  run: |
    uv venv .venv
    source .venv/bin/activate
    uv pip install -r requirements.txt
```

**Benefits**:

- Faster CI builds (10-100x faster package installation)
- More reliable dependency resolution
- Consistent with local development

---

### ⏳ Phase 4: Documentation Updates (COMPLETE)

**Status**: ✅ Complete
**Date**: 2025-11-29

**Changes Made**:

1. Created `docs/PYTHON_VENV_STANDARDIZATION_ANALYSIS.md`
2. Created `docs/PYTHON_UV_MIGRATION_PLAN.md` (this document)
3. Updated `docs/PYTHON_ENVIRONMENT_SETUP.md` with `uv` instructions

---

### ⏳ Phase 5: Team Communication (RECOMMENDED)

**Status**: Pending

**Goal**: Inform team about `uv` adoption and benefits

**Actions**:

1. Announce `uv` adoption in team communication
2. Share migration benefits and performance improvements
3. Provide installation instructions
4. Document troubleshooting guide

**Installation Instructions**:

```bash

# Install uv (recommended)

pip install uv

# Or via Homebrew (macOS)

brew install uv

# Verify installation

uv --version
```

---

## Rollback Plan

If issues arise, rollback is simple:

1. **No code changes needed** - scripts automatically fall back to `venv`
2. **Remove `uv`** if causing issues: `pip uninstall uv`
3. **Scripts continue working** with standard `python3 -m venv`

**No Breaking Changes**: The migration maintains full backward compatibility.

---

## Testing Checklist

### Pre-Migration Testing

- [x] Verify `uv` is installed: `uv --version`
- [x] Test `uv venv` creates valid virtual environments
- [x] Test `uv pip install` works correctly
- [x] Verify scripts work with `uv` when available
- [x] Verify scripts work without `uv` (fallback)

### Post-Migration Testing

- [ ] Test all service scripts with `uv` available
- [ ] Test all service scripts without `uv` (fallback)
- [ ] Verify CI/CD pipelines work with `uv`
- [ ] Test on multiple platforms (macOS, Linux)
- [ ] Verify performance improvements

---

## Performance Benchmarks

### Expected Improvements

**Virtual Environment Creation**:

- Standard `venv`: ~2-5 seconds
- `uv venv`: ~0.5-1 second (2-5x faster)

**Package Installation** (example: FastAPI + dependencies):

- Standard `pip`: ~30-60 seconds
- `uv pip`: ~3-6 seconds (10x faster)

**Dependency Resolution**:

- Standard `pip`: Can be slow for complex dependencies
- `uv pip`: Much faster resolution with better conflict detection

---

## Troubleshooting

### Issue: `uv` Not Found

**Solution**: Install `uv`:

```bash
pip install uv

# Or

brew install uv  # macOS
```

**Fallback**: Scripts automatically use `python3 -m venv` if `uv` is not available.

### Issue: `uv venv` Fails

**Solution**: Scripts automatically fall back to `python3 -m venv`.

**Debug**: Check `uv` version:

```bash
uv --version  # Should be 0.9.0+
```

### Issue: Package Installation Fails with `uv`

**Solution**: Scripts automatically fall back to `pip install`.

**Debug**: Check if package is compatible with `uv`:

```bash
uv pip install --dry-run package-name
```

---

## Success Criteria

### Phase 1 (Core Utilities) ✅

- [x] `setup_venv()` uses `uv` when available
- [x] `install_python_packages()` uses `uv pip` when available
- [x] Fallback to standard tools works correctly
- [x] No breaking changes

### Phase 2 (Service Scripts) ⏳

- [ ] All service scripts tested with `uv`
- [ ] Better user feedback about tool usage
- [ ] Documentation updated

### Phase 3 (CI/CD) ⏳

- [ ] CI/CD pipelines use `uv`
- [ ] Faster CI build times verified
- [ ] CI/CD documentation updated

### Overall Success

- [ ] `uv` is the preferred tool when available
- [ ] Standard tools remain as fallback
- [ ] Performance improvements verified
- [ ] Team adoption successful

---

## Timeline

- **Week 1**: Phase 1 (Core Utilities) ✅ Complete
- **Week 2**: Phase 2 (Service Scripts) - Optional
- **Week 3**: Phase 3 (CI/CD) - Recommended
- **Week 4**: Phase 5 (Team Communication) - Recommended

**Total Estimated Time**: 2-4 hours (excluding optional phases)

---

## References

- [uv Documentation](https://github.com/astral-sh/uv)
- [uv vs pip Performance](https://github.com/astral-sh/uv#benchmarks)
- [Project Analysis](./PYTHON_VENV_STANDARDIZATION_ANALYSIS.md)
- [Python Environment Setup](./PYTHON_ENVIRONMENT_SETUP.md)

---

**Last Updated**: 2025-11-29
**Status**: Phase 1 Complete, Phases 2-5 Pending
