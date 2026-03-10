# Sandbox-Friendly Overrides

This repo defaults build, test, and automation caches to workspace-local paths so agent runs and CI do not depend on `$HOME` or raw `/tmp`.

## Default Local Paths

The shared helper is [scripts/include/workspace_paths.sh](/Users/davidl/Projects/Trading/ib_box_spread_full_universal/scripts/include/workspace_paths.sh).

When sourced, it defaults these paths under the repo:

| Variable | Default |
|----------|---------|
| `XDG_CACHE_HOME` | `.cache/` |
| `TMPDIR` | `.cache/tmp/` |
| `UV_CACHE_DIR` | `.cache/uv/` |
| `PIP_CACHE_DIR` | `.cache/pip/` |
| `SCCACHE_DIR` | `.cache/sccache/` |
| `CCACHE_DIR` | `.cache/ccache/` |
| `GOCACHE` | `.cache/go-build/` |
| `GOMODCACHE` | `.cache/go-mod/` |
| `CARGO_HOME` | `.cache/cargo/` |
| `DISTCC_DIR` | `.cache/distcc/` |
| `BUILD_ARTIFACT_ROOT` | `build/test-artifacts/` |

## How To Use

In repo-local scripts:

```bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
. "${SCRIPT_DIR}/include/workspace_paths.sh"
setup_workspace_paths
```

Callers can still override any of these variables explicitly before invoking the script.

## When To Override

Override these only when you intentionally want non-default behavior:

- use a RAM disk for cache-heavy local builds
- point CI at a runner-managed cache directory
- use a system package install path that requires a temporary staging area outside the repo

Example:

```bash
TMPDIR=/Volumes/RAMDisk/tmp SCCACHE_DIR=/Volumes/RAMDisk/sccache ./scripts/build_variant.sh fast
```

## Current Coverage

These entrypoints already use the helper:

- Python test/lint wrappers
- native fast and distributed build scripts
- exarp-go runner wrappers
- selected install/setup scripts such as `install_deb_repo.sh` and `third_party_dmg.sh`

User-facing runtime config defaults are intentionally unchanged in this pass.
