# Cursor IDE Configuration

This project includes comprehensive Cursor IDE configuration to enhance your development experience.

## Configuration Files

### `.cursorrules`
Main AI assistant rules file that guides Cursor's AI when helping with this codebase. It includes:
- Code style guidelines (C++20, 2-space indentation, Allman braces)
- Build system conventions
- Testing practices
- Security guidelines
- Project structure information

### `.vscode/settings.json`
Workspace settings for Cursor/VS Code:
- **C++ Configuration**: IntelliSense, include paths, compiler settings
- **Editor Settings**: 2-space indentation, 100-character ruler, format on save
- **CMake Integration**: Auto-configuration, build directory settings
- **File Exclusions**: Hides build artifacts from file explorer
- **Python/Rust/TypeScript**: Language-specific settings

### `.vscode/tasks.json`
Pre-configured build tasks:
- **CMake: Configure (Debug)** - Configure with debug preset
- **CMake: Build** - Build the project (default build task)
- **CMake: Build (Release)** - Build release version
- **CMake: Clean** - Clean build artifacts
- **Run Tests** - Execute test suite
- **Setup Worktree** - Run worktree setup script
- **Build Universal** - Run universal build script
- **Run Linters** - Execute linting checks
- **Build Intel Decimal Library** - Build dependency
- **Build TWS API Library** - Build dependency

### `.vscode/launch.json`
Debug configurations:
- **Debug ib_box_spread** - Debug main executable with dry-run
- **Debug ib_box_spread (with config)** - Debug with config file
- **Run Tests** - Debug test suite
- **Attach to Process** - Attach debugger to running process

### `.vscode/extensions.json`
Recommended extensions:
- **C++**: C/C++ extension pack, CMake Tools
- **Python**: Python, Pylance, Black formatter
- **Rust**: rust-analyzer (for agents/backend)
- **Go**: Go extension (for tui/)
- **TypeScript**: ESLint, Prettier (for web/)
- **General**: EditorConfig, GitLens, Markdown tools

### `.editorconfig`
Editor-agnostic configuration for consistent formatting across editors.

## Quick Start

1. **Open the project in Cursor**
   ```bash
   cursor /path/to/ib_box_spread_full_universal
   ```

2. **Install recommended extensions**
   - Cursor will prompt you to install recommended extensions
   - Or use: `Cmd+Shift+P` → "Extensions: Show Recommended Extensions"

3. **Configure CMake** (if needed)
   - Cursor should auto-detect CMakePresets.json
   - Or manually configure: `Cmd+Shift+P` → "CMake: Configure"

4. **Build the project**
   - Use `Cmd+Shift+B` to run default build task
   - Or `Cmd+Shift+P` → "Tasks: Run Task" → "CMake: Build"

5. **Debug**
   - Set breakpoints in your code
   - Press `F5` to start debugging
   - Or use `Cmd+Shift+P` → "Debug: Start Debugging"

## Key Features

### IntelliSense
- Full C++20 IntelliSense with clang
- Auto-completion for TWS API headers
- Go-to-definition support
- Symbol navigation

### Build Integration
- One-click build with `Cmd+Shift+B`
- Build errors shown in Problems panel
- Click errors to jump to source

### Debugging
- Full LLDB integration
- Breakpoints, watch variables, call stack
- Step through code with F10/F11
- Debug console for expressions

### Code Formatting
- Format on save enabled
- Consistent 2-space indentation
- 100-character line length guide
- Auto-trim trailing whitespace

### File Navigation
- Build artifacts hidden from explorer
- Quick file search with `Cmd+P`
- Symbol search with `Cmd+Shift+O`
- Workspace symbol search with `Cmd+T`

## Customization

### User-Specific Settings
Create `.vscode/settings.json.user` (gitignored) for personal preferences:
```json
{
  "editor.fontSize": 14,
  "editor.fontFamily": "Fira Code"
}
```

### Custom Tasks
Add project-specific tasks to `.vscode/tasks.json`:
```json
{
  "label": "My Custom Task",
  "type": "shell",
  "command": "echo 'Hello'",
  "group": "build"
}
```

### Custom Launch Configurations
Add debug configurations to `.vscode/launch.json`:
```json
{
  "name": "My Custom Debug",
  "type": "cppdbg",
  "request": "launch",
  "program": "${workspaceFolder}/build/bin/my_program"
}
```

## Troubleshooting

### IntelliSense Not Working
1. Check that `compile_commands.json` exists in build directory
2. Run CMake configure: `Cmd+Shift+P` → "CMake: Configure"
3. Reload window: `Cmd+Shift+P` → "Developer: Reload Window"

### Build Fails
1. Check CMake preset is configured: `Cmd+Shift+P` → "CMake: Configure"
2. Verify dependencies are built (Intel Decimal, TWS API)
3. Check build output: `View` → `Output` → Select "CMake" or "Tasks"

### Debugger Not Attaching
1. Ensure binary is built with debug symbols (`CMAKE_BUILD_TYPE=Debug`)
2. Check that program path in launch.json is correct
3. Verify LLDB is installed: `which lldb`

### Extensions Not Installing
1. Check internet connection
2. Try installing manually from Extensions view
3. Check Cursor/VS Code version compatibility

## Tips

### Keyboard Shortcuts
- `Cmd+Shift+B` - Build
- `F5` - Start debugging
- `F9` - Toggle breakpoint
- `F10` - Step over
- `F11` - Step into
- `Shift+F11` - Step out
- `Cmd+P` - Quick file open
- `Cmd+Shift+O` - Go to symbol in file
- `Cmd+T` - Go to symbol in workspace
- `Cmd+Shift+F` - Search in files

### Productivity Tips
1. Use `Cmd+Shift+P` for command palette - faster than menus
2. Enable "Format on Save" for consistent code style
3. Use Problems panel (`Cmd+Shift+M`) to see all errors/warnings
4. Use multi-cursor editing (`Cmd+Option+Up/Down`) for bulk edits
5. Use breadcrumbs (`View` → `Show Breadcrumbs`) for navigation

## See Also

- [CURSOR_AI_TUTORIAL.md](CURSOR_AI_TUTORIAL.md) - Cursor AI tutorial and best practices
- [CURSOR_RECOMMENDATIONS.md](CURSOR_RECOMMENDATIONS.md) - Cursor optimization recommendations
- [CURSOR_DOCS_USAGE.md](CURSOR_DOCS_USAGE.md) - Using @docs in Cursor
- [README.md](../README.md) - Main project documentation
- [WORKTREE_SETUP.md](WORKTREE_SETUP.md) - Worktree setup guide
- [QUICK_START.md](QUICK_START.md) - Quick start guide
