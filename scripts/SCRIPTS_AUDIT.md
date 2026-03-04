# Scripts Directory Audit

Last updated: 2026-03-04
Total scripts: 116 (after deduplication)

## Recent Deduplication (2026-03-04)

### Removed Scripts (12 total)

**Duplicates consolidated:**
1. `generate_cpp_coverage.sh` → Use `generate_coverage.sh --cpp`
2. `generate_python_coverage.sh` → Use `generate_coverage.sh --python`
3. `validate_docs_format.py` → Use `exarp_validate_docs_format.py`
4. `automate_documentation_link_fixing.py` → Use `exarp_fix_documentation_links.py`
5. `update_global_docs.sh` → Use `sync_global_docs.py`
6. `collect_system_info.sh` → Use `collect_system_info_python.py`
7. `setup_ramdisk.sh` → Use `workspace_ram_disk_manager.sh`
8. `setup_ram_optimization.sh` → Use `workspace_ram_disk_manager.sh`

**Removed as unused:**
9. `test_ona_connectivity.sh` - ONA integration not in use
10. `install_scancode_env.sh` - Not referenced
11. `install_shellspec.sh` - Not used
12. `check_build_status.sh` - Not used

## Active Script Categories

### Build Scripts (5)
- `build_fast.sh` - Fast builds with sccache ✅ PRIMARY
- `build_distributed.sh` - Distributed builds with distcc
- `build_ramdisk.sh` - Build on ramdisk for performance
- `build_universal.sh` - Universal binary (arm64 + x86_64)
- `build_with_logging.sh` - Build with detailed logging

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
- `exarp_validate_docs_format.py` - Validate doc format ✅ ACTIVE
- `exarp_fix_documentation_links.py` - Fix broken links ✅ ACTIVE
- `sync_global_docs.py` - Sync documentation ✅ ACTIVE
- `list_global_docs.sh` - List all docs
- `generate_docs_summary_tables.py` - Generate doc summaries
- `update_stale_docs.py` - Update stale documentation
- `validate_docs_links.sh` - Validate doc links
- `automate_docs_health_v2.py` - Automated doc health checks

### TODO/Task Management (10)
- `analyze_task_execution_modes.py` - Analyze task execution patterns
- `audit_in_progress_tasks.py` - Audit in-progress tasks
- `automate_todo2_alignment_v2.py` - Align TODO2 with requirements
- `automate_todo2_duplicate_cleanup.py` - Clean duplicate todos
- `automate_todo2_duplicate_detection.py` - Detect duplicate todos
- `batch_update_todos.py` - Batch update operations
- `create_mcp_extensions_todos.py` - Create MCP extension todos
- `exarp_sync_shared_todo.py` - Sync shared todo lists
- `process_tasks_parallel.py` - Parallel task processing
- `resolve_task_clarifications.py` - Resolve task clarifications

### Installation (5)
- `install_ib_gateway.sh` - Install IB Gateway ✅ ACTIVE
- `install_mlx.sh` - Install MLX for Apple Silicon ✅ ACTIVE
- `install_nats.sh` - Install NATS server
- `install_completions.sh` - Install shell completions
- `install_deb_repo.sh` - Install Debian repository
- `install_global_kit.sh` - Install global kit

### Setup (7)
- `workspace_ram_disk_manager.sh` - RAM disk management ✅ CONSOLIDATED
- `setup_notebooks.sh` - Setup Jupyter notebooks
- `setup_platform_settings.sh` - Platform-specific settings
- `setup_pre_commit_hook.sh` - Git pre-commit hooks
- `setup_worktree.sh` - Git worktree setup ✅ ACTIVE
- `setup_github_runner_macos.sh` - GitHub runner (macOS)
- `setup_github_runner_ubuntu.sh` - GitHub runner (Ubuntu)
- `setup_homebrew_tap.sh` - Homebrew tap setup

### Service Management (14)
- `start_ib_service.sh` - Start IB service ✅ ACTIVE
- `start_alpaca_service.sh` - Start Alpaca service
- `start_tastytrade_service.sh` - Start Tastytrade service
- `start_tradestation_service.sh` - Start TradeStation service
- `start_discount_bank_service.sh` - Start Discount Bank service
- `start_risk_free_rate_service.sh` - Start risk-free rate service
- `start_rust_backend.sh` - Start Rust backend
- `start_nats.sh` - Start NATS server
- `start_web_dev.sh` - Start web dev server
- `stop_ib_service.sh` - Stop IB service
- `stop_alpaca_service.sh` - Stop Alpaca service
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
./scripts/exarp_validate_docs_format.py  # Validate docs
./scripts/exarp_fix_documentation_links.py  # Fix links
./scripts/sync_global_docs.py    # Sync documentation
```

**System Setup:**
```bash
./scripts/install_ib_gateway.sh  # Install IB Gateway
./scripts/install_mlx.sh         # Install MLX
./scripts/setup_worktree.sh      # Setup git worktree
```

## Migration Notes

If you were using removed scripts:

| Old Script | New Script | Command |
|------------|------------|---------|
| `generate_cpp_coverage.sh` | `generate_coverage.sh` | `--cpp` flag |
| `generate_python_coverage.sh` | `generate_coverage.sh` | `--python` flag |
| `validate_docs_format.py` | `exarp_validate_docs_format.py` | Direct replacement |
| `automate_documentation_link_fixing.py` | `exarp_fix_documentation_links.py` | Direct replacement |
| `update_global_docs.sh` | `sync_global_docs.py` | Direct replacement |
| `collect_system_info.sh` | `collect_system_info_python.py` | Direct replacement |
| `setup_ramdisk.sh` | `workspace_ram_disk_manager.sh` | `setup` subcommand |
| `setup_ram_optimization.sh` | `workspace_ram_disk_manager.sh` | `optimize` subcommand |

## Maintenance

- Scripts are reviewed periodically for duplicates and dead code
- Unused scripts are moved to `scripts/deprecated/` before deletion
- Breaking changes require migration notes in this file
