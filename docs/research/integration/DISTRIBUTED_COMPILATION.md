# Distributed Compilation Setup

**Date**: 2025-01-27
**Purpose**: Enable fast distributed compilation with distcc and ccache

---

## Overview

This project supports distributed compilation using **distcc** and **ccache** to dramatically reduce build times. Your system already has distcc installed, and we can leverage it for faster development iterations.

**Tools Available**:

- ✅ `distcc` - Distributed C/C++ compilation across network
- ❌ `ccache` - Not installed (optional, but highly recommended)
- ❌ `sccache` - Not installed (Rust-based alternative to ccache)

---

## Compilation Speed Tools Comparison

| Tool         | Purpose                               | Speed Improvement | Use Case                     |
| ------------ | ------------------------------------- | ----------------- | ---------------------------- |
| **distcc**   | Distribute compilation across network | 2-10x             | Multiple machines available  |
| **ccache**   | Cache compilation results locally     | 10-100x           | Repeated builds              |
| **sccache**  | Cache + distributed (Rust-based)      | 10-100x           | Modern alternative to ccache |
| **icecream** | Distributed with smart scheduling     | 2-10x             | Alternative to distcc        |

**Best Combination**: `ccache` + `distcc` (cache first, distribute misses)

---

## Quick Setup

### Option 1: distcc Only (Current System)

Already have distcc installed! Just enable it:

```bash

# Set up distcc hosts (add your machines)

export DISTCC_HOSTS="localhost/4 192.168.1.100/8 192.168.1.101/8"

# Configure CMake to use distcc

cd /Users/davidlowes/Documents/ib_box_spread_full_universal
cmake -S . -B build \
  -DCMAKE_CXX_COMPILER_LAUNCHER=distcc \
  -DIBAPI_INCLUDE_DIR=~/IBJts/source/cppclient \
  -DIBAPI_LIB=~/IBJts/source/cppclient/libTwsApiCpp.dylib

# Build with parallelism

make -j16 -C build
```

### Option 2: ccache + distcc (Recommended)

Install ccache first:

```bash

# Install ccache

brew install ccache

# Set up environment

export DISTCC_HOSTS="localhost/4 192.168.1.100/8"
export CCACHE_PREFIX="distcc"  # Use distcc via ccache

# Configure CMake

cmake -S . -B build \
  -DCMAKE_CXX_COMPILER_LAUNCHER=ccache \
  -DIBAPI_INCLUDE_DIR=~/IBJts/source/cppclient \
  -DIBAPI_LIB=~/IBJts/source/cppclient/libTwsApiCpp.dylib

# Build

make -j16 -C build
```

---

## Detailed Configuration

### distcc Setup

#### 1. Server Configuration (on remote machines)

```bash

# On each remote machine (Linux or macOS)
# Install distcc

brew install distcc  # macOS

# or

sudo apt-get install distcc  # Linux

# Start distccd daemon

distccd --daemon --allow 192.168.1.0/24 --jobs 8 --log-level error

# Verify running

ps aux | grep distccd
```

#### 2. Client Configuration (your development machine)

```bash

# Create ~/.distcc/hosts file

cat > ~/.distcc/hosts << 'EOF'

# Format: hostname/limit

localhost/4
192.168.1.100/8
192.168.1.101/8
EOF

# Or use environment variable

export DISTCC_HOSTS="localhost/4 192.168.1.100/8 192.168.1.101/8"

# Enable verbose logging (optional)

export DISTCC_VERBOSE=1
export DISTCC_LOG=/tmp/distcc.log
```

#### 3. CMake Integration

Add to your build configuration:

```bash
cmake -S . -B build \
  -DCMAKE_CXX_COMPILER_LAUNCHER=distcc \
  -DCMAKE_C_COMPILER_LAUNCHER=distcc \
  -DIBAPI_INCLUDE_DIR=~/IBJts/source/cppclient \
  -DIBAPI_LIB=~/IBJts/source/cppclient/libTwsApiCpp.dylib
```

Or add to `CMakeLists.txt`:

```cmake

# Distributed compilation support

option(ENABLE_DISTCC "Enable distcc distributed compilation" OFF)
option(ENABLE_CCACHE "Enable ccache compilation caching" OFF)

if(ENABLE_DISTCC)
    find_program(DISTCC_EXECUTABLE distcc)
    if(DISTCC_EXECUTABLE)
        message(STATUS "Found distcc: ${DISTCC_EXECUTABLE}")
        set(CMAKE_CXX_COMPILER_LAUNCHER ${DISTCC_EXECUTABLE})
        set(CMAKE_C_COMPILER_LAUNCHER ${DISTCC_EXECUTABLE})
    else()
        message(WARNING "distcc not found, distributed compilation disabled")
    endif()
endif()

if(ENABLE_CCACHE)
    find_program(CCACHE_EXECUTABLE ccache)
    if(CCACHE_EXECUTABLE)
        message(STATUS "Found ccache: ${CCACHE_EXECUTABLE}")
        set(CMAKE_CXX_COMPILER_LAUNCHER ${CCACHE_EXECUTABLE})
        set(CMAKE_C_COMPILER_LAUNCHER ${CCACHE_EXECUTABLE})

        # Use distcc via ccache if both enabled
        if(ENABLE_DISTCC AND DISTCC_EXECUTABLE)
            message(STATUS "Using ccache with distcc backend")
            set(ENV{CCACHE_PREFIX} "distcc")
        endif()
    else()
        message(WARNING "ccache not found, caching disabled")
    endif()
endif()
```

---

## ccache Setup (Recommended)

### Installation

```bash

# macOS

brew install ccache

# Linux

sudo apt-get install ccache
```

### Configuration

```bash

# Configure ccache

ccache --max-size=10G  # Set cache size
ccache --set-config=compression=true
ccache --set-config=compression_level=6

# View configuration

ccache --show-config

# View statistics

ccache --show-stats
```

### CMake Integration

```bash
cmake -S . -B build \
  -DCMAKE_CXX_COMPILER_LAUNCHER=ccache \
  -DENABLE_CCACHE=ON \
  -DIBAPI_INCLUDE_DIR=~/IBJts/source/cppclient \
  -DIBAPI_LIB=~/IBJts/source/cppclient/libTwsApiCpp.dylib
```

---

## sccache Setup (Modern Alternative)

sccache is a Rust-based compilation cache that's faster and more feature-rich than ccache.

### Installation

```bash

# macOS

brew install sccache

# Or download from GitHub
# https://github.com/mozilla/sccache/releases
```

### Configuration

```bash

# Set environment variables

export SCCACHE_DIR=~/.sccache
export SCCACHE_CACHE_SIZE="10G"

# CMake integration

cmake -S . -B build \
  -DCMAKE_CXX_COMPILER_LAUNCHER=sccache \
  -DIBAPI_INCLUDE_DIR=~/IBJts/source/cppclient \
  -DIBAPI_LIB=~/IBJts/source/cppclient/libTwsApiCpp.dylib

# View statistics

sccache --show-stats
```

---

## Performance Comparison

### This Project (Estimated)

Current build structure:

- ~20 C++ source files
- TWS API library (pre-built)
- Tests
- Python bindings (Cython)

**Without any optimization**: ~60-90 seconds (clean build)
**With ccache**: ~3-5 seconds (cached rebuild)
**With distcc (4 machines × 4 cores)**: ~15-20 seconds (clean build)
**With ccache + distcc**: ~3-5 seconds (cached), ~15-20 seconds (clean)

### Benchmark Expectations

| Build Type                   | No Cache | With ccache | With distcc | With Both |
| ---------------------------- | -------- | ----------- | ----------- | --------- |
| Clean build                  | 60-90s   | 60-90s      | 15-20s      | 15-20s    |
| Incremental (1 file changed) | 5-10s    | 2-3s        | 3-5s        | 2-3s      |
| Incremental (header changed) | 30-40s   | 3-5s        | 10-15s      | 3-5s      |
| Rebuild (no changes)         | 60-90s   | 1-2s        | 15-20s      | 1-2s      |

---

## Recommended Configuration for This Project

### For Solo Development

```bash

# Use ccache only (most benefit for solo dev)

brew install ccache
cmake -S . -B build -DENABLE_CCACHE=ON ...
```

### For Team Development

```bash

# Use ccache + distcc

brew install ccache distcc

# Set up distcc hosts

export DISTCC_HOSTS="localhost/4 team-machine-1/8 team-machine-2/8"
export CCACHE_PREFIX="distcc"

# Configure

cmake -S . -B build -DENABLE_CCACHE=ON -DENABLE_DISTCC=ON ...
```

### For CI/CD

```bash

# Use sccache with cloud storage backend
# Supports S3, Redis, GCS for shared cache

export SCCACHE_BUCKET="my-build-cache"
export SCCACHE_REGION="us-east-1"

cmake -S . -B build -DCMAKE_CXX_COMPILER_LAUNCHER=sccache ...
```

---

## Implementation Steps

### Step 1: Add CMake Support

Update `CMakeLists.txt` to add distributed compilation options:

```cmake

# Add after line 22

option(ENABLE_DISTCC "Enable distcc distributed compilation" OFF)
option(ENABLE_CCACHE "Enable ccache compilation caching" OFF)
option(ENABLE_SCCACHE "Enable sccache compilation caching" OFF)

# Add after compiler flags section (after line 60)
# ============================================================================
# Distributed Compilation and Caching
# ============================================================================

# Prioritize: sccache > ccache > distcc

if(ENABLE_SCCACHE)
    find_program(SCCACHE_EXECUTABLE sccache)
    if(SCCACHE_EXECUTABLE)
        message(STATUS "Using sccache: ${SCCACHE_EXECUTABLE}")
        set(CMAKE_CXX_COMPILER_LAUNCHER ${SCCACHE_EXECUTABLE})
        set(CMAKE_C_COMPILER_LAUNCHER ${SCCACHE_EXECUTABLE})
    else()
        message(WARNING "sccache not found, falling back to ccache/distcc")
        set(ENABLE_SCCACHE OFF)
    endif()
endif()

if(ENABLE_CCACHE AND NOT ENABLE_SCCACHE)
    find_program(CCACHE_EXECUTABLE ccache)
    if(CCACHE_EXECUTABLE)
        message(STATUS "Using ccache: ${CCACHE_EXECUTABLE}")
        set(CMAKE_CXX_COMPILER_LAUNCHER ${CCACHE_EXECUTABLE})
        set(CMAKE_C_COMPILER_LAUNCHER ${CCACHE_EXECUTABLE})

        # Use distcc via ccache if both enabled
        if(ENABLE_DISTCC)
            find_program(DISTCC_EXECUTABLE distcc)
            if(DISTCC_EXECUTABLE)
                message(STATUS "Using ccache with distcc backend")
                set(ENV{CCACHE_PREFIX} "distcc")
            endif()
        endif()
    else()
        message(WARNING "ccache not found, trying distcc only")
        set(ENABLE_CCACHE OFF)
    endif()
endif()

if(ENABLE_DISTCC AND NOT ENABLE_CCACHE AND NOT ENABLE_SCCACHE)
    find_program(DISTCC_EXECUTABLE distcc)
    if(DISTCC_EXECUTABLE)
        message(STATUS "Using distcc: ${DISTCC_EXECUTABLE}")
        set(CMAKE_CXX_COMPILER_LAUNCHER ${DISTCC_EXECUTABLE})
        set(CMAKE_C_COMPILER_LAUNCHER ${DISTCC_EXECUTABLE})
    else()
        message(WARNING "distcc not found, distributed compilation disabled")
    endif()
endif()
```

### Step 2: Install ccache (Highly Recommended)

```bash
brew install ccache
```

### Step 3: Set Up distcc Servers (Optional)

If you have multiple machines:

```bash

# On each remote machine

brew install distcc
distccd --daemon --allow 192.168.1.0/24 --jobs 8
```

### Step 4: Build with Optimizations

```bash

# Clean rebuild with ccache

cmake -S . -B build-fast \
  -DCMAKE_BUILD_TYPE=Release \
  -DENABLE_CCACHE=ON \
  -DIBAPI_INCLUDE_DIR=~/IBJts/source/cppclient \
  -DIBAPI_LIB=~/IBJts/source/cppclient/libTwsApiCpp.dylib

make -j$(sysctl -n hw.ncpu) -C build-fast
```

---

## Build Scripts

### scripts/build_fast.sh (with ccache)

```bash

#!/bin/bash
# build_fast.sh - Fast build with ccache

set -e

# Install ccache if not present

if ! command -v ccache &> /dev/null; then
    echo "Installing ccache..."
    brew install ccache
fi

# Configure ccache

ccache --max-size=10G
ccache --set-config=compression=true

# Build directory

BUILD_DIR="build-fast"

# Configure with ccache

cmake -S . -B "$BUILD_DIR" \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_CXX_COMPILER_LAUNCHER=ccache \
  -DENABLE_LTO=ON \
  -DIBAPI_INCLUDE_DIR="${IBAPI_INCLUDE_DIR:-$HOME/IBJts/source/cppclient}" \
  -DIBAPI_LIB="${IBAPI_LIB:-$HOME/IBJts/source/cppclient/libTwsApiCpp.dylib}"

# Build with all cores

make -j$(sysctl -n hw.ncpu) -C "$BUILD_DIR"

# Show ccache statistics

echo ""
echo "=== ccache Statistics ==="
ccache --show-stats
```

### scripts/build_distributed.sh (with distcc + ccache)

```bash

#!/bin/bash
# build_distributed.sh - Distributed build with distcc and ccache

set -e

# Check for required tools

if ! command -v distcc &> /dev/null; then
    echo "Error: distcc not found"
    exit 1
fi

if ! command -v ccache &> /dev/null; then
    echo "Installing ccache..."
    brew install ccache
fi

# distcc hosts (customize for your network)

export DISTCC_HOSTS="${DISTCC_HOSTS:-localhost/4}"
export CCACHE_PREFIX="distcc"

# Configure ccache

ccache --max-size=10G

# Build directory

BUILD_DIR="build-distributed"

# Configure

cmake -S . -B "$BUILD_DIR" \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_CXX_COMPILER_LAUNCHER=ccache \
  -DENABLE_LTO=ON \
  -DIBAPI_INCLUDE_DIR="${IBAPI_INCLUDE_DIR:-$HOME/IBJts/source/cppclient}" \
  -DIBAPI_LIB="${IBAPI_LIB:-$HOME/IBJts/source/cppclient/libTwsApiCpp.dylib}"

# Build with high parallelism

make -j32 -C "$BUILD_DIR"

# Statistics

echo ""
echo "=== ccache Statistics ==="
ccache --show-stats
echo ""
echo "=== distcc Statistics ==="
distcc --show-stats
```

---

## distcc Server Setup

### macOS Server

```bash

# Install distcc

brew install distcc

# Start distccd daemon

distccd --daemon \
  --allow 192.168.1.0/24 \
  --jobs $(sysctl -n hw.ncpu) \
  --log-level error \
  --log-file /tmp/distccd.log

# Verify running

ps aux | grep distccd

# Test from client

DISTCC_HOSTS="server-ip/8" distcc gcc --version
```

### Linux Server

```bash

# Install distcc

sudo apt-get install distcc

# Configure

sudo systemctl enable distcc
sudo systemctl start distcc

# Edit /etc/default/distcc

STARTDISTCC="true"
ALLOWEDNETS="192.168.1.0/24"
LISTENER="0.0.0.0"
JOBS="8"

# Restart

sudo systemctl restart distcc
```

---

## Performance Tuning

### Optimal Parallelism

```bash

# Formula: (local_cores + sum(remote_cores)) * 1.5
# Example: 4 local cores + 8 remote + 8 remote = 20 cores
# Optimal -j flag: 20 * 1.5 = 30

make -j30 -C build
```

### Monitor Performance

```bash

# distcc monitor (shows which hosts are being used)

distccmon-text 1  # Update every 1 second

# ccache statistics

watch -n 1 ccache --show-stats

# Real-time build progress

make -j16 -C build 2>&1 | tee build.log
```

### Troubleshooting

```bash

# Enable verbose logging

export DISTCC_VERBOSE=1
export DISTCC_LOG=/tmp/distcc.log

# Check what's happening

tail -f /tmp/distcc.log

# Pump mode (preprocessing distribution)
# Faster for large projects

pump make -j32 -C build
```

---

## Cloud-Based Distributed Compilation

### Option 1: AWS EC2 Build Farm

```bash

# Launch EC2 instances for compilation
# Use spot instances for cost savings
# Configure distcc to use EC2 IPs

export DISTCC_HOSTS="localhost/4 \
  ec2-1.compute.amazonaws.com/16 \
  ec2-2.compute.amazonaws.com/16 \
  ec2-3.compute.amazonaws.com/16"

make -j64 -C build
```

### Option 2: sccache with S3 Backend

```bash

# Install sccache

brew install sccache

# Configure S3 backend

export SCCACHE_BUCKET="my-build-cache"
export SCCACHE_REGION="us-east-1"
export AWS_ACCESS_KEY_ID="..."
export AWS_SECRET_ACCESS_KEY="..."

# Use with CMake

cmake -S . -B build \
  -DCMAKE_CXX_COMPILER_LAUNCHER=sccache \
  -DENABLE_SCCACHE=ON \
  ...

# All team members share the same S3 cache!
```

---

## Benchmarking

### Measure Build Times

```bash

# Benchmark script
#!/bin/bash

echo "=== Build Time Benchmark ==="

# Clean build without cache

echo "1. Clean build (no cache)..."
rm -rf build-bench
time cmake -S . -B build-bench ... && time make -j4 -C build-bench

# Rebuild without changes

echo "2. Rebuild (no changes, with ccache)..."
time make -j4 -C build-bench

# Change one file

echo "3. Incremental (1 file changed)..."
touch src/box_spread_strategy.cpp
time make -j4 -C build-bench

# Show ccache stats

ccache --show-stats
```

---

## Integration with build_universal.sh

Update `build_universal.sh` to support distributed compilation:

```bash

#!/bin/bash
# Add at the top

# Enable ccache if available

if command -v ccache &> /dev/null; then
    export CMAKE_CXX_COMPILER_LAUNCHER=ccache
    echo "Using ccache for faster builds"

    # Use distcc via ccache if available
    if command -v distcc &> /dev/null && [ -n "$DISTCC_HOSTS" ]; then
        export CCACHE_PREFIX=distcc
        echo "Using distcc via ccache: $DISTCC_HOSTS"
    fi
fi

# Rest of script...
```

---

## CI/CD Integration

### GitHub Actions

```yaml
name: Build

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup ccache
        uses: hendrikmuhs/ccache-action@v1
        with:
          key: ${{ runner.os }}-build
          max-size: 1G

      - name: Configure
        run: |
          cmake -S . -B build \
            -DCMAKE_CXX_COMPILER_LAUNCHER=ccache \
            -DCMAKE_BUILD_TYPE=Release

      - name: Build
        run: make -j$(nproc) -C build
```

---

## Recommendations

### For This Project

1. **Immediate**: Install ccache

   ```bash
   brew install ccache
   ```

2. **Add to CMakeLists.txt**: Support for ccache/distcc options

3. **Update build scripts**: Enable ccache by default

4. **Document in README**: Add build optimization section

### For Your Workflow

- **Solo Development**: ccache only (10-100x speedup on rebuilds)
- **Team of 2-3**: ccache + distcc (2-3x faster clean builds)
- **Team of 4+**: ccache + distcc + multiple servers (3-5x faster)
- **CI/CD**: sccache with S3 backend (shared cache across builds)

---

## Testing Distributed Compilation

```bash

# Test 1: Verify distcc is working

DISTCC_VERBOSE=1 distcc g++ -c test.cpp -o test.o

# Test 2: Build with statistics

time make -j16 -C build
distcc --show-stats

# Test 3: Monitor distribution

distccmon-text 1  # In separate terminal
make -j16 -C build  # In main terminal
```

---

## Troubleshooting

### distcc Issues

**Problem**: "Connection refused"

```bash

# Check distccd is running on server

ssh server "ps aux | grep distccd"

# Test connectivity

telnet server-ip 3632  # distcc default port
```

**Problem**: "Access denied"

```bash

# Check allowed networks on server
# Edit /etc/default/distcc (Linux) or restart with --allow

distccd --daemon --allow 192.168.1.0/24
```

### ccache Issues

**Problem**: "Cache hit rate low"

```bash

# Check ccache configuration

ccache --show-config

# Increase cache size

ccache --max-size=20G

# Check what's being compiled

ccache --zero-stats
make -C build
ccache --show-stats
```

---

## Summary

✅ **distcc is already installed** on your system
📦 **Install ccache** for maximum benefit (10-100x speedup on rebuilds)
🚀 **Use both together** for best results
⚙️ **Update CMakeLists.txt** to support compilation options
📝 **Document in README** for team members

**Expected Impact**:

- Clean builds: 60s → 15-20s with distcc
- Rebuilds: 60s → 1-2s with ccache
- Incremental: 5-10s → 2-3s with ccache

---

## References

- distcc: <https://distcc.org/>
- ccache: <https://ccache.dev/>
- sccache: <https://github.com/mozilla/sccache>
- CMake Compiler Launchers: <https://cmake.org/cmake/help/latest/variable/CMAKE_LANG_COMPILER_LAUNCHER.html>
