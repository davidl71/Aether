# Cross-Platform Development Setup

**Date**: 2025-01-27
**Supported Platforms**: macOS (ARM64/x86_64), Windows, Linux (Ubuntu)

This guide explains how the project is configured for cross-platform development.

---

## Platform-Specific Settings

### VS Code Settings (`.vscode/settings.json`)

The project uses **platform-specific settings** that automatically adapt to your OS:

```json
{
  // Platform-specific C++ IntelliSense (auto-detected)
  "C_Cpp.default.intelliSenseMode": {
    "macos": "macos-clang-arm64",  // or "macos-clang-x64" for Intel Macs
    "windows": "windows-msvc-x64",
    "linux": "linux-gcc-x64"
  },

  // Platform-specific compiler paths (auto-detected)
  "C_Cpp.default.compilerPath": {
    "macos": "/usr/bin/clang++",
    "windows": "C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC/14.xx.xxxxx/bin/Hostx64/x64/cl.exe",
    "linux": "/usr/bin/g++"
  },

  // Platform-specific include paths
  "C_Cpp.default.includePath": {
    "macos": [
      "${workspaceFolder}/native/include",
      "/usr/local/include",
      "/opt/homebrew/include"  // Apple Silicon
    ],
    "windows": [
      "${workspaceFolder}/native/include",
      "C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC/14.xx.xxxxx/include"
    ],
    "linux": [
      "${workspaceFolder}/native/include",
      "/usr/include",
      "/usr/local/include"
    ]
  },

  // Platform-specific Python paths
  "python.defaultInterpreterPath": {
    "macos": "/usr/local/bin/python3",
    "windows": "C:/Python3xx/python.exe",
    "linux": "/usr/bin/python3"
  },

  // Platform-specific terminal environment
  "terminal.integrated.env.osx": {
    "PYTHONPATH": "${workspaceFolder}/python"
  },
  "terminal.integrated.env.windows": {
    "PYTHONPATH": "${workspaceFolder}/python"
  },
  "terminal.integrated.env.linux": {
    "PYTHONPATH": "${workspaceFolder}/python"
  }
}
```

**Note**: VS Code doesn't support JSON object values for platform-specific settings. Instead, we use separate settings blocks or let CMake/IntelliSense auto-detect.

---

## CMake Presets

### macOS Presets

```bash
# ARM64 (Apple Silicon)
cmake --preset macos-arm64-debug
cmake --preset macos-arm64-release

# x86_64 (Intel)
cmake --preset macos-x86_64-debug
cmake --preset macos-x86_64-release
```

### Windows Presets

```bash
# x64 Debug
cmake --preset windows-x64-debug

# x64 Release
cmake --preset windows-x64-release
```

### Linux Presets

```bash
# x64 Debug
cmake --preset linux-x64-debug

# x64 Release
cmake --preset linux-x64-release
```

---

## Platform-Specific Considerations

### macOS

**Compiler**: Clang (Xcode Command Line Tools)
**Package Manager**: Homebrew
**Include Paths**:

- `/usr/local/include` (Intel Macs, Homebrew)
- `/opt/homebrew/include` (Apple Silicon, Homebrew)

**IntelliSense Mode**: `macos-clang-arm64` or `macos-clang-x64`

### Windows

**Compiler**: MSVC (Visual Studio) or MinGW-w64
**Package Manager**: vcpkg or Chocolatey
**Include Paths**:

- Visual Studio: `C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC/...`
- MinGW: `C:/mingw64/include`

**IntelliSense Mode**: `windows-msvc-x64` or `windows-gcc-x64`

**Note**: TWS API on Windows may require different library paths (`.dll` vs `.dylib`/`.so`)

### Linux (Ubuntu)

**Compiler**: GCC
**Package Manager**: apt
**Include Paths**:

- `/usr/include`
- `/usr/local/include`

**IntelliSense Mode**: `linux-gcc-x64`

**Dependencies**:

```bash
sudo apt-get update
sudo apt-get install build-essential cmake ninja-build
sudo apt-get install libprotobuf-dev protobuf-compiler
```

---

## Auto-Detection Strategy

### 1. CMake Auto-Detection

CMake automatically detects:

- Compiler (clang, gcc, msvc)
- Architecture (x86_64, arm64)
- Platform (macOS, Windows, Linux)

### 2. VS Code IntelliSense

VS Code C++ extension can auto-detect:

- Compiler path
- IntelliSense mode
- Include paths (from `compile_commands.json`)

**Best Practice**: Let CMake generate `compile_commands.json` and VS Code will use it.

### 3. Scripts

Scripts use `uname` or environment variables to detect platform:

```bash
# Detect architecture
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
  # Apple Silicon
elif [ "$ARCH" = "x86_64" ]; then
  # Intel/AMD64
fi

# Detect OS
OS=$(uname -s)
if [ "$OS" = "Darwin" ]; then
  # macOS
elif [ "$OS" = "Linux" ]; then
  # Linux
fi
```

---

## Configuration Files

### Platform-Specific User Settings

Create `.vscode/settings.json.user` (gitignored) for platform-specific overrides:

**macOS ARM64**:

```json
{
  "C_Cpp.default.intelliSenseMode": "macos-clang-arm64",
  "C_Cpp.default.compilerPath": "/usr/bin/clang++",
  "C_Cpp.default.includePath": [
    "${workspaceFolder}/native/include",
    "/opt/homebrew/include"
  ]
}
```

**Windows**:

```json
{
  "C_Cpp.default.intelliSenseMode": "windows-msvc-x64",
  "C_Cpp.default.compilerPath": "C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC/14.xx.xxxxx/bin/Hostx64/x64/cl.exe",
  "python.defaultInterpreterPath": "C:/Python3xx/python.exe"
}
```

**Linux**:

```json
{
  "C_Cpp.default.intelliSenseMode": "linux-gcc-x64",
  "C_Cpp.default.compilerPath": "/usr/bin/g++",
  "C_Cpp.default.includePath": [
    "${workspaceFolder}/native/include",
    "/usr/include",
    "/usr/local/include"
  ]
}
```

---

## Troubleshooting

### IntelliSense Not Working

1. **Check `compile_commands.json`**:

   ```bash
   ls build/*/compile_commands.json
   ```

2. **Regenerate compile commands**:

   ```bash
   cmake --preset <your-preset>
   ```

3. **Reload VS Code**: `Cmd+Shift+P` → "Developer: Reload Window"

### Wrong Compiler Detected

1. **Check CMake preset** matches your platform
2. **Verify compiler path** in settings
3. **Check PATH** environment variable

### Include Paths Not Found

1. **Check platform-specific include paths** in settings
2. **Verify dependencies installed** (Homebrew, vcpkg, apt)
3. **Check CMake output** for detected paths

---

## Best Practices

1. **Use CMake Presets**: Always use platform-specific presets
2. **Let CMake Detect**: Don't hardcode paths if CMake can detect them
3. **Use `compile_commands.json`**: VS Code will use it automatically
4. **Platform-Specific User Settings**: Override in `.vscode/settings.json.user`
5. **Document Platform Differences**: Note any platform-specific requirements

---

## Related Documentation

- [CMake Presets Guide](../../external/CMake_PRESETS_GUIDE.md)
- [Windows Setup Guide](WINDOWS_SETUP_GUIDE.md)
- [User vs Workspace Settings](../../USER_VS_WORKSPACE_SETTINGS.md)
