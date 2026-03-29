# Scripts Directory Audit

Last updated: 2026-03-04
Total scripts: 116 (after deduplication)

## Recent Deduplication (2026-03-04)

### Removed Scripts (historical log; counts are approximate)

**Duplicates consolidated:**

1. `generate_cpp_coverage.sh` → Use `generate_coverage.sh --cpp`
2. `generate_python_coverage.sh` → Use `generate_coverage.sh --python`
3. `validate_docs_format.py` → ~~Use `exarp_validate_docs_format.py`~~ **Removed.** Use exarp-go (`health` with `action=docs`) for docs format/health.
4. ~~`automate_documentation_link_fixing.py` → `exarp_fix_documentation_links.py`~~ **Removed.** Use exarp-go for docs health/link fixing.
5. `update_global_docs.sh` → Use `sync_global_docs.py`
6. `collect_system_info.sh` → Use `collect_system_info_python.py`
7. `setup_ramdisk.sh`, `setup_ram_optimization.sh`, `workspace_ram_disk_manager.sh` → **Removed or absent from repo.** Use `setup_disk_caching.sh` for cache layout; optional CMake `*-ramdisk` presets use a `build-ramdisk` binary directory.

**Removed as unused:**
8. `test_ona_connectivity.sh` - ONA integration not in use
9. `install_scancode_env.sh` - Not referenced
10. `install_shellspec.sh` - Not used
11. `check_build_status.sh` - Not used

## Active Script Categories

### Build Scripts

- `build_fast.sh` - Fast builds with sccache ✅ PRIMARY
- `build_distributed.sh` - Distributed builds with distcc
- `build_universal.sh` - Universal binary (arm64 + x86_64)
- `build_with_logging.sh` - Build with detailed logging
- ~~`build_ramdisk.sh`~~ **Removed** – use CMake presets `*-ramdisk` (`binaryDir`: `build-ramdisk`) if you want a separate build tree

### Test Scripts (7)

- `test_ibkr.sh` - IBKR connection tests ✅ ACTIVE
- `test_positions_live.sh` - Live position retrieval ✅ ACTIVE  
- `test_tws_connection.sh` - TWS connection validation
- `test_nats_e2e.sh` - NATS end-to-end tests
- `test_nats_e2e_flow.sh` - NATS flow tests
- `test_nats_integration.sh` - NATS integration tests
- `run_tests.sh` - Main test runner ✅ PRIMARY

### Coverage/Analysis (1)

- `generate_coverage.sh` - Combined C++/Python coverage ✅ CONSOLIDATED

### Documentation (7)

- ~~`exarp_validate_docs_format.py`~~ **Removed** – use exarp-go (`health` with `action=docs`)
- ~~`exarp_fix_documentation_links.py`~~ **Removed** – use exarp-go
- `sync_global_docs.py` - Sync documentation ✅ ACTIVE
- `list_global_docs.sh` - List all docs
- `generate_docs_summary_tables.py` - Generate doc summaries
- `update_stale_docs.py` - Update stale documentation
- `validate_docs_links.sh` - Validate doc links
- ~~`automate_docs_health_v2.py`~~ **Removed** – use exarp-go MCP/CLI

### TODO/Task Management (10)

- `analyze_task_execution_modes.py` - Analyze task execution patterns
- `audit_in_progress_tasks.py` - Audit in-progress tasks
- ~~`automate_todo2_alignment_v2.py`~~ **Removed** – use exarp-go MCP/CLI
- ~~`automate_todo2_duplicate_cleanup.py`~~ **Removed** – no direct Todo2 edits; use exarp-go
- ~~`automate_todo2_duplicate_detection.py`~~ **Removed** – use exarp-go MCP/CLI
- ~~`batch_update_todos.py`~~ **Removed** – no direct Todo2 edits; use exarp-go
- ~~`create_mcp_extensions_todos.py`~~ **Removed** – no direct Todo2 edits; use exarp-go
- ~~`exarp_sync_shared_todo.py`~~ **Removed** – use exarp-go sync_todo_tasks_tool
- ~~`process_tasks_parallel.py`~~ **Removed** – no direct Todo2 edits; use exarp-go
- ~~`resolve_task_clarifications.py`~~ **Removed** – no direct Todo2 edits; use exarp-go

### Installation (5)

- `install_ib_gateway.sh` - Install IB Gateway ✅ ACTIVE
- `install_mlx.sh` - Install MLX for Apple Silicon ✅ ACTIVE
- `install_nats.sh` - Install NATS server
- `install_completions.sh` - Install shell completions
- `install_deb_repo.sh` - Install Debian repository
- `install_global_kit.sh` - Install global kit

### Setup

- `setup_disk_caching.sh` - Disk-based dev caches (replaces RAM-disk helper scripts)
- `setup_platform_settings.sh` - Platform-specific settings
- `setup_pre_commit_hook.sh` - Git pre-commit hooks
- `setup_worktree.sh` - REMOVED (native build retired; use `git worktree add`)
- `setup_github_runner_macos.sh` - GitHub runner (macOS)
- `setup_github_runner_ubuntu.sh` - GitHub runner (Ubuntu)
- `setup_homebrew_tap.sh` - Homebrew tap setup

### Service Management (14)

- `start_ib_service.sh` - Removed; IB daemon retired from active service management
- `start_alpaca_service.sh` - Removed; Alpaca daemon retired from active service management
- `start_tastytrade_service.sh` - Removed; Tastytrade daemon retired from active service management
- `start_discount_bank_service.sh` - Removed; Discount Bank daemon retired from active service management
- `start_risk_free_rate_service.sh` - Removed; standalone risk-free-rate daemon retired from active service management
- `start_rust_backend.sh` - Start Rust backend
- `start_nats.sh` - Start NATS server
- `start_web_dev.sh` - Archived web helper; web is not an active runtime
- `stop_ib_service.sh` - Removed; IB daemon retired from active service management
- `stop_alpaca_service.sh` - Removed; Alpaca daemon retired from active service management
- (... other stop scripts ...)
- `watchdog_services.sh` - Service watchdog

### Utilities (15+)

- `fetch_third_party.sh` - Fetch third-party dependencies ✅ ACTIVE
- `collect_system_info_python.py` - System info collection ✅ CONSOLIDATED
- `diagnose_ibkr.sh` - IBKR diagnostics ✅ ACTIVE
- `run_linters.sh` - Run all linters ✅ ACTIVE
- `generate_completions.sh` - Generate shell completions
- `deduplicate_mcp_servers.py` - Deduplicate MCP config
- (... other utilities ...)

## Script Usage Guidelines

### Recommended Scripts for Common Tasks

**Building:**

```bash
./scripts/fetch_third_party.sh  # First time setup
./scripts/build_fast.sh          # Normal builds
./scripts/build_universal.sh     # Release builds
```

**Testing:**

```bash
./scripts/test_ibkr.sh           # Test IBKR connection
./scripts/run_tests.sh           # Run all tests
./scripts/generate_coverage.sh   # Generate coverage
```

**Documentation:**

```bash
# Docs format/health: use exarp-go MCP (`health` with `action=docs`) or CLI
./scripts/sync_global_docs.py    # Sync documentation
# See docs/MCP_REQUIRED_SERVERS.md for exarp-go setup
```

**System Setup:**

```bash
./scripts/install_ib_gateway.sh  # Install IB Gateway
./scripts/install_mlx.sh         # Install MLX
git worktree add <path> [branch] # Setup git worktree (script removed)
```

## Migration Notes

If you were using removed scripts:

| Old Script | New Script | Command |
|------------|------------|---------|
| `generate_cpp_coverage.sh` | `generate_coverage.sh` | `--cpp` flag |
| `generate_python_coverage.sh` | `generate_coverage.sh` | `--python` flag |
| `validate_docs_format.py` | **Removed** | Use exarp-go `health` with `action=docs` |
| `exarp_validate_docs_format.py` | **Removed** | Use exarp-go (exarp Python tools removed) |
| ~~`automate_documentation_link_fixing.py` / `exarp_fix_documentation_links.py`~~ | **Removed** | Use exarp-go MCP/CLI |
| `update_global_docs.sh` | `sync_global_docs.py` | Direct replacement |
| `collect_system_info.sh` | `collect_system_info_python.py` | Direct replacement |
| `setup_ramdisk.sh` | `setup_disk_caching.sh` | `enable` / `disable` / `status` |
| `setup_ram_optimization.sh` | `setup_disk_caching.sh` | same |
| `workspace_ram_disk_manager.sh` | **Removed** | Use `setup_disk_caching.sh` + CMake `*-ramdisk` presets |

## Maintenance

- Scripts are reviewed periodically for duplicates and dead code
- Unused scripts are moved to `scripts/deprecated/` before deletion
- Breaking changes require migration notes in this file
