# Automated Cross-Platform Setup

**Date**: 2025-01-27
**Scripts**: `scripts/setup_platform_settings.sh` (macOS/Linux), `scripts/setup_platform_settings.ps1` (Windows)

This guide explains how to use the automated setup scripts to configure platform-specific settings.

---

## Quick Start

### macOS / Linux

```bash

# Basic setup (detects platform and generates settings.json.user)

./scripts/setup_platform_settings.sh

# Full setup (also configures CMake)

./scripts/setup_platform_settings.sh --cmake-configure

# Force overwrite existing settings

./scripts/setup_platform_settings.sh --force
```

### Windows (PowerShell)

```powershell

# Basic setup

.\scripts\setup_platform_settings.ps1

# Full setup (also configures CMake)

.\scripts\setup_platform_settings.ps1 -CmakeConfigure

# Force overwrite existing settings

.\scripts\setup_platform_settings.ps1 -Force
```

---

## What the Script Does

### 1. Platform Detection

**macOS/Linux**:

- Detects OS: `uname -s` (Darwin/Linux)
- Detects architecture: `uname -m` (arm64/x86_64)

**Windows**:

- Detects architecture: `$env:PROCESSOR_ARCHITECTURE`

### 2. Compiler Detection

**macOS**:

- Checks `/opt/homebrew/bin/clang++` (Apple Silicon)
- Checks `/usr/local/bin/clang++` (Intel)
- Falls back to `clang++` in PATH

**Linux**:

- Checks `g++` in PATH

**Windows**:

- Searches Visual Studio 2022 installations
- Checks MinGW-w64 locations
- Falls back to `g++` in PATH

### 3. Include Path Detection

**macOS**:

- `/opt/homebrew/include` (Apple Silicon)
- `/usr/local/include` (Intel)

**Linux**:

- `/usr/include`
- `/usr/local/include`
- GCC C++ headers (auto-detects version)

**Windows**:

- Visual Studio MSVC include paths
- MinGW-w64 include paths

### 4. Python Detection

**All Platforms**:

- Checks common installation locations
- Falls back to `python3`/`python` in PATH

### 5. Settings Generation

Creates `.vscode/settings.json.user` with:

- IntelliSense mode (platform-specific)
- Compiler path
- Include paths
- Python interpreter path

### 6. CMake Configuration (Optional)

## MLX Installation (Apple Silicon)

Install via command or Cursor command:

```bash

# Bash script (safe no-op on unsupported platforms)

./scripts/install_mlx.sh
```

Or run the Cursor command:

- `env:install-mlx`

Verification:

```bash
python3 -c "import mlx, mlx_lm; print('OK')"
```

If `--cmake-configure` is used:

- Configures appropriate CMake preset
- Generates `compile_commands.json`
- VS Code will auto-detect settings from `compile_commands.json`

---

## Usage Examples

### First-Time Setup

```bash

# macOS ARM64

./scripts/setup_platform_settings.sh --cmake-configure

# Output:
# [INFO] Detecting platform...
# [SUCCESS] Detected platform: macos-arm64
# [INFO] Detecting C++ compiler...
# [SUCCESS] Found compiler: /usr/bin/clang++
# [INFO] IntelliSense mode: macos-clang-arm64
# [SUCCESS] Generated .vscode/settings.json.user
# [SUCCESS] CMake configured successfully
```

### Update Settings

```bash

# Force regenerate settings

./scripts/setup_platform_settings.sh --force
```

### Manual CMake Configuration

```bash

# Generate settings only

./scripts/setup_platform_settings.sh

# Then configure CMake manually

cmake --preset macos-arm64-debug
```

---

## Generated Settings File

The script generates `.vscode/settings.json.user`:

```json
{
  // Platform-Specific Settings (Auto-Generated)
  "C_Cpp.default.intelliSenseMode": "macos-clang-arm64",
  "C_Cpp.default.compilerPath": "/usr/bin/clang++",
  "C_Cpp.default.includePath": [
    "${workspaceFolder}/native/include",
    "${workspaceFolder}/native/third_party/tws-api/IBJts/source/cppclient/client",
    "/opt/homebrew/include",
    "/usr/local/include"
  ],
  "python.defaultInterpreterPath": "/opt/homebrew/bin/python3"
}
```

---

## Troubleshooting

### Compiler Not Detected

**macOS**:

```bash

# Install Xcode Command Line Tools

xcode-select --install
```

**Linux**:

```bash

# Install build tools

sudo apt-get update
sudo apt-get install build-essential
```

**Windows**:

- Install Visual Studio 2022 (Community is free)
- Or install MinGW-w64

### Include Paths Not Found

The script detects common paths, but you may need to add custom paths manually:

1. Edit `.vscode/settings.json.user`
2. Add paths to `C_Cpp.default.includePath` array

### CMake Configuration Fails

1. **Check dependencies**:

   ```bash
   # macOS
   brew install cmake ninja

   # Linux
   sudo apt-get install cmake ninja-build

   # Windows
   # Install via Visual Studio Installer or Chocolatey
   ```

2. **Check TWS API**:

   ```bash
   ./scripts/check_tws_download.sh
   ```

3. **Manual configuration**:

   ```bash
   cmake --preset <your-platform-preset>
   ```

---

## Integration with Cursor Commands

The script is available as a Cursor command:

- **Command Palette**: `Cmd+Shift+P` → "setup:platform"
- **AI Chat**: "Run the setup:platform command"
- **Full Setup**: "Run setup:platform-full command"

---

## Platform-Specific Notes

### macOS ARM64

- Uses `/opt/homebrew` for Homebrew packages
- IntelliSense mode: `macos-clang-arm64`
- CMake preset: `macos-arm64-debug`

### macOS x86_64

- Uses `/usr/local` for Homebrew packages
- IntelliSense mode: `macos-clang-x64`
- CMake preset: `macos-x86_64-debug`

### Windows

- Searches Visual Studio 2022 installations
- Supports both MSVC and MinGW-w64
- IntelliSense mode: `windows-msvc-x64` or `windows-gcc-x64`
- CMake preset: `windows-x64-debug`

### Linux

- Uses system GCC
- IntelliSense mode: `linux-gcc-x64`
- CMake preset: `linux-x64-debug`

---

## Related Documentation

- [Cross-Platform Setup Guide](research/integration/CROSS_PLATFORM_SETUP.md)
- [Platform-Specific Settings](PLATFORM_SPECIFIC_SETTINGS.md)
- [CMake Presets Guide](external/CMake_PRESETS_GUIDE.md)
