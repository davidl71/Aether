# Cursor Project Commands

**Date**: 2025-01-27
**Location**: `.cursor/commands.json`

This document describes the project-specific commands available in Cursor IDE. These commands can be executed from:

- **Command Palette**: `Cmd+Shift+P` → Type command name
- **AI Chat**: Ask AI to run a command (e.g., "run the build command")
- **Terminal**: Commands are also available as shell scripts

---

## Build Commands

### `build:debug`

Build project in debug mode (ARM64).

**Command**: `cmake --build --preset macos-arm64-debug`

**Use When**: Developing, debugging, or testing

---

### `build:release`

Build project in release mode (ARM64).

**Command**: `cmake --build --preset macos-arm64-release`

**Use When**: Creating production builds or performance testing

---

### `build:configure`

Configure CMake for debug build.

**Command**: `cmake --preset macos-arm64-debug`

**Use When**: First-time setup or after CMake changes

---

### `build:clean`

Clean build artifacts.

**Command**: `cmake --build --preset macos-arm64-debug --target clean`

**Use When**: Troubleshooting build issues or freeing disk space

---

### `build:universal`

Build universal binary (auto-detects architecture).

**Command**: `./scripts/build_universal.sh`

**Use When**: Creating distribution binaries

---

### `build:dependencies`

Build Intel Decimal and TWS API dependencies.

**Command**: Builds both dependency libraries

**Use When**: Setting up project for first time or rebuilding dependencies

---

## Test Commands

### `test:run`

Run all tests with output on failure.

**Command**: `ctest --preset macos-arm64-debug --output-on-failure`

**Use When**: After making code changes or before committing

---

### `test:run-release`

Run tests in release mode.

**Command**: `ctest --preset macos-arm64-release --output-on-failure`

**Use When**: Verifying release build correctness

---

### `test:tws-connection`

Test TWS connection (requires TWS/Gateway running).

**Command**: `./scripts/test_tws_connection.sh`

**Use When**: Verifying TWS API connectivity

---

## Quality Commands

### `lint:run`

Run all linters (cppcheck, clang-tidy, bandit, ESLint, etc.). Includes exarp-go lint when the binary is available.

**Command**: `./scripts/run_linters.sh`

**Use When**: Before committing code or during code review

**Also**: `cmake --build build --target lint` (from configured build dir), or `make lint` (root Makefile).

---

### `lint:exarp`

Run exarp-go lint only (Go/shell/etc.). Requires exarp-go in PATH (e.g. `~/go/bin`).

**Command**: `./scripts/run_exarp_go_tool.sh lint`

**Use When**: Running exarp-go linters separately

---

### `exarp:tool`

Run a specific exarp-go tool (e.g. lint, testing, security). Set `ARG` to the tool name (default: lint).

**Command**: `./scripts/run_exarp_go_tool.sh "${ARG:-lint}"`

**Use When**: Automation or CI; use MCP tools in Cursor for interactive use.

---

### `exarp:list-tools`

List available exarp-go tools.

**Command**: `./scripts/run_exarp_go_tool.sh --list`

**Use When**: Discovering which tools exarp-go provides

---

### `format:code`

Format code using clang-format.

**Command**: Formats all C++ source files

**Use When**: Ensuring consistent code style

---

## Run Commands

### `run:tui`

Run Python TUI application.

**Command**: `python -m python.tui`

**Use When**: Testing TUI interface without live trading

**Environment Variables**:
- `TUI_BACKEND`: `mock`, `rest`, or `file` (default: `mock`)
- `TUI_API_URL`: REST API endpoint (for `rest` backend)
- `TUI_SNAPSHOT_FILE`: JSON file path (for `file` backend)

---

### `run:cli`

Run CLI application (dry-run mode).

**Command**: `./build/macos-arm64-debug/bin/ib_box_spread --dry-run`

**Use When**: Testing CLI without live trading

---

### `run:cli-with-config`

Run CLI with config file (dry-run).

**Command**: `./build/macos-arm64-debug/bin/ib_box_spread --config config/config.json --dry-run`

**Use When**: Testing with specific configuration

---

## Setup Commands

### `setup:worktree`

Setup new git worktree with build.

**Command**: `./scripts/setup_worktree.sh`

**Use When**: Creating isolated development environment

---

### `setup:ramdisk`

Setup RAM disk for faster builds.

**Command**: `./scripts/workspace_ram_disk_manager.sh startup`

**Use When**: Wanting faster build times (requires sufficient RAM)

---

### `ramdisk:status`

Check RAM disk status.

**Command**: `./scripts/workspace_ram_disk_manager.sh status`

**Use When**: Checking if RAM disk is active

---

### `ramdisk:save`

Save RAM disk contents to disk.

**Command**: `./scripts/workspace_ram_disk_manager.sh save`

**Use When**: Preserving RAM disk contents before shutdown

---

### `ramdisk:shutdown`

Save and shutdown RAM disk.

**Command**: `./scripts/workspace_ram_disk_manager.sh shutdown`

**Use When**: Cleaning up RAM disk

---

## Documentation Commands

### `docs:list`

List all global docs paths for Cursor setup.

**Command**: `./scripts/list_global_docs.sh`

**Use When**: Setting up Cursor Docs feature

---

### `docs:sync`

Sync global docs configuration.

**Command**: `python3 scripts/sync_global_docs.py --generate-paths`

**Use When**: Updating documentation paths after adding new docs

---

## Validation Commands

### `validate:config`

Validate configuration file.

**Command**: `./build/macos-arm64-debug/bin/ib_box_spread --config config/config.json --validate`

**Use When**: Checking configuration before running

---

### `check:tws`

Check TWS API download and setup.

**Command**: `./scripts/check_tws_download.sh`

**Use When**: Verifying TWS API installation

---

### `check:feature-parity`

Check feature parity across implementations.

**Command**: `./scripts/check_feature_parity.sh`

**Use When**: Ensuring consistency across language implementations

---

### `check:build-status`

Check current build status and binary locations.

**Command**: Lists built binaries

**Use When**: Finding built executables

---

## Clean Commands

### `clean:all`

Clean all build artifacts and generated files.

**Command**: Removes all build directories and CMake cache

**Use When**: Complete clean rebuild or freeing disk space

---

## Info Commands

### `info:project`

Show project information and status.

**Command**: Displays project name, architecture, and available presets

**Use When**: Quick project status check

---

## Usage Examples

### From Command Palette

1. Press `Cmd+Shift+P` (macOS) or `Ctrl+Shift+P` (Windows/Linux)
2. Type command name (e.g., "build:debug")
3. Press Enter to execute

### From AI Chat

Ask the AI to run commands:

```
Run the build:debug command
```

```
Execute test:run to run all tests
```

```
Check the build status
```

### From Terminal

Commands can also be run directly:

```bash

# Using the command name

cursor-command build:debug

# Or using the underlying command

cmake --build --preset macos-arm64-debug
```

**Make and CMake**: From the repo root you can also use:

- **Make** (wraps CMake presets and scripts): `make build`, `make test`, `make lint`, `make exarp-lint`. Default preset is OS-based (e.g. macos-arm64-debug on macOS); override with `PRESET=linux-x64-debug`. Run `make help`.
- **CMake** (from a configured build dir): `cmake --build build --target lint`, `cmake --build build --target exarp-lint`, `cmake --build build --target exarp-list`.

See `docs/ANSIBLE_SETUP.md` (§ MCP / exarp-go) for exarp-go CI/scripts integration.

---

## Adding New Commands

To add new commands, edit `.cursor/commands.json`:

```json
{
  "name": "command:name",
  "description": "What this command does",
  "command": "actual-shell-command",
  "category": "category-name"
}
```

**Categories**:

- `build` - Build-related commands
- `test` - Testing commands
- `quality` - Linting, formatting, code quality
- `run` - Running applications
- `setup` - Setup and configuration
- `docs` - Documentation management
- `check` - Validation and checking
- `validate` - Configuration validation
- `clean` - Cleanup commands
- `info` - Information commands

---

## Benefits

1. **Quick Access**: No need to remember script paths
2. **Discoverable**: Available in command palette
3. **AI-Friendly**: AI can suggest and execute commands
4. **Consistent**: Standardized command names across team
5. **Documented**: Self-documenting through descriptions

---

## Related Documentation

- [Cursor Setup Guide](research/integration/CURSOR_SETUP.md)
- [Build System Documentation](research/integration/DISTRIBUTED_COMPILATION.md)
- [Testing Guide](platform/README.md)
