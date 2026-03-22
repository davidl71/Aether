# Platform-Specific Settings Reference

**Date**: 2025-01-27
**Quick Reference**: Copy-paste settings for each platform

---

## Quick Setup Guide

1. **Copy** `.vscode/settings.json.user.example` to `.vscode/settings.json.user`
2. **Uncomment** the section for your platform
3. **Customize** paths if needed
4. **Save** - VS Code will automatically use these settings

---

## macOS ARM64 (Apple Silicon)

```json
{
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

**CMake Preset**: `macos-arm64-debug` or `macos-arm64-release`

---

## macOS x86_64 (Intel)

```json
{
  "C_Cpp.default.intelliSenseMode": "macos-clang-x64",
  "C_Cpp.default.compilerPath": "/usr/bin/clang++",
  "C_Cpp.default.includePath": [
    "${workspaceFolder}/native/include",
    "${workspaceFolder}/native/third_party/tws-api/IBJts/source/cppclient/client",
    "/usr/local/include"
  ],
  "python.defaultInterpreterPath": "/usr/local/bin/python3"
}
```

**CMake Preset**: `macos-x86_64-debug` or `macos-x86_64-release`

---

## Windows

```json
{
  "C_Cpp.default.intelliSenseMode": "windows-msvc-x64",
  "C_Cpp.default.compilerPath": "C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC/14.xx.xxxxx/bin/Hostx64/x64/cl.exe",
  "C_Cpp.default.includePath": [
    "${workspaceFolder}/native/include",
    "${workspaceFolder}/native/third_party/tws-api/IBJts/source/cppclient/client",
    "C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC/14.xx.xxxxx/include",
    "C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC/14.xx.xxxxx/atlmfc/include"
  ],
  "python.defaultInterpreterPath": "C:/Python3xx/python.exe"
}
```

**Note**: Replace `14.xx.xxxxx` with your actual MSVC version.

**CMake Preset**: `windows-x64-debug` or `windows-x64-release`

**Alternative (MinGW-w64)**:

```json
{
  "C_Cpp.default.intelliSenseMode": "windows-gcc-x64",
  "C_Cpp.default.compilerPath": "C:/mingw64/bin/g++.exe",
  "C_Cpp.default.includePath": [
    "${workspaceFolder}/native/include",
    "${workspaceFolder}/native/third_party/tws-api/IBJts/source/cppclient/client",
    "C:/mingw64/include"
  ]
}
```

---

## Linux (Ubuntu)

```json
{
  "C_Cpp.default.intelliSenseMode": "linux-gcc-x64",
  "C_Cpp.default.compilerPath": "/usr/bin/g++",
  "C_Cpp.default.includePath": [
    "${workspaceFolder}/native/include",
    "${workspaceFolder}/native/third_party/tws-api/IBJts/source/cppclient/client",
    "/usr/include",
    "/usr/local/include",
    "/usr/include/c++/11"
  ],
  "python.defaultInterpreterPath": "/usr/bin/python3"
}
```

**CMake Preset**: `linux-x64-debug` or `linux-x64-release`

**Dependencies**:

```bash
sudo apt-get update
sudo apt-get install build-essential cmake ninja-build
sudo apt-get install libprotobuf-dev protobuf-compiler
sudo apt-get install python3 python3-pip
```

---

## Auto-Detection (Recommended)

**Best Practice**: Let VS Code auto-detect from `compile_commands.json`:

1. **Configure CMake**:

   ```bash
   cmake --preset <your-platform-preset>
   ```

2. **CMake generates** `compile_commands.json` automatically (if `CMAKE_EXPORT_COMPILE_COMMANDS=ON`)

3. **VS Code C++ extension** will automatically:
   - Detect compiler path
   - Detect IntelliSense mode
   - Detect include paths

4. **Only override** in `settings.json.user` if auto-detection fails

---

## Finding Your Compiler Path

### macOS

```bash
which clang++

# Output: /usr/bin/clang++ or /opt/homebrew/bin/clang++
```

### Windows (MSVC)

```bash
where cl

# Or check: C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC/
```

### Linux

```bash
which g++

# Output: /usr/bin/g++
```

---

## Finding Include Paths

### macOS

```bash

# Homebrew (Apple Silicon)

ls /opt/homebrew/include

# Homebrew (Intel)

ls /usr/local/include

# System

ls /usr/include
```

### Windows

```bash

# Visual Studio

dir "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\*\include"
```

### Linux

```bash
ls /usr/include
ls /usr/local/include
ls /usr/include/c++/11  # GCC version-specific
```

---

## Troubleshooting

### IntelliSense Not Working

1. **Check `compile_commands.json` exists**:

   ```bash
   ls build/*/compile_commands.json
   ```

2. **Regenerate**:

   ```bash
   cmake --preset <your-preset>
   ```

3. **Reload VS Code**: `Cmd+Shift+P` → "Developer: Reload Window"

### Wrong Compiler Detected

1. **Check CMake preset** matches your platform
2. **Verify compiler path** in `settings.json.user`
3. **Check PATH** environment variable

### Include Paths Not Found

1. **Check platform-specific paths** in `settings.json.user`
2. **Verify dependencies installed**
3. **Check CMake output** for detected paths

---

## Related Documentation

- [Cross-Platform Setup Guide]( CROSS_PLATFORM_SETUP.md)
- [CMake Presets Guide](external/CMake_PRESETS_GUIDE.md)
- [User vs Workspace Settings](USER_VS_WORKSPACE_SETTINGS.md)
