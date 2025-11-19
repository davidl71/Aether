# CMake Extension Enhancements

**Date**: 2025-01-27
**Purpose**: Enhanced CMake integration with VS Code/Cursor extensions for better development experience

---

## Overview

This document describes the enhanced CMake integration configured for this project, leveraging VS Code/Cursor extensions to provide a superior CMake development experience.

---

## Installed Extensions

### 1. CMake Tools (`ms-vscode.cmake-tools`)
**Status**: ✅ Already installed and configured

The official Microsoft CMake Tools extension provides:
- Automatic CMake configuration detection
- Build target management
- Debugging integration
- Test explorer integration
- Status bar quick actions

### 2. CMake Language Support (`twxs.cmake`)
**Status**: ✅ Newly added

Enhanced CMake language support with:
- Syntax highlighting for `CMakeLists.txt` and `.cmake` files
- IntelliSense and code completion
- Formatting support
- Better error detection

---

## Enhanced Configuration

### CMake Presets Integration

The project now fully leverages CMake Presets (CMake 3.19+):

```json
"cmake.useCMakePresets": "always",
"cmake.presets.default": "macos-x86_64-debug"
```

**Benefits**:
- Consistent builds across team members
- Easy switching between Debug/Release configurations
- Platform-specific presets (macOS, Windows, Linux)
- Architecture-specific builds (x86_64, ARM64, Universal)

**Available Presets**:
- `macos-x86_64-debug` (default)
- `macos-x86_64-release`
- `macos-arm64-debug`
- `macos-arm64-release`
- `windows-x64-debug`
- `windows-x64-release`
- `linux-x64-debug`
- `linux-x64-release`

### Parallel Build Configuration

```json
"cmake.parallelJobs": 0  // 0 = use all available CPU cores
```

Automatically uses all available CPU cores for maximum build speed.

### Test Explorer Integration

```json
"cmake.testExplorerIntegrationEnabled": true
```

**Benefits**:
- Visual test results in Test Explorer panel
- Run individual tests from the UI
- See test status at a glance
- Integrated with CTest

### Debug Configuration

Pre-configured debug settings:

```json
"cmake.debugConfig": {
  "name": "Debug",
  "type": "cppdbg",
  "MIMode": "lldb",  // macOS
  // ... automatic program path detection
}
```

**Usage**:
1. Set breakpoints in your C++ code
2. Press `F5` or use Command Palette: "CMake: Debug"
3. Extension automatically finds the correct executable path

### Status Bar Integration

Enhanced status bar shows:
- Current CMake preset
- Build status
- Configure status
- Quick actions (Configure, Build, Debug, Launch)

### Logging Configuration

```json
"cmake.loggingLevel": "info"
```

**Available levels**:
- `trace` - Most verbose (for debugging CMake issues)
- `debug` - Detailed information
- `info` - Standard information (default)
- `warning` - Warnings and errors only
- `error` - Errors only

---

## Usage Guide

### Quick Start

1. **Open the project** in Cursor/VS Code
2. **Install recommended extensions** (prompted automatically)
3. **Select CMake preset**:
   - Click on preset name in status bar (bottom)
   - Or: `Cmd+Shift+P` → "CMake: Select Configure Preset"
4. **Configure project**:
   - Status bar: Click "Configure"
   - Or: `Cmd+Shift+P` → "CMake: Configure"
5. **Build project**:
   - `Cmd+Shift+B` (default build task)
   - Or: Status bar → "Build"
   - Or: `Cmd+Shift+P` → "CMake: Build"

### Switching Presets

**Method 1: Status Bar**
1. Click on preset name in status bar
2. Select new preset from dropdown

**Method 2: Command Palette**
1. `Cmd+Shift+P`
2. "CMake: Select Configure Preset"
3. Choose preset

**Method 3: Settings**
Update `cmake.presets.default` in `.vscode/settings.json`

### Building Specific Targets

1. `Cmd+Shift+P` → "CMake: Set Default Target"
2. Select target (e.g., `ib_box_spread`, `box_spread_calc_test`)
3. Build with `Cmd+Shift+B`

### Running Tests

**Method 1: Test Explorer**
1. Open Test Explorer panel (`Cmd+Shift+T`)
2. See all CTest tests
3. Click play button to run individual tests

**Method 2: Command Palette**
1. `Cmd+Shift+P` → "CMake: Run Tests"
2. Or use task: "Run Tests"

**Method 3: Terminal**
```bash
ctest --preset macos-x86_64-debug --output-on-failure
```

### Debugging

1. **Set breakpoints** in your C++ code
2. **Select launch target**:
   - Status bar → "Select Launch Target"
   - Or: `Cmd+Shift+P` → "CMake: Select Launch Target"
3. **Start debugging**:
   - Press `F5`
   - Or: Status bar → "Debug"
   - Or: `Cmd+Shift+P` → "CMake: Debug"

### CMake Script Editing

With `twxs.cmake` extension:
- **Syntax highlighting** for CMakeLists.txt
- **IntelliSense** for CMake commands
- **Format on save** (configured)
- **Error detection** for invalid CMake syntax

---

## Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Build | `Cmd+Shift+B` |
| Configure | Status bar or Command Palette |
| Debug | `F5` |
| Select Preset | Status bar or `Cmd+Shift+P` → "CMake: Select Configure Preset" |
| Select Target | `Cmd+Shift+P` → "CMake: Set Default Target" |
| Run Tests | Test Explorer or `Cmd+Shift+P` → "CMake: Run Tests" |

---

## Troubleshooting

### CMake Not Detected

1. **Check CMake installation**:
   ```bash
   cmake --version  # Should be 3.23+
   ```

2. **Verify CMake Tools extension**:
   - Extensions panel → Search "CMake Tools"
   - Ensure it's installed and enabled

3. **Reload window**:
   - `Cmd+Shift+P` → "Developer: Reload Window"

### Preset Not Found

1. **Verify CMakePresets.json exists**:
   ```bash
   ls CMakePresets.json
   ```

2. **Check preset name**:
   - Ensure preset name matches exactly (case-sensitive)
   - Check `CMakePresets.json` for available presets

3. **Reconfigure**:
   - `Cmd+Shift+P` → "CMake: Delete Cache and Reconfigure"

### Build Errors

1. **Check build output**:
   - View → Output → Select "CMake" or "Tasks"

2. **Verify dependencies**:
   - Intel Decimal library built?
   - TWS API library built?
   - Protocol Buffers installed?

3. **Clean and rebuild**:
   - `Cmd+Shift+P` → "CMake: Clean"
   - Then rebuild

### IntelliSense Not Working

1. **Check compile_commands.json**:
   ```bash
   ls build/compile_commands.json
   ```

2. **Regenerate**:
   - Ensure `CMAKE_EXPORT_COMPILE_COMMANDS=ON` in preset
   - Reconfigure CMake

3. **Reload C++ extension**:
   - `Cmd+Shift+P` → "C/C++: Reset IntelliSense Database"

### Test Explorer Empty

1. **Verify CTest integration**:
   - Check `CMakePresets.json` has test presets
   - Ensure tests are added with `add_test()` in CMakeLists.txt

2. **Reconfigure**:
   - `Cmd+Shift+P` → "CMake: Configure"

3. **Check CTest output**:
   ```bash
   ctest --preset macos-x86_64-debug --verbose
   ```

---

## Advanced Features

### Custom Build Variants

Configure default build variants:

```json
"cmake.defaultVariants": {
  "buildType": {
    "default": "debug",
    "description": "The build type to use"
  }
}
```

### Custom Debug Configuration

Override debug settings in `.vscode/launch.json`:

```json
{
  "name": "Custom Debug",
  "type": "cppdbg",
  "request": "launch",
  "program": "${command:cmake.launchTargetPath}",
  "args": ["--config", "config.json"],
  "cwd": "${workspaceFolder}",
  "MIMode": "lldb"
}
```

### CMake Script Formatting

CMake files are automatically formatted on save with:
- 2-space indentation
- Consistent spacing
- Proper line breaks

Configure in `.vscode/settings.json`:
```json
"[cmake]": {
  "editor.defaultFormatter": "twxs.cmake",
  "editor.formatOnSave": true,
  "editor.tabSize": 2
}
```

---

## Best Practices

1. **Always use presets** - Don't manually configure CMake
2. **Select preset before building** - Ensures correct configuration
3. **Use Test Explorer** - Visual test management is easier
4. **Leverage status bar** - Quick access to common actions
5. **Check build output** - CMake Tools provides detailed logs
6. **Format CMake files** - Keep CMakeLists.txt clean and readable

---

## Related Documentation

- [CMake Presets Documentation](https://cmake.org/cmake/help/latest/manual/cmake-presets.7.html)
- [CMake Tools Extension Docs](https://github.com/microsoft/vscode-cmake-tools)
- [CMake Language Support](https://marketplace.visualstudio.com/items?itemName=twxs.cmake)
- [Project CMakePresets.json](../CMakePresets.json)
- [Cursor Setup Guide](./CURSOR_SETUP.md)

---

## Summary

The enhanced CMake integration provides:

✅ **Automatic preset detection**
✅ **Visual build management**
✅ **Integrated debugging**
✅ **Test explorer integration**
✅ **Enhanced CMake script editing**
✅ **Parallel builds**
✅ **Status bar quick actions**

All configured and ready to use!
