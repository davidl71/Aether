# Git Worktree Setup Guide

This guide explains how to use the `setup_worktree.sh` script to create a new git worktree with all TWS API dependencies pre-built.

## Overview

The `setup_worktree.sh` script automates the complete setup of a new git worktree, including:

1. **Creating the git worktree** - Sets up a new worktree from your specified branch
2. **Building Intel Decimal library** - Compiles `libbid.a` required by TWS API
3. **Building TWS API library** - Compiles `libtwsapi.dylib` with all dependencies
4. **Building main project** - Configures and builds the `ib_box_spread` executable

## Prerequisites

Before running the script, ensure you have:

- **Git** - For worktree management
- **CMake 3.21+** - Build system
- **C++ Compiler** - clang++ or g++ (Xcode Command Line Tools on macOS)
- **Protocol Buffers** - `brew install protobuf`
- **Abseil libraries** - `brew install abseil` (required by modern protobuf)
- **TWS API source** - Extracted to `native/third_party/tws-api/`
- **Intel Decimal library source** - Extracted to `native/third_party/IntelRDFPMathLib20U4/`

## Usage

### Basic Usage

Create a worktree with default name and branch:

```bash
./scripts/setup_worktree.sh
```

This creates a worktree named `worktree-YYYYMMDD-HHMMSS` from the `main` branch.

### Custom Worktree Name

Specify a custom worktree name:

```bash
./scripts/setup_worktree.sh my-feature-branch
```

### Custom Branch

Specify both worktree name and branch:

```bash
./scripts/setup_worktree.sh my-worktree develop
```

### Full Example

```bash
# From the repository root
cd /path/to/ib_box_spread_full_universal

# Create worktree for a new feature
./scripts/setup_worktree.sh feature-tws-integration main

# The script will:
# 1. Create worktree at ../feature-tws-integration
# 2. Build Intel Decimal library
# 3. Build TWS API library
# 4. Build main project
# 5. Print summary with next steps
```

## What the Script Does

### Step 1: Prerequisites Check

The script verifies that all required tools are installed:

- Git
- CMake
- Make or Ninja
- Protocol Buffers library

### Step 2: Create Git Worktree

Creates a new git worktree at `../${WORKTREE_NAME}` from the specified branch.

### Step 3: Build Intel Decimal Library

- Configures CMake build for Intel Decimal Floating-Point Math Library
- Compiles `libbid.a` static library
- Output: `native/third_party/IntelRDFPMathLib20U4/LIBRARY/libbid.a`

### Step 4: Build TWS API Library

- Configures CMake build for TWS API client library
- Links Intel Decimal library and Protocol Buffers
- Compiles `libtwsapi.dylib` shared library
- Creates symlink for debug/release compatibility
- Output: `native/third_party/tws-api/IBJts/source/cppclient/client/build/lib/libtwsapi.dylib`

### Step 5: Build Main Project

- Configures CMake for the main project
- Builds the `ib_box_spread` executable
- Output: `build/bin/ib_box_spread`

## Output

The script provides colored output showing:

- ✅ Success messages for completed steps
- ⚠️ Warnings for non-critical issues
- ❌ Errors for failures (script exits on errors)

## Troubleshooting

### Worktree Already Exists

If the worktree directory already exists:

```bash
# Remove the existing worktree
git worktree remove ../worktree-name

# Or manually remove and try again
rm -rf ../worktree-name
./scripts/setup_worktree.sh worktree-name
```

### Missing Dependencies

If prerequisites are missing, the script will list them:

```bash
# Install missing dependencies
brew install cmake protobuf abseil
```

### TWS API Source Not Found

Ensure TWS API is extracted to the correct location:

```bash
# Check if TWS API exists
ls -la native/third_party/tws-api/IBJts/source/cppclient/client/
```

### Intel Decimal Library Not Found

Ensure Intel Decimal library is extracted:

```bash
# Check if Intel Decimal library exists
ls -la native/third_party/IntelRDFPMathLib20U4/LIBRARY/src/
```

## After Setup

Once the script completes, you can:

1. **Navigate to the worktree:**

   ```bash
   cd ../worktree-name
   ```

2. **Run the application:**

   ```bash
   ./build/bin/ib_box_spread --help
   ```

3. **Configure the application:**

   ```bash
   cp config/config.example.json config/config.json
   # Edit config/config.json with your settings
   ```

4. **Run tests:**

   ```bash
   cd build
   ctest --output-on-failure
   ```

## Removing a Worktree

To remove a worktree when you're done:

```bash
# From the main repository
git worktree remove ../worktree-name

# Or manually
rm -rf ../worktree-name
git worktree prune
```

## Script Options

The script accepts two optional arguments:

1. **Worktree name** (default: `worktree-YYYYMMDD-HHMMSS`)
2. **Branch name** (default: `main`)

## Integration with CI/CD

The script can be used in CI/CD pipelines:

```bash
#!/bin/bash
set -euo pipefail

# Create worktree for CI build
./scripts/setup_worktree.sh ci-build main

# Run tests
cd ../ci-build
cd build
ctest --output-on-failure

# Cleanup
cd ../..
git worktree remove ../ci-build
```

## See Also

- [README.md](../../../README.md) - Main project documentation
- [QUICK_START.md](QUICK_START.md) - Quick start guide
- TWS API build (TWS_BUILD_COMPLETE.md removed; see [API_DOCUMENTATION_INDEX.md](../../API_DOCUMENTATION_INDEX.md))
