# Python Virtual Environment Standardization Analysis

**Date**: 2025-11-29  
**Question**: Should we standardize on `uvx` for Python virtual environments?

---

## Current State

### Virtual Environment Solutions in Use

1. **Standard `python3 -m venv`** (Primary)
   - Used throughout the project via `scripts/include/python_utils.sh`
   - Function: `setup_venv()` creates `.venv` directories
   - Found in: 20+ shell scripts
   - Examples:
     - `web/scripts/run-*.sh` (alpaca, ib, tastytrade, tradestation services)
     - `scripts/start_rust_backend.sh`
     - `scripts/run-jupyterlab-service.sh`

2. **`uvx`** (Already in Use)
   - Used for MCP servers in `.cursor/mcp.json`:
     - `exarp` (project management automation)
     - `notebooklm` (via mcpower-proxy)
   - Installed via Ansible playbooks (`ansible/roles/devtools/tasks/main.yml`)
   - Version: Available (installed at `/home/david/.local/bin/uvx`)

3. **`uv`** (Available but Not Used)
   - Installed: `uv 0.9.11`
   - Not currently used for virtual environments
   - Could replace `venv` + `pip`

4. **Poetry** (Mentioned but Not Adopted)
   - Referenced in TODO: "Adopt Poetry for Python dependency management" (SHARED-32)
   - Status: Pending
   - No `poetry.lock` files found

### Dependency Management

- **Primary**: `requirements.txt` (compiled from `requirements.in` via `pip-compile`)
- **Secondary**: Some `pyproject.toml` files exist:
  - `python/pyproject.toml`
  - `mcp/trading_server/pyproject.toml`
  - `agents/backend/python/pyproject.toml`
- **Tool**: `pip` for installation

---

## Understanding `uvx` vs `uv`

### `uvx` (Tool Runner)
- **Purpose**: Run Python tools in isolated environments (like `npx` for Node.js)
- **Use Case**: Running tools without installing them globally
- **Example**: `uvx exarp --mcp` (runs exarp in isolated environment)
- **Not for**: Creating project virtual environments

### `uv` (Package Manager)
- **Purpose**: Fast Python package manager and virtual environment tool
- **Features**:
  - `uv venv` - Create virtual environments (replaces `python3 -m venv`)
  - `uv pip install` - Install packages (replaces `pip install`)
  - `uv sync` - Sync dependencies from `pyproject.toml` or `requirements.txt`
  - Much faster than `pip` (10-100x)
  - Better dependency resolution

---

## Recommendation: Standardize on `uv` (Not Just `uvx`)

### ✅ **Yes, Standardize on `uv` Ecosystem**

**Rationale:**
1. **Already Using `uvx`**: Successfully running MCP servers
2. **`uv` is Available**: Already installed (v0.9.11)
3. **Modern Tooling**: Faster, better dependency resolution
4. **Compatible**: Works with existing `requirements.txt` files
5. **Future-Proof**: Actively developed, gaining adoption

### Migration Strategy

#### Phase 1: New Scripts Use `uv`
- New scripts use `uv venv` instead of `python3 -m venv`
- New dependency installs use `uv pip install` or `uv sync`

#### Phase 2: Update Existing Scripts (Gradual)
- Update `scripts/include/python_utils.sh` to support `uv`
- Add `setup_venv_uv()` function alongside existing `setup_venv()`
- Update service scripts to use `uv` when available

#### Phase 3: Full Migration (Optional)
- Replace all `python3 -m venv` with `uv venv`
- Replace all `pip install` with `uv pip install`
- Consider migrating to `pyproject.toml` for dependency management

---

## Implementation Plan

### Option A: Gradual Migration (Recommended)

**Step 1: Update `python_utils.sh`**
```bash
# Add uv support alongside existing venv
setup_venv_uv() {
  # Use uv if available, fallback to python3 -m venv
  if command -v uv &> /dev/null; then
    uv venv "${VENV_DIR}"
  else
    "${PYTHON_CMD}" -m venv "${VENV_DIR}"
  fi
}
```

**Step 2: Update Service Scripts**
- Modify `web/scripts/run-*.sh` to prefer `uv` when available
- Keep fallback to standard `venv` for compatibility

**Step 3: Documentation**
- Update `docs/PYTHON_ENVIRONMENT_SETUP.md`
- Document `uv` as preferred method
- Keep `venv` as fallback

### Option B: Full Migration (Aggressive)

**Step 1: Replace All `venv` Usage**
- Update all scripts to use `uv venv`
- Remove `python3 -m venv` references

**Step 2: Replace All `pip` Usage**
- Update all scripts to use `uv pip install`
- Or migrate to `uv sync` with `pyproject.toml`

**Step 3: Update CI/CD**
- Use `uv` in GitHub Actions
- Faster CI builds

---

## Benefits of Standardizing on `uv`

### Performance
- **10-100x faster** than `pip` for package installation
- Faster virtual environment creation
- Faster dependency resolution

### Compatibility
- Works with existing `requirements.txt` files
- Can use `pyproject.toml` if desired
- Drop-in replacement for `pip` commands

### Modern Tooling
- Actively developed (Astral, creators of Ruff)
- Better error messages
- Better dependency conflict resolution
- Built-in virtual environment management

### Consistency
- Same toolchain for:
  - Virtual environments (`uv venv`)
  - Package installation (`uv pip install`)
  - Tool execution (`uvx`)

---

## Potential Issues & Mitigations

### Issue 1: `uv` Not Available on All Systems
**Mitigation**: Keep fallback to standard `venv` in scripts

### Issue 2: Learning Curve
**Mitigation**: Minimal - `uv` commands are similar to `pip`

### Issue 3: CI/CD Compatibility
**Mitigation**: `uv` is available via GitHub Actions, can install via pip

### Issue 4: Team Adoption
**Mitigation**: Gradual migration allows team to adapt

---

## Recommended Approach

### ✅ **Hybrid Approach: Prefer `uv`, Fallback to `venv`**

1. **Update `python_utils.sh`** to detect and use `uv` when available
2. **Keep `venv` as fallback** for systems without `uv`
3. **Document both approaches** in setup guides
4. **Update CI/CD** to use `uv` (faster builds)
5. **Gradually migrate** scripts to prefer `uv`

### Code Example

```bash
# Updated setup_venv() function
setup_venv() {
  local python_dir="${1:-}"
  local venv_dir="${2:-${python_dir}/.venv}"

  VENV_DIR="${venv_dir}"
  ACTIVATE_PATH="${VENV_DIR}/bin/activate"

  # Prefer uv if available, fallback to python3 -m venv
  if [ ! -f "${ACTIVATE_PATH}" ]; then
    if command -v uv &> /dev/null; then
      echo "Creating virtual environment with uv at ${VENV_DIR}..." >&2
      uv venv "${VENV_DIR}" || return 1
    else
      echo "Creating virtual environment with venv at ${VENV_DIR}..." >&2
      "${PYTHON_CMD}" -m venv "${VENV_DIR}" || return 1
    fi
  fi

  # Rest of function remains the same...
}
```

---

## Next Steps

1. ✅ **Analysis Complete** (this document)
2. ⏳ **Update `python_utils.sh`** to support `uv`
3. ⏳ **Test migration** on one service script
4. ⏳ **Update documentation**
5. ⏳ **Gradual rollout** to all scripts

---

## Conclusion

**Recommendation**: ✅ **Yes, standardize on `uv` ecosystem**

- `uvx` is already working well for MCP servers
- `uv` provides better performance and modern tooling
- Can be adopted gradually with fallback to `venv`
- Compatible with existing `requirements.txt` workflow
- Future-proof choice

**Not Just `uvx`**: Use the full `uv` toolchain:
- `uv venv` for virtual environments
- `uv pip install` for package installation
- `uvx` for tool execution (already in use)

---

**Last Updated**: 2025-11-29  
**Status**: Analysis Complete - Ready for Implementation
