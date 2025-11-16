# Extension Analysis - Redundancy & Performance

**Date**: 2025-01-27
**Total Installed**: 21 extensions

---

## ⚠️ CRITICAL: Conflicting Extensions

### 1. **`llvm-vs-code-extensions.vscode-clangd`** ❌ REMOVE

**Conflict**: Conflicts with `ms-vscode.cpptools` (C++ IntelliSense)

**Issue**: Both are C++ language servers that will:
- Fight for control of C++ files
- Cause duplicate diagnostics
- Slow down IntelliSense
- Create conflicting suggestions

**Recommendation**: **REMOVE** `vscode-clangd` - use `cpptools` instead

**Why**:
- `cpptools` is the official Microsoft C++ extension
- Better CMake integration
- More widely used and supported
- Already recommended in project

**Action**:
```bash
cursor --uninstall-extension llvm-vs-code-extensions.vscode-clangd
```

---

### 2. **`anysphere.cpptools`** ⚠️ CHECK

**Status**: Cursor's custom C++ tools

**Potential Conflict**: May conflict with `ms-vscode.cpptools` if both are active

**Recommendation**:
- If `ms-vscode.cpptools` is installed, **REMOVE** `anysphere.cpptools`
- If only `anysphere.cpptools` is installed, keep it (Cursor-specific)

**Action**: Check which one is actually providing IntelliSense

---

### 3. **`anysphere.cursorpyright`** ⚠️ CHECK

**Status**: Cursor's Python type checker

**Potential Conflict**: May conflict with Pylance if both are active

**Recommendation**:
- If Pylance is active, **REMOVE** `cursorpyright` (redundant)
- Pylance is faster and more feature-complete

**Note**: Settings already disable cursorpyright (see `.vscode/settings.json`)

**Action**: Verify Pylance is working, then remove cursorpyright

---

## 🗑️ Unnecessary Extensions (Not Used in Project)

### 4. **`ms-vscode.powershell`** ❌ REMOVE

**Reason**: Project doesn't use PowerShell scripts
- Uses bash scripts (macOS/Linux)
- Windows scripts would use PowerShell, but none exist yet

**Impact**: Low (only activates on `.ps1` files)

**Recommendation**: **REMOVE** unless you plan to add PowerShell scripts

**Action**:
```bash
cursor --uninstall-extension ms-vscode.powershell
```

---

### 5. **`amazonwebservices.codewhisperer-for-command-line-companion`** ❌ REMOVE

**Reason**: AWS-specific, not used in this project
- Project focuses on IBKR (Interactive Brokers)
- No AWS integration

**Impact**: Low (only activates in terminal)

**Recommendation**: **REMOVE** unless using AWS services

**Action**:
```bash
cursor --uninstall-extension amazonwebservices.codewhisperer-for-command-line-companion
```

---

## ✅ Keep These (Useful)

### Essential Extensions
- ✅ `ms-vscode.cmake-tools` - CMake integration
- ✅ `ms-python.python` - Python support
- ✅ `ms-python.black-formatter` - Python formatting
- ✅ `rust-lang.rust-analyzer` - Rust support
- ✅ `dbaeumer.vscode-eslint` - TypeScript/JavaScript linting
- ✅ `eamodio.gitlens` - Git integration
- ✅ `editorconfig.editorconfig` - EditorConfig
- ✅ `redhat.vscode-yaml` - YAML support
- ✅ `timonwong.shellcheck` - Shell script linting
- ✅ `streetsidesoftware.code-spell-checker` - Spell checking
- ✅ `usernamehw.errorlens` - Inline errors
- ✅ `sswg.swift-lang` - Swift support
- ✅ `vadimcn.vscode-lldb` - C++ debugging
- ✅ `yzhang.markdown-all-in-one` - Markdown
- ✅ `davidanson.vscode-markdownlint` - Markdown linting

---

## 📊 Performance Impact

### High Impact (Remove These)
1. **`vscode-clangd`** - Conflicts with cpptools, causes slowdowns
2. **`cursorpyright`** - Redundant with Pylance (if Pylance is active)

### Medium Impact (Consider Removing)
1. **`powershell`** - Not used, but low overhead
2. **`codewhisperer`** - Not used, but low overhead

### Low Impact (Keep)
- All other extensions are actively used or have minimal overhead

---

## 🎯 Recommended Actions

### Immediate (Performance Issues)

```bash
# Remove conflicting C++ language server
cursor --uninstall-extension llvm-vs-code-extensions.vscode-clangd

# Remove redundant Python type checker (if Pylance is active)
cursor --uninstall-extension anysphere.cursorpyright
```

### Optional (Cleanup)

```bash
# Remove unused extensions
cursor --uninstall-extension ms-vscode.powershell
cursor --uninstall-extension amazonwebservices.codewhisperer-for-command-line-companion
```

### Verify After Removal

```bash
# Check remaining extensions
cursor --list-extensions

# Reload Cursor
# Cmd+Shift+P → "Developer: Reload Window"
```

---

## 🔍 How to Check for Conflicts

### Check C++ IntelliSense

1. Open a C++ file (`native/src/tws_client.cpp`)
2. Check if IntelliSense works
3. Check Output panel → "C/C++" for errors
4. If you see clangd errors, remove `vscode-clangd`

### Check Python IntelliSense

1. Open a Python file (`python/integration/data_provider_router.py`)
2. Check if autocomplete works
3. Check Output panel → "Python Language Server" or "Pylance"
4. If both are active, remove `cursorpyright`

---

## 📈 Expected Performance Improvements

After removing conflicting extensions:

1. **Faster C++ IntelliSense**: No more clangd/cpptools conflicts
2. **Faster Python IntelliSense**: Single language server (Pylance)
3. **Reduced Memory Usage**: Fewer active language servers
4. **Cleaner Diagnostics**: No duplicate error messages

---

## Summary

**Remove Immediately**:
- ❌ `llvm-vs-code-extensions.vscode-clangd` (conflicts with cpptools)
- ⚠️ `anysphere.cursorpyright` (if Pylance is active)

**Remove Optional**:
- ❌ `ms-vscode.powershell` (not used)
- ❌ `amazonwebservices.codewhisperer-for-command-line-companion` (not used)

**Keep**: All other extensions are useful and actively used
