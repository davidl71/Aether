# Portable build and exarp-go runner

This document describes the portable build wrapper and the exarp-go runner used across macOS (Intel/ARM) and Linux.

## Portable build wrapper

**Script**: `scripts/build_portable.sh`  
**Just**: `just build-portable [build|clean|test|install] [--debug|--release]`

Detects OS and architecture and uses the matching CMake preset:

| Platform        | Preset              |
|-----------------|---------------------|
| macOS arm64     | `macos-arm64-{debug,release}` |
| macOS x86_64    | `macos-x86_64-{debug,release}` |
| Linux x86_64    | `linux-x64-{debug,release}`   |
| Linux aarch64   | `linux-aarch64-{debug,release}` |

### Usage

```bash
./scripts/build_portable.sh                    # build (release)
./scripts/build_portable.sh --debug            # build debug
./scripts/build_portable.sh test               # configure + run tests
./scripts/build_portable.sh clean              # clean build dir
./scripts/build_portable.sh install            # install (if preset supports it)

# Override preset or build type
CMAKE_PRESET=linux-x64-debug ./scripts/build_portable.sh
CMAKE_BUILD_TYPE=Debug ./scripts/build_portable.sh

# Use Nix dev shell (when flake.nix exists)
USE_NIX=1 ./scripts/build_portable.sh
just nix build-portable
```

### Presets

Presets are defined in `CMakePresets.json`. The script does not create the build directory for you; the first run will configure via `cmake --preset <name>`.

---

## exarp-go runner (portable)

**Script**: `scripts/run_exarp_go.sh`

Used as the exarp-go MCP server command in `.cursor/mcp.json`. Ensures the correct project is used via `PROJECT_ROOT` (e.g. `.todo2` and task store).

### Resolution order

1. **Within exarp-go working dir**  
   If CWD or `EXARP_GO_ROOT` is the exarp-go repo (detected via `go.mod` containing `exarp-go` and presence of `main.go`/`cmd/`/`bin/exarp-go`):
   - Use that repo's `bin/exarp-go` if built, otherwise `go run .` in that repo.

2. **Otherwise**  
   Use globally installed `exarp-go` from PATH.

3. **Fallback**  
   If global is not found:
   - `EXARP_GO_ROOT/bin/exarp-go` (if set and present)
   - `PROJECT_ROOT/../exarp-go/bin/exarp-go`
   - Then `~/go/bin/exarp-go`, `~/Projects/exarp-go/bin/exarp-go`, `/usr/local/bin/exarp-go`

### Environment

| Variable             | Purpose |
|----------------------|--------|
| `PROJECT_ROOT`       | Project exarp-go serves (set by script to repo root if unset). |
| `EXARP_GO_ROOT`      | exarp-go repo root; used for working-dir build and migrations. |
| `EXARP_GO_VERBOSE=1` | Log which exarp-go binary/source is used. |
| `EXARP_MIGRATIONS_DIR` | Set automatically from `EXARP_GO_ROOT/migrations` when using working-dir. |

### Usage

- **From this project**: MCP uses the script as-is; no change needed.
- **From exarp-go repo**: `cd /path/to/exarp-go` then invoke the script (e.g. via full path); local build or `go run .` is used.
- **Override repo**: `EXARP_GO_ROOT=/path/to/exarp-go ./scripts/run_exarp_go.sh …`
- **Debug**: `EXARP_GO_VERBOSE=1 ./scripts/run_exarp_go.sh -list`

### Related scripts

- **`scripts/init_exarp_todo2_db.sh`** — Runs `exarp-go task sync` with the same resolution (global, then working-dir fallback). See [EXARP_TODO2_DB_INIT.md](EXARP_TODO2_DB_INIT.md).
- **`scripts/run_exarp_go_tool.sh`** — Invokes `run_exarp_go.sh` with `-tool <name>` for lint and other tools.

---

## References

- [CURSOR_PROJECT_COMMANDS.md](CURSOR_PROJECT_COMMANDS.md) — `build:portable`, `build:universal`
- [EXARP_TODO2_DB_INIT.md](EXARP_TODO2_DB_INIT.md) — Todo2 DB setup and init script
- [NIX_MIGRATION_PLAN.md](planning/NIX_MIGRATION_PLAN.md) — Nix dev shell and `USE_NIX`
