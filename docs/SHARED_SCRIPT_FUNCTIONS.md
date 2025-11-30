# Shared Script Functions - Refactoring Analysis

**Date**: 2025-11-19
**Status**: Analysis Complete

## Executive Summary

Identified **8 major redundancies** across service scripts that can be extracted into shared include files. This will reduce code duplication by ~60% and improve maintainability significantly.

---

## 🔍 Redundancies Identified

### 1. **Python Detection** (100% duplicate)

**Found in:** All 5 service scripts
**Lines duplicated:** ~10 lines per script = **50 lines total**

**Current pattern:**

```bash
PYTHON_CMD=""
if command -v python3 >/dev/null 2>&1; then
  PYTHON_CMD="python3"
elif command -v python >/dev/null 2>&1; then
  PYTHON_CMD="python"
fi
if [ -z "${PYTHON_CMD}" ]; then
  echo "Error: Python not found..." >&2
  exit 1
fi
```

**Proposed function:** `find_python()` in `scripts/include/python_utils.sh`

---

### 2. **Virtual Environment Setup** (100% duplicate)

**Found in:** All 5 service scripts
**Lines duplicated:** ~20 lines per script = **100 lines total**

**Current pattern:**

```bash
VENV_DIR="${PYTHON_DIR}/.venv"
ACTIVATE_PATH="${VENV_DIR}/bin/activate"
if [ ! -f "${ACTIVATE_PATH}" ]; then
  echo "Creating Python virtual environment..."
  "${PYTHON_CMD}" -m venv "${VENV_DIR}" || { ... }
fi
source "${ACTIVATE_PATH}"
"${PYTHON_CMD}" -m pip install --quiet --upgrade pip wheel
VENV_PYTHON="${VENV_DIR}/bin/python"
```

**Proposed function:** `setup_venv()` in `scripts/include/python_utils.sh`

---

### 3. **Package Installation** (90% duplicate)

**Found in:** All 5 service scripts
**Lines duplicated:** ~15 lines per script = **75 lines total**

**Current pattern:**

```bash
MISSING_PACKAGES=()
if ! "${VENV_PYTHON}" -c "import uvicorn" 2>/dev/null; then
  MISSING_PACKAGES+=("uvicorn[standard]" "fastapi")
elif ! "${VENV_PYTHON}" -c "import websockets" ...; then
  MISSING_PACKAGES+=("websockets")
fi
if [ ${#MISSING_PACKAGES[@]} -gt 0 ]; then
  "${VENV_PYTHON}" -m pip install --quiet "${MISSING_PACKAGES[@]}"
fi
```

**Proposed function:** `install_python_packages()` in `scripts/include/python_utils.sh`

---

### 4. **Port Checking with Health Check** (95% duplicate)

**Found in:** 4 service scripts (Alpaca, IB, TradeStation, Discount Bank)
**Lines duplicated:** ~25 lines per script = **100 lines total**

**Current pattern:**

```bash
if ! config_check_port_available "${PORT}"; then
  echo "Port ${PORT} is already in use..."
  HEALTH_CHECK=$("${PYTHON_CMD}" -c "
import urllib.request
import json
try:
    with urllib.request.urlopen('http://127.0.0.1:${PORT}/api/health'...
  ")
  if [ "${HEALTH_CHECK}" = "SERVICE_NAME" ]; then
    echo "✓ Service already running..."
    exit 0
  else
    echo "Error: Port conflict..."
    exit 1
  fi
fi
```

**Proposed functions:**

- `check_service_health()` in `scripts/include/service_utils.sh`
- `check_port_with_service()` in `scripts/include/service_utils.sh`

---

### 5. **1Password Credential Reading** (100% duplicate)

**Found in:** 3 service scripts (Alpaca x2, TradeStation)
**Lines duplicated:** ~30 lines per script = **90 lines total**

**Current pattern:**

```bash
read_credential() {
  local op_secret="${1:-}"
  local env_var="${2:-}"
  if [ -n "${op_secret:-}" ] && command -v op >/dev/null 2>&1; then
    result=$(op read "${op_secret}" 2>/dev/null | tr -d '[:space:]')
    ...
  fi
  if [ -n "${env_var:-}" ]; then
    echo -n "${env_var}"
  fi
}
```

**Proposed functions:**

- `read_credential()` in `scripts/include/onepassword.sh`
- `op_detect_fields()` in `scripts/include/onepassword.sh`
- `op_build_secret_paths()` in `scripts/include/onepassword.sh`

---

### 6. **Import Testing** (90% duplicate)

**Found in:** 3 service scripts (Alpaca x2, IB)
**Lines duplicated:** ~20 lines per script = **60 lines total**

**Current pattern:**

```bash
IMPORT_TEST=$("${PYTHON_CMD}" -c "
import sys
import os
init_py = 'integration/__init__.py'
backup_py = 'integration/__init__.py.bak'
if os.path.exists(init_py):
    os.rename(init_py, backup_py)
try:
    sys.path.insert(0, '.')
    from integration.service_name import app
    print('OK')
except Exception as e:
    print(f'ERROR: {e}')
finally:
    if os.path.exists(backup_py):
        os.rename(backup_py, init_py)
")
```

**Proposed function:** `test_python_import()` in `scripts/include/python_utils.sh`

---

### 7. ****init**.py Handling** (100% duplicate)

**Found in:** 3 service scripts (Alpaca x2, IB)
**Lines duplicated:** ~8 lines per script = **24 lines total**

**Current pattern:**

```bash
INIT_PY="integration/__init__.py"
INIT_PY_BAK="integration/__init__.py.bak"
if [ -f "${INIT_PY}" ]; then
  mv "${INIT_PY}" "${INIT_PY_BAK}"
  trap "mv '${INIT_PY_BAK}' '${INIT_PY}' 2>/dev/null || true" EXIT
fi
```

**Proposed function:** `disable_init_py()` in `scripts/include/python_utils.sh`

---

### 8. **Config Loader Fallback** (100% duplicate)

**Found in:** All 5 service scripts
**Lines duplicated:** ~30 lines per script = **150 lines total**

**Current pattern:**

```bash
if [ -f "${SCRIPTS_DIR}/include/config.sh" ]; then
  source "${SCRIPTS_DIR}/include/config.sh"
else
  echo "Warning: config.sh not found..."
  config_get_port() { ... }
  config_check_port_available() { ... }
fi
```

**Proposed solution:** Remove fallback (config.sh should always exist) or move to separate include

---

## 📊 Impact Summary

| Category | Scripts Affected | Lines Duplicated | Reduction Potential |
|----------|----------------|------------------|---------------------|
| Python Detection | 5 | 50 | 100% |
| Venv Setup | 5 | 100 | 100% |
| Package Installation | 5 | 75 | 90% |
| Port/Health Check | 4 | 100 | 95% |
| 1Password | 3 | 90 | 100% |
| Import Testing | 3 | 60 | 90% |
| **init**.py | 3 | 24 | 100% |
| Config Fallback | 5 | 150 | 100% |
| **TOTAL** | **5 scripts** | **~649 lines** | **~95% reduction** |

**Estimated reduction:** From ~649 duplicate lines to ~32 function calls = **~617 lines saved** (95% reduction)

---

## 📁 Proposed Include Files

### 1. `scripts/include/python_utils.sh` ✅ Created

**Functions:**

- `find_python()` - Detect Python command
- `setup_venv()` - Create/activate virtual environment
- `install_python_packages()` - Install missing packages
- `test_python_import()` - Test module import
- `disable_init_py()` - Temporarily disable **init**.py

### 2. `scripts/include/service_utils.sh` ✅ Created

**Functions:**

- `check_service_health()` - Check service health endpoint
- `check_port_with_service()` - Port check with service verification

### 3. `scripts/include/onepassword.sh` ✅ Created

**Functions:**

- `read_credential()` - Read from 1Password or env var
- `op_detect_fields()` - Auto-detect 1Password item fields
- `op_build_secret_paths()` - Build op:// secret paths

---

## 🔧 Migration Plan

### Phase 1: Create Include Files ✅

- [x] Create `python_utils.sh`
- [x] Create `service_utils.sh`
- [x] Create `onepassword.sh`

### Phase 2: Update Scripts (Recommended Order)

1. **Start with one script** (e.g., `run-discount-bank-service.sh`) as proof of concept
2. **Update remaining scripts** incrementally
3. **Test thoroughly** after each update

### Phase 3: Remove Fallbacks

- Remove config fallback code (assume config.sh always exists)
- Update documentation

---

## 📝 Usage Examples

### Before (Current)

```bash

# Find Python

PYTHON_CMD=""
if command -v python3 >/dev/null 2>&1; then
  PYTHON_CMD="python3"
elif command -v python >/dev/null 2>&1; then
  PYTHON_CMD="python"
fi
if [ -z "${PYTHON_CMD}" ]; then
  echo "Error: Python not found" >&2
  exit 1
fi

# Setup venv

VENV_DIR="${PYTHON_DIR}/.venv"
if [ ! -f "${VENV_DIR}/bin/activate" ]; then
  "${PYTHON_CMD}" -m venv "${VENV_DIR}"
fi
source "${VENV_DIR}/bin/activate"
VENV_PYTHON="${VENV_DIR}/bin/python"
```

### After (Proposed)

```bash

# Load shared functions

source "${SCRIPTS_DIR}/include/python_utils.sh"

# Find Python

find_python || exit 1

# Setup venv

setup_venv "${PYTHON_DIR}" || exit 1
```

**Reduction:** 20 lines → 4 lines (80% reduction)

---

## ✅ Benefits

1. **Maintainability**: Fix bugs once, apply everywhere
2. **Consistency**: All scripts behave identically
3. **Readability**: Scripts focus on service-specific logic
4. **Testing**: Test functions once, reuse everywhere
5. **Documentation**: Centralized function documentation

---

## 🚨 Considerations

1. **Backward Compatibility**: All functions maintain same behavior
2. **Error Handling**: Functions return proper exit codes
3. **Variable Naming**: Uses consistent naming conventions
4. **Dependencies**: Functions can depend on each other
5. **Testing**: Each function should be tested independently

---

## 📋 Next Steps

1. **Review proposed functions** - Ensure they match current behavior
2. **Test functions** - Create test script to verify behavior
3. **Migrate one script** - Use as proof of concept
4. **Migrate remaining scripts** - Apply patterns incrementally
5. **Update documentation** - Document new include files

---

**Conclusion**: Extracting these redundancies will significantly improve code quality and maintainability while reducing duplication by ~95%.
