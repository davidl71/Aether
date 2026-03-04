# RAM-Based Development Workflow Optimization

**Purpose**: Maximize development speed using RAM disks and RAM-based tools

---

## Overview

This guide covers all RAM-based optimizations available for speeding up your development workflow. RAM access is **100-1000x faster** than SSD access, making it ideal for:

- Compiler caches (ccache, sccache)
- Build artifacts (temporary builds)
- Package manager caches (pip, npm, cargo)
- Temporary files
- Test artifacts

---

## Quick Start

### Option 1: Comprehensive RAM Optimization (Recommended)

Enable all RAM-based optimizations with one command:

```bash

# Enable RAM optimization (creates 12GB RAM disk, links all caches)

./scripts/setup_ram_optimization.sh enable

# Source environment in current shell

source .ram-optimization-env

# Check status

./scripts/setup_ram_optimization.sh status
```

### Option 2: Build-Only RAM Disk

For faster builds only:

```bash

# Create RAM disk for builds

./scripts/setup_ramdisk.sh create

# Build on RAM disk

./scripts/build_ramdisk.sh build
```

---

## Available Optimizations

### 1. Compiler Caches (ccache/sccache)

**Impact**: 10-100x faster rebuilds

**Setup**:

```bash
./scripts/setup_ram_optimization.sh enable
source .ram-optimization-env
```

**What it does**:

- Links `~/.ccache` → RAM disk
- Links `~/.sccache` → RAM disk
- Sets cache sizes to 4GB each

**Benefits**:

- Cache hits from RAM instead of SSD
- 100x+ faster cache lookups
- Reduced disk wear

**Manual setup**:

```bash

# ccache

ccache --max-size=4G
export CCACHE_DIR="/Volumes/IBBoxSpreadDev/caches/ccache"

# sccache

export SCCACHE_DIR="/Volumes/IBBoxSpreadDev/caches/sccache"
export SCCACHE_CACHE_SIZE="4G"
sccache --stop-server
sccache --start-server
```

---

### 2. Build Artifacts on RAM Disk

**Impact**: 2-5x faster builds (I/O bound)

**Setup**:

```bash

# Create RAM disk

./scripts/setup_ramdisk.sh create

# Build on RAM disk

./scripts/build_ramdisk.sh build

# Or manually

cmake --preset macos-universal-debug -B build-ramdisk
cmake --build build-ramdisk
```

**Benefits**:

- Faster compilation (faster I/O)
- Reduced SSD wear
- Clean builds don't clutter disk

**Size recommendation**: 8-16GB depending on project size

---

### 3. Third-Party Trees on Read-Only Compressed DMG (macOS)

**Impact**: Fewer disk reads for vendor trees (TWS API, Intel decimal); compressed storage; no writes to third-party during builds.

**Setup** (one-time after fetch):

```bash
# Fetch third-party first (if not already done)
./scripts/fetch_third_party.sh

# Create read-only compressed DMG from native/third_party (tws-api, Intel*)
./scripts/third_party_dmg.sh create

# Optional: use DMG in builds (mount and symlink native/third_party -> mount)
export USE_THIRD_PARTY_DMG=1
./scripts/build_fast.sh   # or ensure_third_party will mount when needed
```

**Commands**:

- `./scripts/third_party_dmg.sh create` – Pack current tws-api and IntelRDFPMathLib* into `.dmg/ThirdParty.dmg` (run after fetch).
- `./scripts/third_party_dmg.sh mount` – Attach DMG and symlink `native/third_party/tws-api` and Intel* to the mounted volume (moves originals to `native/third_party/.orig`).
- `./scripts/third_party_dmg.sh unmount` – Restore `.orig`, remove symlinks, detach DMG.
- `./scripts/third_party_dmg.sh status` – Show whether DMG exists, is mounted, and symlinks active.

With `USE_THIRD_PARTY_DMG=1`, `ensure_third_party` (used by build scripts) will run `third_party_dmg.sh mount` when the DMG exists so builds read from the compressed image.

**Benefits**:

- Fewer bytes read from disk (compressed image).
- No writes to vendor trees during build.
- Same `native/third_party` paths for CMake; no config changes.

---

### 3. Python Cache Optimization

**Impact**: Faster Python operations, reduced disk I/O

**Setup**:

```bash
./scripts/setup_ram_optimization.sh enable
source .ram-optimization-env
```

**What it does**:

- Links `~/.cache/pip` → RAM disk
- Sets `PYTHONPYCACHEPREFIX` to RAM disk

**Benefits**:

- Faster pip installs (cache from RAM)
- Faster Python module imports
- Cleaner disk

**Manual setup**:

```bash
export PYTHONPYCACHEPREFIX="/Volumes/IBBoxSpreadDev/caches/pip/__pycache__"
export pip_cache_dir="/Volumes/IBBoxSpreadDev/caches/pip"
```

---

### 4. Rust Cargo Cache

**Impact**: Faster Rust builds (crate downloads and compilation)

**Setup**:

```bash
./scripts/setup_ram_optimization.sh enable
source .ram-optimization-env
```

**What it does**:

- Links `~/.cargo/registry` → RAM disk (crate registry cache)
- Links `~/.cargo/git` → RAM disk (git dependencies cache)
- Sets `CARGO_TARGET_DIR` to RAM disk (build output: `target/` for all Cargo workspaces)

**Benefits**:

- Faster crate downloads (from RAM cache)
- Faster Rust compilation (registry, git, and build output on RAM)
- Reduced disk usage

**Manual setup**:

```bash

# Move cargo registry cache

mv ~/.cargo/registry /Volumes/IBBoxSpreadDev/caches/cargo-registry
ln -sf /Volumes/IBBoxSpreadDev/caches/cargo-registry ~/.cargo/registry

# Move cargo git cache

mv ~/.cargo/git /Volumes/IBBoxSpreadDev/caches/cargo-git
ln -sf /Volumes/IBBoxSpreadDev/caches/cargo-git ~/.cargo/git
```

---

### 5a. Non-C++ project dirs (venv, node_modules)

**Impact**: Faster Python/Node installs and runs when project dirs are on RAM

**Setup**:

```bash
./scripts/setup_ram_optimization.sh enable
source .ram-optimization-env
```

**What it does** (only when the path does not already exist):

- Symlinks project `.venv` → RAM disk (run `uv sync` or `python -m venv .venv` to populate)
- Symlinks project `node_modules` → RAM disk (run `npm install` to populate)
- Symlinks `web/node_modules` → RAM disk (run `npm install` in web to populate)

**Benefits**:

- Faster `uv sync` / `pip install` and Python imports (venv on RAM)
- Faster `npm install` and bundler I/O (node_modules on RAM)

**Note**: If `.venv` or `node_modules` already exist, they are not replaced. Remove them and run `enable` again to put them on the ramdisk.

---

### 5. Node.js Cache

**Impact**: Faster npm/yarn operations

**Setup**:

```bash
./scripts/setup_ram_optimization.sh enable
source .ram-optimization-env
```

**What it does**:

- Links `~/.cache/node` → RAM disk
- Sets `npm_config_cache` environment variable

**Benefits**:

- Faster npm/yarn installs
- Faster package resolution

**Manual setup**:

```bash
export npm_config_cache="/Volumes/IBBoxSpreadDev/caches/node"

# Or for yarn

export YARN_CACHE_FOLDER="/Volumes/IBBoxSpreadDev/caches/yarn"
```

---

### 6. Temporary Files on RAM

**Impact**: Faster temp file operations

**Setup**:

```bash
./scripts/setup_ram_optimization.sh enable
source .ram-optimization-env
```

**What it does**:

- Sets `TMPDIR`, `TMP`, `TEMP` to RAM disk

**Benefits**:

- Faster temporary file I/O
- Cleaner disk (temp files disappear on reboot)

---

### 7. Distributed sccache with Redis (Advanced)

**Impact**: Shared cache across multiple machines

**Setup**:

```bash

# Install Redis (if not installed)

brew install redis

# Setup Redis backend for sccache

./scripts/setup_ram_optimization.sh redis
```

**What it does**:

- Configures sccache to use Redis backend
- Allows sharing cache across multiple development machines
- Can use Redis running on RAM (via Docker with tmpfs)

**Benefits**:

- Shared cache across team/workstations
- Faster clean builds on new machines
- Centralized cache management

**Configuration**:

```bash

# Start Redis

brew services start redis

# Configure sccache

export SCCACHE_REDIS="redis://localhost:6379"
sccache --stop-server
sccache --start-server
```

**Multi-machine setup**:

```bash

# On Redis server (expose Redis)

redis-cli CONFIG SET bind 0.0.0.0

# On client machines

export SCCACHE_REDIS="redis://your-redis-server:6379"
```

---

## Performance Comparison

### Build Performance

| Scenario | Disk | RAM Disk | Improvement |
|----------|------|----------|-------------|
| Clean build | 60-90s | 45-70s | 1.2-1.3x |
| Rebuild (ccache hit) | 1-2s | 0.5-1s | 2x |
| Cache lookup | 10-50ms | 0.1-1ms | 10-500x |

### Cache Access

| Operation | Disk | RAM | Improvement |
|-----------|------|-----|-------------|
| ccache lookup | 10-20ms | 0.1-0.5ms | 20-200x |
| sccache lookup | 5-15ms | 0.1-0.3ms | 15-150x |
| pip cache | 5-10ms | 0.1-0.2ms | 25-100x |

---

## RAM Disk Size Recommendations

### Minimal Setup (8GB)

- Build artifacts only
- For: Small projects, limited RAM

### Recommended Setup (12GB)

- Build artifacts
- Compiler caches (ccache/sccache)
- Python cache
- For: Most development workflows

### Full Setup (16-24GB)

- Build artifacts
- All compiler caches
- All package manager caches
- Temporary files
- For: Heavy development, multiple languages

---

## Scripts Reference

### setup_ram_optimization.sh

Comprehensive RAM optimization for caches and temporary files:

```bash
./scripts/setup_ram_optimization.sh enable   # Enable all optimizations
./scripts/setup_ram_optimization.sh disable  # Disable and restore
./scripts/setup_ram_optimization.sh status   # Show current status
./scripts/setup_ram_optimization.sh redis    # Setup Redis backend
```

### setup_ramdisk.sh

RAM disk for build artifacts:

```bash
./scripts/setup_ramdisk.sh create   # Create RAM disk
./scripts/setup_ramdisk.sh status   # Check status
./scripts/setup_ramdisk.sh unmount  # Unmount (loses data!)
```

### build_ramdisk.sh

Build on RAM disk:

```bash
./scripts/build_ramdisk.sh build   # Build on RAM disk
./scripts/build_ramdisk.sh test    # Run tests
./scripts/build_ramdisk.sh clean   # Clean build
./scripts/build_ramdisk.sh status  # Check status
```

---

## Best Practices

### 1. Start with Cache Optimization

Begin with compiler caches (biggest impact, smallest RAM usage):

```bash
./scripts/setup_ram_optimization.sh enable
source .ram-optimization-env
```

### 2. Add Build RAM Disk for Iterative Development

For active development with frequent rebuilds:

```bash
./scripts/setup_ramdisk.sh create
./scripts/build_ramdisk.sh build
```

### 3. Persistent Environment

Add to your shell profile (`.zshrc` or `.bashrc`):

```bash

# RAM optimization (if RAM disk exists)

if [ -f "${HOME}/path/to/project/.ram-optimization-env" ]; then
  source "${HOME}/path/to/project/.ram-optimization-env"
fi
```

### 4. Automated Startup

Create a startup script:

```bash

#!/bin/bash
# ~/bin/dev-setup.sh

# Enable RAM optimization

cd ~/path/to/project
./scripts/setup_ram_optimization.sh enable || true
source .ram-optimization-env || true

# Create build RAM disk

./scripts/setup_ramdisk.sh create || true
```

### 5. Memory Management

Monitor RAM usage:

```bash

# Check RAM disk usage

df -h /Volumes/IBBoxSpreadDev

# Check cache sizes

du -sh ~/.ccache ~/.sccache ~/.cache/pip

# Check overall RAM usage

vm_stat
```

---

## Troubleshooting

### RAM Disk Not Mounting

```bash

# Check if already mounted

df -h | grep IBBoxSpread

# Unmount any existing instance

./scripts/setup_ramdisk.sh unmount

# Recreate

./scripts/setup_ramdisk.sh create
```

### Cache Links Broken

```bash

# Check links

ls -la ~/.ccache ~/.sccache ~/.cache/pip

# Re-enable

./scripts/setup_ram_optimization.sh disable
./scripts/setup_ram_optimization.sh enable
```

### Out of RAM

If you run out of RAM:

1. Reduce RAM disk size:

   ```bash
   RAMDISK_SIZE_GB=8 ./scripts/setup_ramdisk.sh create
   ```

2. Disable less critical caches:

   ```bash
   # Keep compiler caches, disable package caches
   ```

3. Unmount build RAM disk when not needed:

   ```bash
   ./scripts/setup_ramdisk.sh unmount
   ```

---

## Combining with Other Optimizations

### ccache/sccache + RAM Disk

Best of both worlds:

- Compiler caches in RAM (fast lookups)
- Build artifacts in RAM (fast I/O)

```bash
./scripts/setup_ram_optimization.sh enable
./scripts/setup_ramdisk.sh create
cmake --preset macos-universal-debug -B build-ramdisk -DENABLE_CCACHE=ON
cmake --build build-ramdisk
```

### RAM Disk + Redis sccache

For teams:

- Shared Redis cache (across machines)
- Local RAM disk (fast builds)

```bash
./scripts/setup_ram_optimization.sh enable
./scripts/setup_ram_optimization.sh redis
./scripts/setup_ramdisk.sh create
```

---

## Summary

**Recommended Setup**:

1. **Compiler caches** → RAM (biggest impact, ~4GB)
2. **Build artifacts** → RAM disk (active development, ~8-12GB)
3. **Package caches** → RAM (convenience, ~2-4GB)
4. **Temporary files** → RAM (cleaner disk, ~1GB)

**Total RAM Usage**: ~15-20GB for full optimization

**Expected Speedup**:

- Clean builds: 1.2-1.5x faster
- Rebuilds: 2-10x faster (with cache)
- Cache operations: 10-500x faster

---

For questions or issues, see the main README or open an issue.
