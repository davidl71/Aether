# CMake Presets Guide

**Source**: Project CMakePresets.json configuration
**Last Updated**: 2025-01-27

This guide explains how to use CMake presets in this project for building across different architectures and configurations.

---

## Available Presets

### Configure Presets

#### macOS x86_64 (Intel Macs)

```bash
# Debug build
cmake --preset macos-x86_64-debug

# Release build
cmake --preset macos-x86_64-release
```

**Output**: `build/macos-x86_64-debug/` or `build/macos-x86_64-release/`

#### macOS ARM64 (Apple Silicon)

```bash
# Debug build
cmake --preset macos-arm64-debug

# Release build
cmake --preset macos-arm64-release
```

**Output**: `build/macos-arm64-debug/` or `build/macos-arm64-release/`

#### Universal (Deprecated)

```bash
# Debug build (DEPRECATED - use macos-x86_64-debug)
cmake --preset macos-universal-debug

# Release build (DEPRECATED - use macos-x86_64-release)
cmake --preset macos-universal-release
```

**Note**: Universal presets are deprecated. Use architecture-specific presets instead.

---

## Build Presets

After configuring, use build presets:

```bash
# Build debug
cmake --build --preset macos-x86_64-debug

# Build release
cmake --build --preset macos-x86_64-release

# Build ARM64 debug
cmake --build --preset macos-arm64-debug

# Build ARM64 release
cmake --build --preset macos-arm64-release
```

---

## Test Presets

Run tests with test presets:

```bash
# Run tests (debug)
ctest --preset macos-x86_64-debug --output-on-failure

# Run tests (release)
ctest --preset macos-x86_64-release --output-on-failure

# Run tests (ARM64 debug)
ctest --preset macos-arm64-debug --output-on-failure

# Run tests (ARM64 release)
ctest --preset macos-arm64-release --output-on-failure
```

---

## Common Workflows

### 1. Configure and Build (Debug)

```bash
# Configure
cmake --preset macos-arm64-debug

# Build
cmake --build --preset macos-arm64-debug

# Run tests
ctest --preset macos-arm64-debug --output-on-failure
```

### 2. Configure and Build (Release)

```bash
# Configure
cmake --preset macos-arm64-release

# Build
cmake --build --preset macos-arm64-release

# Run tests
ctest --preset macos-arm64-release --output-on-failure
```

### 3. Clean Build

```bash
# Remove build directory
rm -rf build/macos-arm64-debug

# Reconfigure
cmake --preset macos-arm64-debug

# Rebuild
cmake --build --preset macos-arm64-debug
```

---

## Preset Configuration Details

### Debug Presets

**Settings**:

- `CMAKE_BUILD_TYPE`: `Debug`
- `CMAKE_EXPORT_COMPILE_COMMANDS`: `ON` (for IntelliSense)
- `CMAKE_COLOR_DIAGNOSTICS`: `ON` (colored output)

**Use For**:

- Development
- Debugging
- Testing
- IntelliSense support

### Release Presets

**Settings**:

- `CMAKE_BUILD_TYPE`: `Release`
- `CMAKE_EXPORT_COMPILE_COMMANDS`: `ON`
- `CMAKE_COLOR_DIAGNOSTICS`: `ON`

**Use For**:

- Production builds
- Performance testing
- Distribution

---

## Architecture-Specific Settings

### x86_64 (Intel)

```json
"CMAKE_OSX_ARCHITECTURES": "x86_64",
"CMAKE_OSX_DEPLOYMENT_TARGET": "15.0"
```

### ARM64 (Apple Silicon)

```json
"CMAKE_OSX_ARCHITECTURES": "arm64",
"CMAKE_OSX_DEPLOYMENT_TARGET": "15.0"
```

---

## VS Code Integration

### Using CMake Tools Extension

1. **Configure**: Press `Cmd+Shift+P` → "CMake: Configure"
2. **Select Preset**: Choose from available presets
3. **Build**: Press `Cmd+Shift+B` or use "CMake: Build"
4. **Debug**: Press `F5` (uses launch.json configuration)

### Tasks Integration

Pre-configured tasks in `.vscode/tasks.json`:

- **CMake: Configure (Debug)** - Uses `macos-universal-debug` preset
- **CMake: Build** - Default build task
- **CMake: Build (Release)** - Release build
- **Run Tests** - Runs ctest with output-on-failure

---

## Environment Variables

Some presets may require environment variables:

```bash
# TWS API path (if not using default)
export IBAPI_INCLUDE_DIR=~/IBJts/source/cppclient
export IBAPI_LIB=~/IBJts/source/cppclient/libTwsApiCpp.dylib

# Intel Decimal Library path
export INTEL_DECIMAL_LIB=~/path/to/libbid.a
```

---

## Troubleshooting

### Preset Not Found

```bash
# List available presets
cmake --list-presets

# Verify CMakePresets.json exists
ls -la CMakePresets.json
```

### Wrong Architecture

```bash
# Check current architecture
uname -m

# Use appropriate preset
# For Apple Silicon (arm64): macos-arm64-debug
# For Intel (x86_64): macos-x86_64-debug
```

### Build Directory Mismatch

```bash
# Clean and reconfigure
rm -rf build/*
cmake --preset macos-arm64-debug
```

---

## Best Practices

1. **Use architecture-specific presets** instead of universal
2. **Configure once** per preset, then build multiple times
3. **Use debug preset** for development and testing
4. **Use release preset** for production builds
5. **Run tests** after each build to verify correctness
6. **Clean build directory** when switching presets

---

## References

- [CMake Presets Documentation](https://cmake.org/cmake/help/latest/manual/cmake-presets.7.html)
- [Project CMakePresets.json](../../CMakePresets.json)
- [Build System Documentation](../research/integration/DISTRIBUTED_COMPILATION.md)
