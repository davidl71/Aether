# iTerm2 Integration Guide

This project supports executing VS Code tasks in iTerm2 instead of the integrated terminal, providing better terminal features, tmux integration, and performance for long-running tasks.

## Overview

iTerm2 integration allows you to:

- **Better Terminal Experience**: Full iTerm2 features (split panes, better rendering, etc.)
- **tmux Integration**: Native tmux support with iTerm2 Control Mode
- **Performance**: Better performance for long-running builds and tests
- **Service Monitoring**: Better visibility for service logs and status
- **Terminal Space**: More screen real estate for complex output

## Prerequisites

1. **iTerm2 Installed**: Download from [iTerm2 website](https://iterm2.com/)
2. **Extension Installed**: The `execute-in-iterm2` extension (or similar) should be installed
   - VS Code will prompt you to install recommended extensions
   - Or install manually: `Cmd+Shift+X` → Search "execute in iterm2"

## Configuration

### Extension Setup

The extension is already configured in `.vscode/extensions.json`:

```json
"tlehman.execute-in-iterm2"
```

### Task Configuration

Tasks are configured in `.vscode/tasks.json` with both standard and iTerm2 variants:

- **Standard Tasks**: Run in VS Code integrated terminal (default)
- **iTerm2 Tasks**: Run in iTerm2 (suffixed with "(iTerm2)")

## Available iTerm2 Tasks

### Build Tasks

- **CMake: Build (iTerm2)**: Build project in iTerm2
- **Build Universal (iTerm2)**: Build universal binary in iTerm2

### Test Tasks

- **Run Tests (iTerm2)**: Run test suite in iTerm2

### Service Tasks

- **Launch PWA Services (iTerm2)**: Launch all PWA services in iTerm2 with tmux

## Usage

### Running Tasks in iTerm2

1. **Command Palette**:
   - `Cmd+Shift+P` → "Tasks: Run Task"
   - Select task with "(iTerm2)" suffix

2. **Keyboard Shortcuts**:
   - Configure custom keybindings in `.vscode/keybindings.json` (if created)

3. **Task Runner**:
   - `Cmd+Shift+B` for default build task (standard terminal)
   - Use Command Palette for iTerm2 variants

### Example: Building in iTerm2

```bash
# Via Command Palette
Cmd+Shift+P → "Tasks: Run Task" → "CMake: Build (iTerm2)"
```

This will:

1. Open a new iTerm2 tab
2. Change to project directory
3. Execute the build command
4. Keep the tab open for viewing output

## Integration with Launch Scripts

The project's launch scripts (`web/scripts/launch-all-pwa-services.sh`) automatically detect iTerm2 and use native tmux integration:

```bash
# Script detects iTerm2 and uses Control Mode
if [ -n "${ITERM_PROFILE:-}" ] || [ -n "${ITERM_SESSION_ID:-}" ]; then
  echo "Detected iTerm2 - using native tmux integration"
  tmux -CC new-session -d -s pwa-services
fi
```

## Benefits for This Project

### Long-Running Builds

Universal binary builds can take 10+ minutes. Running in iTerm2 provides:

- Better output rendering
- Ability to detach/reattach to tmux session
- More terminal space for verbose output

### Service Monitoring

PWA services run continuously. iTerm2 integration provides:

- Native tmux support for service management
- Better log viewing with split panes
- Easy session management

### Test Execution

ShellSpec tests and C++ tests benefit from:

- Better output formatting
- Ability to scroll through long test output
- Terminal history preservation

## Configuration Options

### Custom iTerm2 Profile

You can configure tasks to use a specific iTerm2 profile:

```json
{
  "label": "My Task (iTerm2)",
  "type": "shell",
  "options": {
    "shell": {
      "executable": "/usr/bin/osascript",
      "args": [
        "-e",
        "tell application \"iTerm2\"\n  tell current window\n    create tab with profile \"MyProfile\"\n    tell current session of current tab\n      write text \"cd '${workspaceFolder}' && my-command\"\n    end tell\n  end tell\nend tell"
      ]
    }
  }
}
```

### Environment Variables

Tasks inherit environment variables from VS Code settings. Configure in `.vscode/settings.json`:

```json
{
  "terminal.integrated.env.osx": {
    "PYTHONPATH": "${workspaceFolder}/python",
    "PATH": "/opt/homebrew/bin:${env:PATH}"
  }
}
```

## Troubleshooting

### Extension Not Found

If the extension isn't available:

1. Check VS Code Marketplace for alternative iTerm2 extensions
2. Use AppleScript-based approach (already configured in tasks)
3. Manually configure tasks with osascript

### Tasks Not Opening iTerm2

1. **Check iTerm2 Installation**:

   ```bash
   which osascript
   # Should return: /usr/bin/osascript
   ```

2. **Test AppleScript**:

   ```bash
   osascript -e 'tell application "iTerm2" to activate'
   ```

3. **Check Permissions**:
   - System Preferences → Security & Privacy → Accessibility
   - Ensure VS Code/Cursor has permissions

### Terminal Not Switching Directory

Tasks use `${workspaceFolder}` variable. If directory is wrong:

1. Ensure workspace is properly opened in VS Code
2. Check that `workspaceFolder` resolves correctly
3. Use absolute paths if needed

## Advanced Usage

### Creating Custom iTerm2 Tasks

Add custom tasks to `.vscode/tasks.json`:

```json
{
  "label": "My Custom Task (iTerm2)",
  "type": "shell",
  "command": "${workspaceFolder}/scripts/my_script.sh",
  "options": {
    "shell": {
      "executable": "/usr/bin/osascript",
      "args": [
        "-e",
        "tell application \"iTerm2\"\n  tell current window\n    create tab with default profile\n    tell current session of current tab\n      write text \"cd '${workspaceFolder}' && ./scripts/my_script.sh\"\n    end tell\n  end tell\nend tell"
      ]
    }
  },
  "presentation": {
    "reveal": "always",
    "panel": "dedicated"
  }
}
```

### Integration with tmux

For tasks that need tmux sessions:

```json
{
  "label": "Service Monitor (iTerm2 + tmux)",
  "type": "shell",
  "command": "tmux",
  "args": [
    "-CC",
    "new-session",
    "-d",
    "-s",
    "services",
    "${workspaceFolder}/scripts/launch-all-pwa-services.sh"
  ],
  "options": {
    "shell": {
      "executable": "/usr/bin/osascript",
      "args": [
        "-e",
        "tell application \"iTerm2\"\n  tell current window\n    create tab with default profile\n    tell current session of current tab\n      write text \"cd '${workspaceFolder}' && tmux -CC new-session -d -s services ./scripts/launch-all-pwa-services.sh\"\n    end tell\n  end tell\nend tell"
      ]
    }
  }
}
```

## Best Practices

1. **Use iTerm2 for Long Tasks**: Builds, tests, and service monitoring
2. **Use Integrated Terminal for Quick Tasks**: Quick commands, linting, etc.
3. **Configure Profiles**: Set up iTerm2 profiles for different task types
4. **Monitor Resources**: iTerm2 tasks don't block VS Code UI
5. **Session Management**: Use tmux for persistent service sessions

## Related Documentation

- [MCP Servers](./MCP_SERVERS.md) - iTerm2 MCP Server integration
- [PWA Patterns](./PWA_PATTERNS_APPLICABILITY.md) - Service launch patterns
- [Cursor Setup](./CURSOR_SETUP.md) - General Cursor/VS Code configuration

## See Also

- [iTerm2 Documentation](https://iterm2.com/documentation.html)
- [tmux Manual](https://man.openbsd.org/tmux)
- [VS Code Tasks Documentation](https://code.visualstudio.com/docs/editor/tasks)
