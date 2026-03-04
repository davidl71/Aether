# Scripts Deduplication Plan

## Analysis Summary

Total scripts analyzed: **130+** files in `scripts/` directory

### Identified Duplicates and Consolidation Opportunities

---

## 1. Coverage Scripts (CONSOLIDATE → 1 script)

**Current** (3 scripts):
- `generate_coverage.sh` (62 lines) - Combined C++ and Python
- `generate_cpp_coverage.sh` (79 lines) - C++ only
- `generate_python_coverage.sh` (79 lines) - Python only

**Action**: ✅ KEEP `generate_coverage.sh` (already combines both)
- **DELETE**: `generate_cpp_coverage.sh` (redundant, use `generate_coverage.sh --cpp`)
- **DELETE**: `generate_python_coverage.sh` (redundant, use `generate_coverage.sh --python`)

---

## 2. Documentation Validation (CONSOLIDATE → 1 script)

**Current** (2 scripts):
- `validate_docs_format.py` (199 lines) – **deleted**
- `exarp_validate_docs_format.py` (253 lines) – **removed** (exarp Python tools removed from repo)

**Action**: Use **exarp-go** for docs format/health (check_documentation_health_tool). No local Python exarp scripts.
- **DELETE**: `validate_docs_format.py` (older version) – already gone
- **REMOVED**: `exarp_validate_docs_format.py` – exarp Python tools removed; use exarp-go

---

## 3. Documentation Link Fixing (CONSOLIDATE → 1 script)

**Current** (2 scripts):
- `automate_documentation_link_fixing.py` (267 lines)
- `exarp_fix_documentation_links.py` (87 lines) - **Different implementation**

**Action**: ✅ KEEP `exarp_fix_documentation_links.py` (newer, simpler)
- **DELETE**: `automate_documentation_link_fixing.py` (older version)

---

## 4. Documentation Sync (CONSOLIDATE → 1 script)

**Current** (3 scripts):
- `sync_global_docs.py` (219 lines) - Full sync implementation
- `update_global_docs.sh` (196 lines) - Shell wrapper
- `list_global_docs.sh` (58 lines) - Just lists docs

**Action**: ✅ KEEP `sync_global_docs.py` (primary implementation)
- **DELETE**: `update_global_docs.sh` (redundant wrapper)
- **KEEP**: `list_global_docs.sh` (useful standalone utility)

---

## 5. System Info Collection (CONSOLIDATE → 1 script)

**Current** (2 scripts):
- `collect_system_info.sh` (27 lines) - Simple shell version
- `collect_system_info_python.py` (252 lines) - Comprehensive Python version

**Action**: ✅ KEEP `collect_system_info_python.py` (more comprehensive)
- **DELETE**: `collect_system_info.sh` (limited functionality)

---

## 6. RAM/Ramdisk Setup (CONSOLIDATE → 2 scripts)

**Current** (4 scripts):
- `setup_ramdisk.sh` (190 lines) - Basic ramdisk setup
- `setup_ram_optimization.sh` (395 lines) - Advanced RAM optimization
- `build_ramdisk.sh` (221 lines) - Build on ramdisk
- `workspace_ram_disk_manager.sh` (356 lines) - Full workspace manager

**Action**: ✅ KEEP `workspace_ram_disk_manager.sh` (most comprehensive)
- ✅ KEEP `build_ramdisk.sh` (specific build use case)
- **DELETE**: `setup_ramdisk.sh` (redundant with workspace manager)
- **DELETE**: `setup_ram_optimization.sh` (merged into workspace manager)

---

## 7. TODO/Task Scripts (EVALUATE)

**Current** (10 scripts – exarp-go updates Todo2; no direct edits from this repo):
- `analyze_task_execution_modes.py` - Analysis (read-only) ✅ KEEP
- `audit_in_progress_tasks.py` - Audit (read-only) ✅ KEEP
- ~~`automate_todo2_alignment_v2.py`~~ **Removed** – use exarp-go
- ~~`automate_todo2_duplicate_cleanup.py`~~ **Removed** – no direct Todo2 edits
- ~~`automate_todo2_duplicate_detection.py`~~ **Removed** – use exarp-go
- ~~`batch_update_todos.py`~~ **Removed** – no direct Todo2 edits
- ~~`create_mcp_extensions_todos.py`~~ **Removed** – no direct Todo2 edits
- ~~`exarp_sync_shared_todo.py`~~ **Removed** – use exarp-go sync_todo_tasks_tool
- ~~`process_tasks_parallel.py`~~ **Removed** – no direct Todo2 edits
- ~~`resolve_task_clarifications.py`~~ **Removed** – no direct Todo2 edits

**Action**: exarp-go is the source of task/todo updates. Scripts that wrote to `.todo2/state.todo2.json` were removed. Use exarp-go MCP (task_workflow, sync_todo_tasks_tool, etc.) or CLI.

---

## 8. Build Scripts (KEEP ALL - Different Use Cases)

**Current** (5 scripts):
- `build_fast.sh` - Fast incremental builds with sccache
- `build_distributed.sh` - Distributed builds with distcc
- `build_ramdisk.sh` - Build on ramdisk
- `build_universal.sh` - Universal binary (arm64 + x86_64)
- `build_with_logging.sh` - Build with detailed logging

**Action**: ✅ KEEP ALL (different build strategies)

---

## 9. Test Scripts (KEEP RECENT)

**Current**:
- `test_ibkr.sh` (recently modified Mar 4)
- `test_positions_live.sh` (recently modified Mar 4)
- `test_nats_e2e.sh` (old)
- `test_nats_e2e_flow.sh` (old)
- `test_nats_integration.sh` (old)
- `test_ona_connectivity.sh` (old)

**Action**: 
- ✅ KEEP: Recent IBKR test scripts
- **EVALUATE**: NATS test scripts (if NATS integration is active)
- **DELETE**: `test_ona_connectivity.sh` (outdated, ONA not in use)

---

## 10. Installation Scripts (EVALUATE)

**Current**:
- `install_deb_repo.sh` - For Debian package distribution
- `install_global_kit.sh` - Global kit installation
- `install_ib_gateway.sh` - IB Gateway installation
- `install_mlx.sh` - MLX installation
- `install_nats.sh` - NATS installation
- `install_scancode_env.sh` - ScanCode environment
- `install_shellspec.sh` - ShellSpec testing framework

**Action**: ✅ KEEP ACTIVE, DELETE UNUSED
- ✅ KEEP: `install_ib_gateway.sh`, `install_mlx.sh` (actively used)
- **EVALUATE**: `install_nats.sh` (if NATS is used)
- **DELETE**: `install_scancode_env.sh` (not referenced)
- **DELETE**: `install_shellspec.sh` (not used)

---

## Summary of Deletions

### High Confidence Deletions (12 scripts):
1. ❌ `generate_cpp_coverage.sh` → Use `generate_coverage.sh --cpp`
2. ❌ `generate_python_coverage.sh` → Use `generate_coverage.sh --python`
3. ❌ `validate_docs_format.py` / `exarp_validate_docs_format.py` → Use exarp-go (check_documentation_health_tool)
4. ❌ `automate_documentation_link_fixing.py` → Use `exarp_fix_documentation_links.py`
5. ❌ `update_global_docs.sh` → Use `sync_global_docs.py`
6. ❌ `collect_system_info.sh` → Use `collect_system_info_python.py`
7. ❌ `setup_ramdisk.sh` → Use `workspace_ram_disk_manager.sh`
8. ❌ `setup_ram_optimization.sh` → Use `workspace_ram_disk_manager.sh`
9. ❌ `test_ona_connectivity.sh` → ONA not in use
10. ❌ `install_scancode_env.sh` → Not referenced
11. ❌ `install_shellspec.sh` → Not used
12. ❌ `check_build_status.sh` → Not used (use build scripts directly)

### Space Savings
Estimated: **~2,500 lines** of redundant code removed

---

## Migration Guide

### Before Deletion - Update References

1. **Coverage Scripts**:
   ```bash
   # Old: ./scripts/generate_cpp_coverage.sh
   # New: ./scripts/generate_coverage.sh --cpp
   
   # Old: ./scripts/generate_python_coverage.sh
   # New: ./scripts/generate_coverage.sh --python
   ```

2. **Documentation**:
   ```bash
   # Old: ./scripts/validate_docs_format.py or exarp_validate_docs_format.py
   # New: exarp-go MCP check_documentation_health_tool (workingDirectory = project root)
   
   # Old: ./scripts/automate_documentation_link_fixing.py
   # New: exarp-go MCP/CLI (exarp Python tools removed)
   
   # Old: ./scripts/update_global_docs.sh
   # New: ./scripts/sync_global_docs.py
   ```

3. **System Info**:
   ```bash
   # Old: ./scripts/collect_system_info.sh
   # New: ./scripts/collect_system_info_python.py
   ```

4. **RAM/Ramdisk**:
   ```bash
   # Old: ./scripts/setup_ramdisk.sh
   # New: ./scripts/workspace_ram_disk_manager.sh setup
   
   # Old: ./scripts/setup_ram_optimization.sh
   # New: ./scripts/workspace_ram_disk_manager.sh optimize
   ```

---

## Execution Plan

1. ✅ Create backup branch: `git checkout -b scripts-deduplication`
2. ✅ Remove duplicate scripts
3. ✅ Update SCRIPTS_AUDIT.md
4. ✅ Test that remaining scripts work
5. ✅ Commit and push
6. ✅ Merge to main after validation

---

## Post-Deduplication Metrics

**Before**: 130+ scripts
**After**: ~118 scripts
**Reduction**: ~12 scripts (9%)
**Lines saved**: ~2,500 lines
